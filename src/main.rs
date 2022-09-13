// Copyright (C) 2019 - 2022 Wilfred Bos
// Licensed under the MIT license. See the LICENSE file for the terms and conditions.

mod config;
mod player_id;

#[path = "./utils/hvsc.rs"] mod hvsc;

use std::cmp::min;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::process::exit;
use std::time::Instant;

use rayon::prelude::*;
use self::player_id::{PlayerId, SidIdHolder, SidInfo};
use self::config::Config;

const DEFAULT_FILENAME_COL_WIDTH: usize = 56;

fn main() {
    match run() {
        Ok(_) => {}
        Err(message) => {
            eprintln!("ERROR: {}", message);
            exit(1);
        }
    }
}

pub struct PlayerInfo {
    pub players: Vec<(String, Vec<usize>)>,
    pub filename: String,
}

fn run() -> Result<(), String> {
    if env::args().count() <= 1 {
        print_usage();
        return Ok(());
    }

    let config = Config::read()?;

    let start_time = Instant::now();

    if config.verify_signatures {
        verify_signatures(config.config_file.clone())?;
        verify_sidid_info(config.config_file)?;
        return Ok(());
    }

    if config.show_player_info {
        let player_infos = load_info_file(config.config_file)?;
        let player_name = config.player_name.unwrap();
        let player_info = PlayerId::find_player_info(&player_infos, &player_name);
        if let Some(player_info) = player_info {
            println!("Player info:\n\n{}", player_info.signature_name);
            for info_line in player_info.info_lines {
                println!("{}", info_line);
            }
        } else {
            println!("No info found for player ID: {}", &player_name);
        }
        return Ok(());
    }

    let sid_ids = load_config_file(config.config_file, config.player_name)?;

    if config.scan_hvsc {
        println!("Scanning HVSC location: {}", config.base_path);
    }

    println!("Processing...");

    let max_depth = if config.recursive { usize::MAX } else { 1 };

    let files = if !config.filename.is_empty() {
        globwalk::GlobWalkerBuilder::from_patterns(&config.base_path, &[&config.filename])
            .max_depth(max_depth)
            .build().unwrap()
            .into_iter()
            .filter_map(Result::ok)
            .map(|p| p.path().display().to_string())
            .collect::<Vec<String>>()
    } else {
        vec![]
    };

    if files.is_empty() {
        println!("No file(s) found.");
        return Ok(());
    }

    let mut identified_players = 0;
    let mut identified_files = 0;
    let mut unidentified_files = 0;
    let processed_files = files.len();

    let pool = rayon::ThreadPoolBuilder::new().num_threads(config.cpu_threads).build().unwrap();
    let _ = pool.install(|| {
        let players_found: Vec<PlayerInfo> = files
            .par_iter()
            .map(|path| {
                let players_found = PlayerId::find_players_in_file(path, &sid_ids, config.scan_for_multiple);

                PlayerInfo {
                    players: players_found,
                    filename: path.to_owned()
                }
            })
            .filter(|info|
                (info.players.is_empty() && (config.only_list_unidentified || config.list_unidentified))||
                (!info.players.is_empty() && !config.only_list_unidentified))
            .collect();

        let filename_strip_length = get_filename_strip_length(config.base_path, &files);
        let filename_width = calculate_filename_width(config.truncate_filenames, &players_found, filename_strip_length);

        for player_info in &players_found {
            let filename = player_info.filename[filename_strip_length..].to_string();
            let filename_size = if config.truncate_filenames {
                min(filename.len(), filename_width)
            } else {
                filename.len()
            };

            if player_info.players.is_empty() {
                unidentified_files += 1;

                println!("{:<0width$} >> UNIDENTIFIED <<", filename[..filename_size].replace('\\', "/"), width = filename_width);
            } else {
                identified_files += 1;
                identified_players += player_info.players.len();

                for (index, player) in player_info.players.iter().enumerate() {
                    let player_name = if config.display_hex_offset {
                        let player_indexes = player.1.iter().map(|index| format!("${:04X}", index)).collect::<Vec<String>>();
                        format!("{} {}", player.0, player_indexes.join(" "))
                    } else {
                        player.0.to_string()
                    };
                    if index == 0 {
                        println!("{:<0width$} {}", filename[..filename_size].replace('\\', "/"), player_name, width = filename_width);
                    } else {
                        println!("{:<0width$} {}", "", player_name, width = filename_width);
                    }
                }
            }
        }

        if identified_files > 0 {
            unidentified_files = processed_files - identified_files;

            output_occurrence_statistics(&sid_ids, &players_found);
        }
    });

    println!("\nSummary:");
    println!("Identified players    {:>9}", identified_players);
    println!("Identified files      {:>9}", identified_files);
    println!("Unidentified files    {:>9}", unidentified_files);
    println!("Total files processed {:>9}", processed_files);

    output_elapsed_time(start_time);
    Ok(())
}

