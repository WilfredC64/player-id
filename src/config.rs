// Copyright (C) 2022 Wilfred Bos
// Licensed under the MIT license. See the LICENSE file for the terms and conditions.

use std::env;

pub struct Config {
    pub cpu_threads: usize,
    pub display_hex_offset: bool,
    pub list_unidentified: bool,
    pub only_list_unidentified: bool,
    pub recursive: bool,
    pub scan_for_multiple: bool,
    pub scan_hvsc: bool,
    pub show_player_info: bool,
    pub truncate_filenames: bool,
    pub verify_signatures: bool,
    pub player_name: Option<String>,
    pub config_file: Option<String>,
    pub base_path: String,
    pub filename: String
}

impl Config {
    pub fn read() -> Result<Config, String> {
        let max_threads = rayon::current_num_threads();
        let mut cpu_threads = max_threads;
        let mut display_hex_offset = false;
        let mut list_unidentified = false;
        let mut only_list_unidentified = false;
        let mut recursive = false;
        let mut scan_for_multiple = false;
        let mut scan_hvsc = false;
        let mut show_player_info = false;
        let mut truncate_filenames = false;
        let mut verify_signatures = false;
        let mut config_file = None;
        let mut player_name = None;

        for argument in env::args().filter(|arg| arg.len() > 1 && arg.starts_with('-')) {
            match &argument[1..2] {
                "c" => cpu_threads = Self::parse_argument_number("Max threads", &argument[2..])? as usize,
                "f" => config_file = Some(argument[2..].to_string()),
                "h" => scan_hvsc = true,
                "m" => scan_for_multiple = true,
                "n" => show_player_info = true,
                "o" => only_list_unidentified = true,
                "p" => player_name = Some(argument[2..].to_string()),
                "t" => truncate_filenames = true,
                "s" => recursive = true,
                "u" => list_unidentified = true,
                "v" => verify_signatures = true,
                "x" => display_hex_offset = true,
                _ => return Err(format!("Unknown option: {}", argument))
            }
        }

        if cpu_threads > max_threads {
            cpu_threads = max_threads;
        }

        let (mut base_path, mut filename) = Self::get_filename_and_base_path();

        if scan_hvsc {
            Self::set_hvsc_config(&mut recursive, &mut base_path, &mut filename)?;
        }

        if show_player_info {
            Self::validate_player_info_option(show_player_info, &player_name)?;
        }

        Ok(Config {
            cpu_threads,
            config_file,
            display_hex_offset,
            list_unidentified,
            recursive,
            scan_for_multiple,
            scan_hvsc,
            show_player_info,
            truncate_filenames,
            only_list_unidentified,
            verify_signatures,
            player_name,
            base_path,
            filename
        })
    }

    fn validate_player_info_option(show_player_info: bool, player_name: &Option<String>) -> Result<(), String>{
        if show_player_info && player_name.is_none() {
            return Err("Player info can only be used when -p option is provided with a player name.".to_string());
        }
        Ok(())
    }

    fn set_hvsc_config(recursive: &mut bool, base_path: &mut String, filename: &mut String) -> Result<(), String>{
        if let Ok(hvsc_location) = env::var("HVSC") {
            *recursive = true;
            *base_path = hvsc_location;
            if filename.is_empty() {
                *filename = "*.sid".to_string();
            }
        } else {
            return Err("HVSC environment variable not found.".to_string());
        }
        Ok(())
    }

    fn get_filename_and_base_path() -> (String, String) {
        let filename = env::args().last().unwrap();
        if !filename.starts_with('-') {
            Self::split_file_path(filename.trim())
        } else {
            ("".to_string(), "".to_string())
        }
    }

    fn parse_argument_number(arg_name: &str, arg_value: &str) -> Result<u32, String> {
        let number = match arg_value.parse::<u32>() {
            Ok(i) => i,
            Err(_e) => return Err(format!("{} must be a valid number.", arg_name))
        };
        if number > 0 {
            Ok(number)
        } else {
            Err(format!("{} must be higher than 0.", arg_name))
        }
    }

    fn split_file_path(filename: &str) -> (String, String) {
        let mut index = filename.rfind('/');
        if index.is_none() {
            index = filename.rfind('\\');
        }
        if let Some(index) = index {
            return (filename[..index].to_owned(), filename[index + 1..].to_owned())
        }
        (".".to_string(), filename.to_string())
    }
}
