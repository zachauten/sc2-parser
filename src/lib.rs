mod builds;
mod cluster;
mod decoders;
mod events;
mod game;
mod mpq;
mod parser;
mod protocol;
mod replay;
mod utils;

use serde::Serialize;

use wasm_bindgen::prelude::*;

extern crate console_error_panic_hook;

#[derive(Clone, Serialize)]
pub struct Player {
    id: u8,
    name: String,
    race: String,
}

#[derive(Clone, Serialize)]
pub struct TinybirdGame {
    content_hash: String,
    winner_id: u8,
    winner_name: String,
    winner_race: String,
    winner_build: String,
    loser_id: u8,
    loser_name: String,
    loser_race: String,
    loser_build: String,
    matchup: String,
    players: String,
    player_names: String,
    builds: String,
    map: String,
    game_length: u16,
    played_at: u128,
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct TinybirdTimelineEntry {
    content_hash: String,
    gameloop: u16,
    // win: bool,
    win: u8,
    player: String,
    player_race: String,
    // player_build: String,
    player_collection_rate: u16,
    player_army_value: u16,
    player_workers_active: u16,
    // player_workers_lost: u16,
    // player_workers_killed: u16,
    opponent: String,
    opponent_race: String,
    // opponent_build: String,
    opponent_collection_rate: u16,
    opponent_army_value: u16,
    opponent_workers_active: u16,
    // opponent_workers_lost: u16,
    // opponent_workers_killed: u16,
    matchup: String,
    map: String,
    game_length: u16,
    played_at: u128,
    game_version: String,
}

pub use crate::replay::Replay;

#[wasm_bindgen]
pub fn test() -> String {
    console_error_panic_hook::set_once();
    "Hello world!".to_string()
}
