use crate::decoders::DecoderResult;
use crate::replay::Replay;

use std::fs::read_dir;
use std::io::Result;
use std::path::Path;

use serde::{Deserialize, Serialize};

pub fn visit_dirs(replays: &mut Vec<Replay>, dir: &Path) -> Result<()> {
    const VALID_TAGS: [&str; 10] = [
        "ASUS ROG",
        "DreamHack Masters",
        "HomeStory Cup",
        "IEM Katowice",
        "TSL",
        "Wardi",
        "OlimoLeague",
        "AlphaX",
        "WESG",
        "WCS",
    ];

    if dir.is_dir() {
        for entry in read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            // let filename = entry.file_name();
            if path.is_dir() && !path.to_str().unwrap().contains("PiG") {
                visit_dirs(replays, &entry.path())?;
            }

            match path.extension() {
                Some(extension) => {
                    if extension == "SC2Replay" {
                        let current_path = path.to_str().unwrap();
                        let mut tags = vec![];

                        for tag in VALID_TAGS {
                            if current_path.contains(tag) {
                                tags.push(tag.to_string());
                            }
                        }

                        let path_str = path.to_str().unwrap();
                        println!("parsing replay {:?}", path_str);
                        let bytes = std::fs::read(path_str).expect("Failed to read replay file");

                        let replay = Replay::new(bytes, path_str, tags);
                        let raw_played_at = &replay
                            .parsed
                            .player_info
                            .iter()
                            .find(|(field, _)| *field == "m_timeUTC")
                            .unwrap()
                            .1;
                        let mut played_at = 0;
                        if let DecoderResult::Value(value) = raw_played_at {
                            // TODO: this truncation is not working properly
                            played_at = *value as u128;
                        }
                        // game records time in window epoch for some reason
                        // https://en.wikipedia.org/wiki/Epoch_(computing)
                        played_at = (played_at / 10000000) - 11644473600;

                        // replays.push(replay);

                        // 1st Jan 2023 1672531200
                        // 1st Jan 2022 1640995200
                        // 1st Jan 2021 1609459200
                        // 1st Jan 2020 1577836800
                        // 1st Jan 2019 1546300800
                        // 1st Jan 2018 1514764800
                        if (1640995200..1672531200).contains(&played_at) {
                            replays.push(replay);
                        }
                    }
                }
                None => continue,
            }
        }
    }
    Ok(())
}
