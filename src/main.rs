// Copyright (C) 2019 - 2022 Wilfred Bos
// Licensed under the MIT license. See the LICENSE file for the terms and conditions.

mod config;
mod player_id;

#[path = "./utils/hvsc.rs"] mod hvsc;

use std::cmp::min;
use std::collections::HashMap;
use std::env;
use std::process::exit;
use std::time::Instant;

use rayon::prelude::*;
use self::config::Config;
use self::player_id::{PlayerId, SignatureConfig, SignatureMatch};

const DEFAULT_FILENAME_COL_WIDTH: usize = 56;

fn main() {
    if env::args().count() <= 1 {
        print_usage();
        return;
    }

    match run() {
        Ok(_) => {}
        Err(message) => {
            eprintln!("ERROR: {}\r", message);
            exit(1);
        }
    }
}

pub struct FileMatches {
    pub matches: Vec<SignatureMatch>,
    pub filename: String,
}

fn run() -> Result<(), String> {
    let config = Config::read()?;

    if config.verify_signatures {
        PlayerId::verify_signatures(config.config_file.clone())?;
        PlayerId::verify_signature_info(config.config_file)?;
        return Ok(());
    } else if config.show_player_info {
        PlayerId::display_player_info(config.config_file, &config.player_name.unwrap())?;
        return Ok(());
    }

    if config.scan_hvsc {
        eprintln!("Scanning HVSC location: {}\r", config.base_path);
    }

    eprintln!("Processing...\r");

    let start_time = Instant::now();
    let signature_ids = PlayerId::load_config_file(config.config_file, config.player_name)?;

    let max_depth = if config.recursive { usize::MAX } else { 1 };

    let files = if !config.filename.is_empty() {
        globwalk::GlobWalkerBuilder::from_patterns(&config.base_path, &[&config.filename])
            .max_depth(max_depth)
            .case_insensitive(true)
            .build().unwrap()
            .into_iter()
            .filter_map(Result::ok)
            .map(|p| p.path().display().to_string())
            .collect::<Vec<String>>()
    } else {
        vec![]
    };

    if files.is_empty() {
        eprintln!("No file(s) found.\r");
        return Ok(());
    }

    let mut identified_players = 0;
    let mut identified_files = 0;
    let mut unidentified_files = 0;
    let processed_files = files.len();

    let pool = rayon::ThreadPoolBuilder::new().num_threads(config.cpu_threads).build().unwrap();
    let _ = pool.install(|| {
        let matches: Vec<FileMatches> = files
            .par_iter()
            .map(|path| {
                let matches = PlayerId::find_players_in_file(path, &signature_ids, config.scan_for_multiple);

                FileMatches {
                    matches,
                    filename: path.to_owned()
                }
            })
            .filter(|info|
                (info.matches.is_empty() && (config.only_list_unidentified || config.list_unidentified))||
                (!info.matches.is_empty() && !config.only_list_unidentified))
            .collect();

        let filename_strip_length = get_filename_strip_length(config.base_path, &files);
        let filename_width = calculate_filename_width(config.truncate_filenames, &matches, filename_strip_length);

        for file_matches in &matches {
            let filename = file_matches.filename[filename_strip_length..].to_string();
            let filename_size = if config.truncate_filenames {
                min(filename.len(), filename_width)
            } else {
                filename.len()
            };

            if file_matches.matches.is_empty() {
                unidentified_files += 1;

                println!("{:<0width$} >> UNIDENTIFIED <<\r", filename[..filename_size].replace('\\', "/"), width = filename_width);
            } else {
                identified_files += 1;
                identified_players += file_matches.matches.len();

                for (index, player) in file_matches.matches.iter().enumerate() {
                    let player_name = if config.display_hex_offset {
                        let player_indexes = player.indexes.iter().map(|index| format!("${:04X}", index)).collect::<Vec<String>>();
                        format!("{} {}", player.signature_name, player_indexes.join(" "))
                    } else {
                        player.signature_name.to_string()
                    };
                    if index == 0 {
                        println!("{:<0width$} {}\r", filename[..filename_size].replace('\\', "/"), player_name, width = filename_width);
                    } else {
                        println!("{:<0width$} {}\r", "", player_name, width = filename_width);
                    }
                }
            }
        }

        if identified_files > 0 {
            unidentified_files = processed_files - identified_files;

            output_occurrence_statistics(&signature_ids, &matches);
        }
    });

    println!("\r\nSummary:\r");
    println!("Identified players    {:>9}\r", identified_players);
    println!("Identified files      {:>9}\r", identified_files);
    println!("Unidentified files    {:>9}\r", unidentified_files);
    println!("Total files processed {:>9}\r", processed_files);

    output_elapsed_time(start_time);
    Ok(())
}

