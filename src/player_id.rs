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
pub use signature::{SignatureConfig, SignatureInfo, SignatureMatch};

const DEFAULT_CONFIG_FILE_NAME: &str = "sidid.cfg";

pub struct PlayerId {}

impl PlayerId {
    pub fn find_players_in_buffer(buffer: &[u8], signature_ids: &Vec<SignatureConfig>, scan_for_multiple: bool) -> Vec<SignatureMatch> {
        Signature::find_signatures(buffer, 0, signature_ids, scan_for_multiple)
    }

    pub fn find_players_in_file(filename: &str, sid_ids: &Vec<SignatureConfig>, scan_for_multiple: bool) -> Vec<SignatureMatch> {
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

    pub fn is_config_file(filename: &str) -> bool {
        if let Ok(path) = PlayerId::get_config_path_with_fallback(filename) {
            Signature::is_config_file(&path)
        } else {
            false
        }
    }

    pub fn load_config_file(config_path: &PathBuf, player_name: Option<&String>) -> Result<Vec<SignatureConfig>, String> {
        let sid_ids = Signature::read_config_file(config_path, player_name)?;
        if sid_ids.is_empty() {
            return if player_name.is_none() {
                Err("No signature defined.".to_string())
            } else {
                Err(format!("No signature found with name: {}", player_name.unwrap()))
            }
        }
        Ok(sid_ids)
    }

    pub fn load_info_file(config_path: &PathBuf) -> Result<Vec<SignatureInfo>, String> {
        let sid_infos = Signature::read_info_file(config_path)?;
        if sid_infos.is_empty() {
            return Err("No info sections defined.".to_string());
        }
        Ok(sid_infos)
    }

    pub fn get_info_file_path(config_file: Option<&String>) -> Result<PathBuf, String> {
        let config_path_string = PlayerId::get_config_path(config_file)?.display().to_string().replace(".cfg", ".nfo");
        PlayerId::get_config_path_with_fallback(&config_path_string)
    }

    pub fn get_config_path(config_file: Option<&String>) -> Result<PathBuf, String> {
        let config_file = if let Some(config_file) = config_file {
            if config_file.is_empty() {
                return Err("No filename provided for config file.".to_string());
            }
            config_file.to_string()
        } else {
            env::var("SIDIDCFG").unwrap_or_else(|_| DEFAULT_CONFIG_FILE_NAME.to_string())
        };

        PlayerId::get_config_path_with_fallback(&config_file)
    }

    pub fn verify_signatures(config_file: Option<&String>) -> Result<bool, String> {
        eprintln!("Checking signatures...\r");

        let config_path = PlayerId::get_config_path(config_file)?;
        eprintln!("Verify config file: {}\r\n\r", config_path.display());

        let issues_found = Signature::verify_config_file(&config_path)?;

        if !issues_found {
            eprintln!("No issues found in configuration.\r");
        }
        Ok(issues_found)
    }

    pub fn verify_signature_info(config_file: Option<&String>) -> Result<bool, String> {
        eprintln!("\r\nChecking info file...\r");

        let config_path = PlayerId::get_config_path(config_file)?;
        let sid_ids = Signature::read_config_file(&config_path, None)?;

        let config_path_string = config_path.display().to_string().replace(".cfg", ".nfo");
        let config_path = PlayerId::get_config_path_with_fallback(&config_path_string);

        if let Ok(config_path) = config_path {
            eprintln!("Verify info file: {}\r\n\r", config_path.display());

            let issues_found = Signature::verify_info_file(&config_path, &sid_ids)?;

            if !issues_found {
                eprintln!("No issues found in info file.\r");
            }
            Ok(issues_found)
        } else {
            eprintln!("\r\nNo info file found: {}\r", config_path_string);
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

    fn read_file(filename: &str) -> std::io::Result<Vec<u8>> {
        let mut data = vec![];
        File::open(filename)?.read_to_end(&mut data)?;
        Ok(data)
    }
}
