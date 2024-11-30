use crate::decoders::{DecoderResult, EventEntry};
use crate::mpq::MPQArchive;
use crate::protocol::Protocol;

use serde::{Deserialize, Serialize};
use sha256::digest_bytes;

use std::io::{BufReader, Cursor, Read, Seek};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    pub entries: Vec<EventEntry>,
}

impl Event {
    pub fn new(entries: Vec<EventEntry>) -> Self {
        Self { entries }
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerMetadata {
    #[serde(rename(deserialize = "PlayerID"))]
    pub player_id: u8,
    #[serde(rename(deserialize = "APM"))]
    pub apm: f32,
    #[serde(rename(deserialize = "Result"))]
    pub result: String,
    #[serde(rename(deserialize = "SelectedRace"))]
    pub selected_race: String,
    #[serde(rename(deserialize = "AssignedRace"))]
    pub assigned_race: String,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Metadata {
    #[serde(rename(deserialize = "Title"))]
    pub title: String,
    #[serde(rename(deserialize = "GameVersion"))]
    pub game_version: String,
    #[serde(rename(deserialize = "DataBuild"))]
    pub data_build: String,
    #[serde(rename(deserialize = "DataVersion"))]
    pub data_version: String,
    #[serde(rename(deserialize = "BaseBuild"))]
    pub base_build: String,
    #[serde(rename(deserialize = "Duration"))]
    pub duration: u16,
    #[serde(rename(deserialize = "Players"))]
    pub players: Vec<PlayerMetadata>,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Parsed {
    pub player_info: Vec<EventEntry>,
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

#[wasm_bindgen]
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
