// Copyright (C) 2019 - 2022 Wilfred Bos
// Licensed under the MIT license. See the LICENSE file for the terms and conditions.

#![allow(dead_code)]

#[path = "./utils/sid_file.rs"] mod sid_file;
mod bndm;
mod signature;

use std::env;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use signature::{Signature};
pub use signature::{SignatureHolder, SignatureInfo, SignatureMatch};

const DEFAULT_CONFIG_FILE_NAME: &str = "sidid.cfg";

pub struct PlayerId {}

impl PlayerId {
    pub fn find_players_in_buffer(buffer: &[u8], signature_ids: &Vec<SignatureHolder>, scan_for_multiple: bool) -> Vec<SignatureMatch> {
        Signature::find_signatures(buffer, 0, signature_ids, scan_for_multiple)
    }

    pub fn find_players_in_file(filename: &str, sid_ids: &Vec<SignatureHolder>, scan_for_multiple: bool) -> Vec<SignatureMatch> {
        if let Ok(sid_data) = Self::read_file(filename) {
            let start_offset = if sid_file::is_sid_file(&sid_data) {
                sid_file::get_data_offset(&sid_data)
            } else {
                0
            };

            Signature::find_signatures(&sid_data, start_offset, sid_ids, scan_for_multiple)
        } else {
            vec![]
        }
    }

    pub fn find_player_info(signature_infos: &[SignatureInfo], player_name: &str) -> Option<SignatureInfo> {
        Signature::find_signature_info(signature_infos, player_name)
    }

    pub fn display_player_info(config_file: Option<String>, player_name: &str) -> Result<(), String> {
        let player_infos = PlayerId::load_info_file(config_file)?;

        if let Some(player_info) = Signature::find_signature_info(&player_infos, player_name) {
            println!("Player info:\n\n{}\n{}", player_info.signature_name, player_info.info_lines.join("\n"));
        } else {
            println!("No info found for player ID: {}", &player_name);
        }
        Ok(())
    }

    pub fn is_config_file(filename: &str) -> bool {
        if let Ok(path) = PlayerId::get_config_path_with_fallback(filename) {
            Signature::is_config_file(&path)
        } else {
            false
        }
    }

    pub fn load_config_file(config_file: Option<String>, player_name: Option<String>) -> Result<Vec<SignatureHolder>, String> {
        let config_path = PlayerId::get_config_path(config_file)?;
        println!("Using config file: {}\n", config_path.display());

        let sid_ids = Signature::read_config_file(&config_path, player_name)?;
        if sid_ids.is_empty() {
            return Err("No signature defined.".to_string());
        }
        Ok(sid_ids)
    }

    pub fn load_info_file(config_file: Option<String>) -> Result<Vec<SignatureInfo>, String> {
        let config_path_string = PlayerId::get_config_path(config_file)?.display().to_string().replace(".cfg", ".nfo");
        let config_path = PlayerId::get_config_path_with_fallback(&config_path_string)?;
        println!("Using info file: {}\n", config_path.display());

        let sid_infos = Signature::read_info_file(&config_path)?;
        if sid_infos.is_empty() {
            return Err("No signature defined.".to_string());
        }
        Ok(sid_infos)
    }

    pub fn verify_signatures(config_file: Option<String>) -> Result<bool, String> {
        println!("Checking signatures...");

        let config_path = PlayerId::get_config_path(config_file)?;
        println!("Verify config file: {}\n", config_path.display());

        let issues_found = Signature::verify_config_file(&config_path)?;

        if !issues_found {
            println!("No issues found in configuration.");
        }
        Ok(issues_found)
    }

    pub fn verify_signature_info(config_file: Option<String>) -> Result<bool, String> {
        println!("\nChecking info file...");

        let config_path = PlayerId::get_config_path(config_file)?;
        let sid_ids = Signature::read_config_file(&config_path, None)?;

        let config_path_string = config_path.display().to_string().replace(".cfg", ".nfo");
        let config_path = PlayerId::get_config_path_with_fallback(&config_path_string);

        if let Ok(config_path) = config_path {
            println!("Verify info file: {}\n", config_path.display());

            let issues_found = Signature::verify_info_file(&config_path, &sid_ids)?;

            if !issues_found {
                println!("No issues found in info file.");
            }
            Ok(issues_found)
        } else {
            println!("\nNo info file found: {}", config_path_string);
            Ok(true)
        }
    }

    fn get_config_path_with_fallback(filename: &str) -> Result<PathBuf, String> {
        let file = Path::new(filename).to_path_buf();
        if file.exists() {
            return Ok(file)
        } else {
            let default_config_file_path = env::current_exe().unwrap().parent().unwrap().join(filename);
            if default_config_file_path.exists() {
                return Ok(default_config_file_path)
            }
        }
        Err(format!("File doesn't exist: {}", filename))
    }

    fn get_config_path(config_file: Option<String>) -> Result<PathBuf, String> {
        let config_file = if let Some(config_file) = config_file {
            if config_file.is_empty() {
                return Err("Invalid config filename. No space allowed after -f switch.".to_string());
            }
            config_file
        } else {
            let config_file = env::var("SIDIDCFG");
            if let Ok(config_file) = config_file {
                config_file
            } else {
                DEFAULT_CONFIG_FILE_NAME.to_string()
            }
        };

        let config_path = PlayerId::get_config_path_with_fallback(&config_file)?;
        Ok(config_path)
    }

    fn read_file(filename: &str) -> std::io::Result<Vec<u8>> {
        let mut file = File::open(filename)?;
        let mut data = vec![];
        file.read_to_end(&mut data)?;
        Ok(data)
    }
}
