use crate::decoders::{DecoderResult, EventEntry};
use crate::mpq::MPQArchive;
use crate::protocol::Protocol;

use serde::{Deserialize, Serialize};
use sha256::digest_bytes;

use std::io::{BufReader, Cursor, Read, Seek};
use wasm_bindgen::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    pub entries: Vec<(String, DecoderResult)>,
}

impl Event {
    pub fn new(entries: Vec<(String, DecoderResult)>) -> Event {
        Event { entries }
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerMetadata {
    pub PlayerID: u8,
    pub APM: f32,
    pub Result: String,
    pub SelectedRace: String,
    pub AssignedRace: String,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub Title: String,
    pub GameVersion: String,
    pub DataBuild: String,
    pub DataVersion: String,
    pub BaseBuild: String,
    pub Duration: u16,
    pub Players: Vec<PlayerMetadata>,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Parsed {
    #[wasm_bindgen(skip)]
    pub player_info: Vec<EventEntry>,
    #[wasm_bindgen(skip)]
    pub tracker_events: Vec<Event>,
    pub metadata: Metadata,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Serialize, Deserialize)]
pub struct Replay {
    pub file_path: String,
    pub content_hash: String,
    pub parsed: Parsed,
}

#[wasm_bindgen(getter_with_clone)]
impl Replay {
    #[wasm_bindgen(constructor)]
    pub fn new(bytes: Vec<u8>, path: &str) -> Self {
        let content_hash = digest_bytes(&bytes);
        let cursor = Cursor::new(bytes);
        let reader = BufReader::new(cursor);
        let archive = MPQArchive::new(reader);
        let protocol: Protocol = Protocol::new();
        let parsed = Self::parse(archive, protocol);
        Self {
            file_path: path.to_string(),
            content_hash,
            parsed,
        }
    }

    fn parse<T: Seek + Read>(mut archive: MPQArchive<T>, protocol: Protocol) -> Parsed {
        let contents = archive.read_file("replay.tracker.events").unwrap();
        let raw_metadata = archive.read_file("replay.gamemetadata.json").unwrap();
        let metadata_str = String::from_utf8(raw_metadata.clone()).unwrap();
        let metadata: Metadata = serde_json::from_str(&metadata_str).unwrap();
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
