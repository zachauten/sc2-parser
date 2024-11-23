use crate::decoders::{DecoderResult, EventEntry};
use crate::mpq::MPQArchive;
use crate::protocol::Protocol;

use serde::{Deserialize, Serialize};
use sha256::digest_bytes;

use std::io::{BufReader, Cursor, Read, Seek};

use wasm_bindgen::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    pub entries: Vec<(String, DecoderResult)>,
}

impl Event {
    pub fn new(entries: Vec<(String, DecoderResult)>) -> Event {
        Event { entries }
    }
}

#[derive(Debug, Deserialize)]
pub struct PlayerMetadata<'a> {
    pub PlayerID: u8,
    pub APM: f32,
    pub Result: &'a str,
    pub SelectedRace: &'a str,
    pub AssignedRace: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct Metadata<'a> {
    pub Title: &'a str,
    pub GameVersion: &'a str,
    // pub DataBuild: &'a str,
    // pub DataVersion: &'a str,
    // pub BaseBuild: &'a str,
    pub Duration: u16,
    // pub IsNotAvailable: bool,
    pub Players: Vec<PlayerMetadata<'a>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Parsed {
    pub player_info: Vec<EventEntry>,
    pub tracker_events: Vec<Event>,
    pub metadata: String,
}

// #[wasm_bindgen(getter_with_clone)]
#[derive(Serialize, Deserialize)]
pub struct Replay {
    pub file_path: String,
    pub content_hash: String,
    pub parsed: Parsed,
}

#[wasm_bindgen]
impl Replay {
    #[wasm_bindgen(constructor)]
    pub fn constructor(bytes: Vec<u8>, path: &str) -> JsValue {
        let replay = Self::new(bytes, path);
        serde_wasm_bindgen::to_value(&replay).unwrap()
    }
}

impl Replay {
    pub fn new(bytes: Vec<u8>, path: &str) -> Replay {
        let content_hash = digest_bytes(&bytes);
        let cursor = Cursor::new(bytes);
        let reader = BufReader::new(cursor);
        let archive = MPQArchive::new(reader);
        let protocol: Protocol = Protocol::new();
        let parsed = Replay::parse(archive, protocol);
        Replay {
            file_path: path.to_string(),
            content_hash,
            parsed,
        }
    }

    fn parse<T: Seek + Read>(mut archive: MPQArchive<T>, protocol: Protocol) -> Parsed {
        let contents = archive.read_file("replay.tracker.events").unwrap();
        let raw_metadata = archive.read_file("replay.gamemetadata.json").unwrap();
        let metadata = String::from_utf8(raw_metadata.clone()).unwrap();
        let details = archive.read_file("replay.details").unwrap();
        let player_info = protocol.decode_replay_details(details);
        let tracker_events = protocol.decode_replay_tracker_events(contents);
        Parsed {
            player_info,
            tracker_events,
            metadata,
        }
    }
}