fn output_elapsed_time(start_time: Instant) {
    let time_millis = start_time.elapsed().as_millis();
    let time_seconds = time_millis / 1000;
    let seconds = time_seconds % 60;
    let minutes = time_seconds / 60 % 60;
    let hours = time_seconds / 60 / 60;
    println!("\nTotal time: {:0>2}:{:0>2}:{:0>2} (+{} milliseconds)", hours, minutes, seconds, time_millis % 1000);
}

fn verify_signatures(config_file: Option<String>) -> Result<bool, String> {
    println!("Checking signatures...");

    let config_path = get_config_path(config_file)?;
    println!("Verify config file: {}\n", config_path.display());

    let issues_found = PlayerId::verify_config_file(&config_path)?;

    if !issues_found {
        println!("No issues found in configuration.");
    }
    Ok(issues_found)
}

fn verify_sidid_info(config_file: Option<String>) -> Result<bool, String> {
    println!("\nChecking info file...");

    let config_path = get_config_path(config_file)?;
    let sid_ids = PlayerId::load_config_file(&config_path, None)?;

    let config_path_string = config_path.display().to_string().replace(".cfg", ".nfo");
    let config_path = PlayerId::get_config_path(Some(config_path_string.clone()));
    if let Ok(config_path) = config_path {
        println!("Verify info file: {}\n", config_path.display());

        let issues_found = PlayerId::verify_info_file(&config_path, &sid_ids)?;

        if !issues_found {
            println!("No issues found in info file.");
        }
        Ok(issues_found)
    } else {
        println!("\nNo info file found: {}", config_path_string);
        Ok(true)
    }
}

fn load_config_file(config_file: Option<String>, player_name: Option<String>) -> Result<Vec<SidIdHolder>, String> {
    let config_path = get_config_path(config_file)?;
    println!("Using config file: {}\n", config_path.display());

    let sid_ids = PlayerId::load_config_file(&config_path, player_name)?;
    if sid_ids.is_empty() {
        return Err("No signature defined.".to_string());
    }
    Ok(sid_ids)
}

fn load_info_file(config_file: Option<String>) -> Result<Vec<SidInfo>, String> {
    let config_path_string = get_config_path(config_file)?.display().to_string().replace(".cfg", ".nfo");
    let config_path = PlayerId::get_config_path(Some(config_path_string))?;
    println!("Using info file: {}\n", config_path.display());

    let sid_infos = PlayerId::load_info_file(&config_path)?;
    if sid_infos.is_empty() {
        return Err("No signature defined.".to_string());
    }
    Ok(sid_infos)
}

fn get_config_path(config_file: Option<String>) -> Result<PathBuf, String> {
    let config_file = if let Some(config_file) = config_file {
        if config_file.is_empty() {
            return Err("Invalid config filename. No space allowed after -f switch.".to_string());
        }
        Some(config_file)
    } else {
        let config_file = env::var("SIDIDCFG");
        if let Ok(config_file) = config_file {
            Some(config_file)
        } else {
            Some("sidid.cfg".to_string())
        }
    };

    let config_path = PlayerId::get_config_path(config_file)?;
    Ok(config_path)
}

fn calculate_filename_width(truncate_filenames: bool, players_found: &[PlayerInfo], filename_strip_length: usize) -> usize {
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

fn output_occurrence_statistics(sid_ids: &Vec<SidIdHolder>, player_info: &Vec<PlayerInfo>) {
    println!("\nDetected players          Count");
    println!("-------------------------------");

    let mut player_occurrence: HashMap<String, i32> = HashMap::new();
    for players in player_info {
        for player in &players.players {
            let occurrence: i32 = *player_occurrence.get(&player.0).unwrap_or(&0);
            player_occurrence.insert(player.0.to_owned(), occurrence + 1);
        }
    }

    let mut previous_player_name = "";
    for sid_id in sid_ids {
        if !sid_id.signature_name.eq(previous_player_name) {
            previous_player_name = &sid_id.signature_name;
            let occurrence = player_occurrence.get(&sid_id.signature_name);
            if let Some(occurrence) = occurrence {
                println!("{:<24} {:>6}", sid_id.signature_name, occurrence);
            }
        }
    }
}

fn print_usage() {
    println!("C64 Music Player Identifier (PI) v2.0 - Copyright (c) 2012-2022 Wilfred Bos");
    println!("\nUsage: player-id <options> <file_path_pattern>");
    println!("\n<Options>");
    println!("  -c{{max_threads}}: set the maximum CPU threads to be used [Default is all]");
    println!("  -f{{config_file}}: config file [Default SIDIDCFG env. var. / sidid.cfg file]");
    println!("  -h: scan HVSC location [Uses HVSC env. variable for HVSC path]");
    println!("  -n: show player info [use together with -p option]");
    println!("  -m: scan for multiple signatures");
    println!("  -o: list only unidentified files");
    println!("  -p{{player name}}: scan only for specific player name");
    println!("  -s: include subdirectories");
    println!("  -t: truncate filenames");
    println!("  -u: list also unidentified files");
    println!("  -v: verify signatures");
    println!("  -x: display hexadecimal offset of signature found");
}