fn output_elapsed_time(start_time: Instant) {
    let time_millis = start_time.elapsed().as_millis();
    let time_seconds = time_millis / 1000;
    let seconds = time_seconds % 60;
    let minutes = time_seconds / 60 % 60;
    let hours = time_seconds / 60 / 60;
    eprintln!("\r\nTotal time: {:0>2}:{:0>2}:{:0>2} (+{} milliseconds)\r", hours, minutes, seconds, time_millis % 1000);
}

fn calculate_filename_width(truncate_filenames: bool, players_found: &[FileMatches], filename_strip_length: usize) -> usize {
    if !truncate_filenames {
        let longest_filename = players_found.iter().max_by(|x, y| x.filename.len().cmp(&y.filename.len()));
        if let Some(longest_filename) = longest_filename {
            let filename_width = longest_filename.filename.len() - filename_strip_length;

            if filename_width > DEFAULT_FILENAME_COL_WIDTH {
                return filename_width;
            }
        }
    }
    DEFAULT_FILENAME_COL_WIDTH
}

fn get_filename_strip_length(base_path: String, files: &Vec<String>) -> usize {
    let base_path_length = if base_path.eq(".") { 2 } else { 0 };
    if !files.is_empty() {
        let hvsc_root = hvsc::get_hvsc_root(files.first().unwrap());
        if let Some(hvsc_root) = hvsc_root {
            hvsc_root.len() + 1
        } else {
            base_path_length
        }
    } else {
        base_path_length
    }
}

fn output_occurrence_statistics(signature_ids: &Vec<SignatureConfig>, player_info: &Vec<FileMatches>) {
    println!("\r\nDetected players          Count\r");
    println!("-------------------------------\r");

    let mut player_occurrence: HashMap<String, i32> = HashMap::new();
    for players in player_info {
        for player in &players.matches {
            let occurrence: i32 = *player_occurrence.get(&player.signature_name).unwrap_or(&0);
            player_occurrence.insert(player.signature_name.to_owned(), occurrence + 1);
        }
    }

    let mut previous_player_name = "";
    for signature_id in signature_ids {
        if !signature_id.signature_name.eq(previous_player_name) {
            previous_player_name = &signature_id.signature_name;
            if let Some(occurrence) = player_occurrence.get(&signature_id.signature_name) {
                println!("{:<24} {:>6}\r", signature_id.signature_name, occurrence);
            }
        }
    }
}

fn print_usage() {
    println!("C64 Music Player Identifier (PI) v2.0 - Copyright (c) 2012-2022 Wilfred Bos\r");
    println!("\r\nUsage: player-id <options> <file_path_pattern>\r");
    println!("\r\n<Options>\r");
    println!("  -c{{max_threads}}: set the maximum CPU threads to be used [Default is all]\r");
    println!("  -f{{config_file}}: config file [Default SIDIDCFG env. var. / sidid.cfg file]\r");
    println!("  -h: scan HVSC location [Uses HVSC env. variable for HVSC path]\r");
    println!("  -n: show player info [use together with -p option]\r");
    println!("  -m: scan for multiple signatures\r");
    println!("  -o: list only unidentified files\r");
    println!("  -p{{player_name}}: scan only for specific player name\r");
    println!("  -s: include subdirectories\r");
    println!("  -t: truncate filenames\r");
    println!("  -u: list also unidentified files\r");
    println!("  -v: verify signatures\r");
    println!("  -x: display hexadecimal offset of signature found\r");
}
