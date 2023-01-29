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
    pub filename: String,
    pub convert_file_format: Option<String>
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
        let mut convert_file_format = None;

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
                "w" => convert_file_format = Some(argument[2..].to_string()),
                "x" => display_hex_offset = true,
                _ => return Err(format!("Unknown option: {argument}"))
            }
        }

        if cpu_threads > max_threads {
            cpu_threads = max_threads;
        }

        let (mut base_path, mut filename) = Self::get_filename_and_base_path();

        if scan_hvsc {
            Self::set_hvsc_config(&mut recursive, &mut base_path, &mut filename)?;
        }

        if config_file.is_none() {
            config_file = env::var("SIDIDCFG").ok();
        }

        if show_player_info {
            Self::validate_player_info_option(show_player_info, player_name.as_ref())?;
        } else {
            Self::validate_player_name(player_name.as_ref())?;
        }

        Self::validate_file_format_option(&convert_file_format)?;

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
            filename,
            convert_file_format
        })
    }

    fn validate_player_info_option(show_player_info: bool, player_name: Option<&String>) -> Result<(), String> {
        if show_player_info && (player_name.is_none() || player_name.unwrap().is_empty()) {
            return Err("Player info can only be used when -p option is provided with a player name.".to_string());
        }
        Ok(())
    }

    fn validate_player_name(player_name: Option<&String>) -> Result<(), String> {
        if player_name.is_some() && player_name.unwrap().is_empty() {
            return Err("Player name cannot be empty.".to_string());
        }
        Ok(())
    }

    fn validate_file_format_option(file_format: &Option<String>) -> Result<(), String> {
        if let Some(file_format) = file_format {
            match file_format.as_str() {
                "o" | "n" => {},
                _ => return Err("Output format should be specified with -wo for old format or -wn for new format".to_string())
            }
        }
        Ok(())
    }

    fn set_hvsc_config(recursive: &mut bool, base_path: &mut String, filename: &mut String) -> Result<(), String> {
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
            Err(_e) => return Err(format!("{arg_name} must be a valid number."))
        };
        if number > 0 {
            Ok(number)
        } else {
            Err(format!("{arg_name} must be higher than 0."))
        }
    }

    fn split_file_path(filename: &str) -> (String, String) {
        let filename_unix = filename.replace('\\', "/");
        if let Some(index) = filename_unix.rfind('/') {
            return match index {
                0 => (filename[..1].to_string(), filename[1..].to_owned()),
                x if x > 1 && filename_unix.starts_with("./") => (filename[2..index].to_owned(), filename[index + 1..].to_owned()),
                _ => (filename[..index].to_owned(), filename[index + 1..].to_owned())
            }
        }
        (".".to_string(), filename.to_string())
    }
}

#[cfg(test)]
#[path = "./config_test.rs"]
mod config_test;
