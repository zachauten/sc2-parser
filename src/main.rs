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

use crate::builds::Builds;
use crate::parser::{ReplayParser, ReplaySummary};
use crate::replay::Replay;
use crate::utils::visit_dirs;

use csv::Writer;
use serde::Serialize;
use std::collections::HashSet;
use std::fs::File;
use std::path::Path;

use std::time::Instant;
use wasm_bindgen::prelude::*;
extern crate console_error_panic_hook;

#[derive(Serialize)]
#[serde(untagged)]
pub enum SummaryStat {
    ResourceValues((u16, u16)),
    Value(u16),
}

#[derive(Clone, Serialize)]
pub struct Player {
    id: u8,
    name: String,
    race: String,
}

#[derive(Serialize)]
struct SerializedReplays {
    replays: Vec<ReplaySummary>,
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

fn main() {
    let now = Instant::now();

    let replay_dir = Path::new("/mnt/c/Users/zacha/Documents/StarCraft II/Accounts/50968896/1-S2-1-2508124/Replays/Multiplayer/");
    let mut replays: Vec<Replay> = vec![];
    let mut seen_replays: HashSet<String> = HashSet::new();
    visit_dirs(&mut replays, replay_dir).unwrap();

    let num_replays = replays.len();
    println!("visited {:?} files in {:.2?}", num_replays, now.elapsed());

    let replay_summaries: Vec<ReplaySummary> = vec![];
    let mut replay_builds: Vec<String> = vec![];
    let mut result = SerializedReplays {
        replays: replay_summaries,
    };

    let mut tinybird_serialized: Vec<TinybirdGame> = vec![];

    let mut replay_parser = ReplayParser::new();

    let mut build_tokens = Builds::new();

    for replay in replays {
        let content_hash = replay.content_hash.clone();
        // don't include replays we've seen before
        if seen_replays.contains(&content_hash) {
            continue;
        }

        // refactor event and replay parsers into single parser
        let replay_summary = match replay_parser.parse_replay(replay, &mut replay_builds) {
            Ok(summary) => summary,
            Err(e) => {
                // panic!("Error parsing replay: {e}");
                continue;
            }
        };
        result.replays.push(replay_summary);
        seen_replays.insert(content_hash);
    }

    for replay_summary in result.replays {
        if !replay_summary.tinybird.winner_build.is_empty()
            && !replay_summary.tinybird.loser_build.is_empty()
        {
            tinybird_serialized.push(replay_summary.tinybird.clone());
        }

        let mut races = vec![];
        let mut matchup = vec![];
        for player in &replay_summary.players {
            races.push(player.race.clone());
            matchup.push(player.race.clone());
        }
        matchup.sort();

        let matchup_prefix = matchup.join(",");

        for (p_id, player_build_index) in replay_summary.build_mappings.iter().enumerate() {
            let player_build = replay_builds[*player_build_index as usize]
                .split(",")
                .map(|s| s.to_string())
                .collect();
            let token_prefix = format!("{}-{}", races[p_id], matchup_prefix);

            let win = (p_id + 1) == replay_summary.winner as usize;
            build_tokens.generate_tokens(&player_build, win, token_prefix);
        }
    }

    build_tokens.generate_matchup_build_trees();

    println!(
        "{:?} replays parsed in {:.2?}, {:?} per replay",
        num_replays,
        now.elapsed(),
        now.elapsed() / num_replays as u32
    );

    let build_output = File::create("generated/builds.json").unwrap();
    serde_json::to_writer(&build_output, &build_tokens.builds);

    let raw_build_tree_output = File::create("generated/raw_build_tree.json").unwrap();
    serde_json::to_writer(&raw_build_tree_output, &build_tokens.raw_build_tree);

    let build_token_output = File::create("generated/tokens.json").unwrap();
    serde_json::to_writer(&build_token_output, &build_tokens.build_token_path_mappings);

    File::create("tinybird_sc2.csv").unwrap();
    let mut wtr = Writer::from_path("tinybird_sc2.csv").unwrap();
    for record in tinybird_serialized {
        wtr.serialize(record).unwrap();
    }
    wtr.flush().unwrap();

    println!("replays serialized in {:?}", now.elapsed());
}

#[wasm_bindgen]
pub fn test() -> String {
    console_error_panic_hook::set_once();
    "Hello world!".to_string()
}
