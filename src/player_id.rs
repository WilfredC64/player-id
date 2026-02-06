// Copyright (C) 2019 - 2022 Wilfred Bos
// Licensed under the MIT license. See the LICENSE file for the terms and conditions.

#![allow(dead_code)]

mod signature;
mod validate;

use std::env;
use std::fs::{self, File, read};
use std::io::{self, BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

use encoding_rs::WINDOWS_1252;
use encoding_rs_io::DecodeReaderBytesBuilder;

use super::sid_file;
use signature::Signature;
pub use signature::{SignatureConfig, SignatureInfo, SignatureMatch};

const DEFAULT_CONFIG_FILE_NAME: &str = "sidid.cfg";

pub struct PlayerId {}

impl PlayerId {
    pub fn find_players_in_buffer(buffer: &[u8], signature_ids: &Vec<SignatureConfig>, scan_for_multiple: bool) -> Vec<SignatureMatch> {
        Signature::find_signatures(buffer, 0, signature_ids, scan_for_multiple)
    }

    pub fn find_players_in_file(filename: &str, signature_ids: &Vec<SignatureConfig>, scan_for_multiple: bool) -> Vec<SignatureMatch> {
        if let Ok(data) = read(filename) {
            let data_offset = Self::get_data_offset(filename, &data);

            Signature::find_signatures(&data, data_offset, signature_ids, scan_for_multiple)
        } else {
            vec![]
        }
    }

    pub fn find_player_info<'a>(signature_infos: &'a [SignatureInfo], player_name: &str) -> Option<&'a SignatureInfo> {
        Signature::find_signature_info(signature_infos, player_name)
    }

    pub fn is_config_file(filename: &str) -> bool {
        if let Ok(path) = PlayerId::get_config_path_with_fallback(filename) && let Ok(file) = File::open(path) {
            let lines = Self::get_first_few_lines_from_file(file);
            return Signature::is_config_file(&lines);
        }
        false
    }

    pub fn load_config_file(config_path: &PathBuf, player_name: Option<&String>) -> Result<Vec<SignatureConfig>, String> {
        let lines = Self::read_text_file(config_path)?;
        let signature_ids = Signature::read_config_lines(&lines, player_name)?;
        if signature_ids.is_empty() {
            return Err(if let Some(player_name) = player_name {
                format!("No signature found with name: {player_name}")
            } else {
                "No signature defined.".to_string()
            })
        }
        Ok(signature_ids)
    }

    pub fn load_info_file(config_path: &PathBuf) -> Result<Vec<SignatureInfo>, String> {
        let lines = Self::read_text_file(config_path)?;
        let signature_infos = Signature::read_info_lines(&lines)?;
        if signature_infos.is_empty() {
            return Err("No info sections defined.".to_string());
        }
        Ok(signature_infos)
    }

    pub fn get_info_file_path(config_file: Option<&String>) -> Result<PathBuf, String> {
        let config_path_string = PlayerId::get_config_path(config_file)?.with_extension("nfo");
        PlayerId::get_config_path_with_fallback(&config_path_string.to_string_lossy())
    }

    pub fn get_config_path(config_file: Option<&String>) -> Result<PathBuf, String> {
        let config_file = if let Some(config_file) = config_file {
            if config_file.is_empty() {
                return Err("No filename provided for config file.".to_string());
            }
            config_file.to_string()
        } else {
            DEFAULT_CONFIG_FILE_NAME.to_string()
        };

        PlayerId::get_config_path_with_fallback(&config_file)
    }

    pub fn convert_file_format(config_file: Option<&String>, new_format: bool) -> Result<(), String> {
        let issues_found = Self::verify_signatures(config_file)?;
        if issues_found {
            return Err("Issues found in config file.".to_string());
        }

        eprintln!("\r\nWriting signatures in {} format.\r", if new_format { "new" } else { "old" });

        let config_path = PlayerId::get_config_path(config_file)?;
        eprintln!("Writing config file to: {}\r", config_path.display());

        let lines = Self::read_text_file(&config_path)?;
        let signature_ids = Signature::read_config_lines(&lines, None)?;

        let output_string = Self::convert_ids_to_string(signature_ids, new_format);

        fs::write(&config_path, output_string)
            .map_err(|e| format!("Error writing config file: {e}"))?;

        eprintln!("Done!\r");
        Ok(())
    }

    fn convert_ids_to_string(signature_ids: Vec<SignatureConfig>, new_format: bool) -> String {
        let mut output_strings = vec![];
        let mut previous_signature_name = "".to_string();

        for signature in signature_ids {
            if signature.signature_name.ne(&previous_signature_name) {
                if !output_strings.is_empty() && !signature.signature_name.starts_with('(') {
                    output_strings.push("\r\n".to_string() + &signature.signature_name);
                } else {
                    output_strings.push(signature.signature_name.to_owned());
                }
            }

            previous_signature_name = signature.signature_name;
            let mut output_string = "".to_string();

            for bndm_config in signature.bndm_configs {
                if !output_string.is_empty() {
                    output_string += if new_format { " && " } else { " AND " };
                }

                output_string += &bndm_config.pattern.iter()
                    .map(|byte| {
                        if let Some(wildcard) = bndm_config.wildcard && *byte == wildcard {
                            return "??".to_string();
                        }
                        format!("{byte:02X}")
                    })
                    .collect::<Vec<String>>()
                    .join(" ");
            }

            if !new_format {
                output_string += " END";
            }

            output_strings.push(output_string);
        }
        output_strings.join("\r\n") + "\r\n"
    }

    pub fn verify_signatures(config_file: Option<&String>) -> Result<bool, String> {
        eprintln!("Checking signatures...\r");

        let config_path = PlayerId::get_config_path(config_file)?;
        eprintln!("Verify config file: {}\r\n\r", config_path.display());

        let lines = Self::read_text_file(&config_path)?;
        let issues_found = validate::verify_config_file(&lines)?;

        if !issues_found {
            eprintln!("No issues found in configuration.\r");
        }
        Ok(issues_found)
    }

    pub fn verify_signature_info(config_file: Option<&String>) -> Result<bool, String> {
        eprintln!("\r\nChecking info file...\r");

        let config_path = PlayerId::get_config_path(config_file)?;
        let lines = Self::read_text_file(&config_path)?;
        let signature_ids = Signature::read_config_lines(&lines, None)?;

        let config_path_string = config_path.display().to_string().replace(".cfg", ".nfo");
        let config_path = PlayerId::get_config_path_with_fallback(&config_path_string);

        if let Ok(config_path) = config_path {
            eprintln!("Verify info file: {}\r\n\r", config_path.display());

            let lines = Self::read_text_file(&config_path)?;
            let issues_found = validate::verify_info_file(&lines, &signature_ids)?;

            if !issues_found {
                eprintln!("No issues found in info file.\r");
            }
            Ok(issues_found)
        } else {
            eprintln!("\r\nNo info file found: {config_path_string}\r");
            Ok(true)
        }
    }

    fn get_data_offset(filename: &str, data: &[u8]) -> usize {
        if sid_file::is_sid_file(data) {
            sid_file::get_data_offset(data)
        } else if filename.ends_with(".prg") { 2 } else { 0 }
    }

    fn get_config_path_with_fallback(filename: &str) -> Result<PathBuf, String> {
        let file = Path::new(filename);
        if file.exists() {
            return Ok(file.to_path_buf())
        }

        let default_config_file_path = env::current_exe().map_err(|_| "Could not determine executable location")?
            .parent().ok_or("Could not determine executable directory")?
            .join(filename);

        if default_config_file_path.exists() {
            Ok(default_config_file_path)
        } else {
            Err(format!("File doesn't exist: {filename}"))
        }
    }

    fn read_text_file(config_path: &PathBuf) -> Result<Vec<String>, String> {
        let lines = Self::read_lines(config_path);
        lines.map_err(|_| format!("Error reading file: {}", config_path.display()))
    }

    fn read_lines(filename: &PathBuf) -> io::Result<Vec<String>> {
        let file = File::open(filename)?;
        let lines = BufReader::new(
            DecodeReaderBytesBuilder::new()
                .encoding(Some(WINDOWS_1252))
                .build(file)).lines();
        Ok(lines.map_while(Result::ok).collect())
    }

    fn get_first_few_lines_from_file(file: File) -> Vec<String> {
        let reader = BufReader::new(
            DecodeReaderBytesBuilder::new()
                .encoding(Some(WINDOWS_1252))
                .build(file));
        let chunk = reader.take(1000);
        chunk.lines().map_while(Result::ok).collect()
    }
}
