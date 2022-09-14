// Copyright (C) 2019 - 2022 Wilfred Bos
// Licensed under the MIT license. See the LICENSE file for the terms and conditions.

#![allow(dead_code)]

#[path = "./utils/sid_file.rs"] mod sid_file;
mod bndm;
mod signature;

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use signature::{Signature};
pub use signature::{SignatureHolder, SignatureInfo};

pub struct PlayerId {
}

impl PlayerId {
    pub fn find_players_in_buffer(buffer: &[u8], sid_ids: &Vec<SignatureHolder>, scan_for_multiple: bool) -> Vec<(String, Vec<usize>)> {
        Signature::find_signatures(buffer, 0, sid_ids, scan_for_multiple)
    }

    pub fn find_players_in_file(filename: &str, sid_ids: &Vec<SignatureHolder>, scan_for_multiple: bool) -> Vec<(String, Vec<usize>)> {
        let sid_data = Self::read_file(filename);
        if let Ok(sid_data) = sid_data {
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

    pub fn find_player_info(sid_infos: &[SignatureInfo], player_name: &str) -> Option<SignatureInfo> {
        Signature::find_signature_info(sid_infos, player_name)
    }

    pub fn get_config_path(filename: Option<String>) -> Result<PathBuf, String> {
        Signature::get_config_path(filename)
    }

    pub fn is_config_file(filename: &str) -> bool {
        let path = Signature::get_config_path(Some(filename.to_string()));
        if let Ok(path) = path {
            Signature::is_config_file(&path)
        } else {
            false
        }
    }

    pub fn read_config_file(file_path: &PathBuf, player_name: Option<String>) -> Result<Vec<SignatureHolder>, String> {
        Signature::read_config_file(file_path, player_name)
    }

    pub fn read_info_file(file_path: &PathBuf) -> Result<Vec<SignatureInfo>, String> {
        Signature::read_info_file(file_path)
    }

    pub fn verify_config_file(file_path: &PathBuf) -> Result<bool, String> {
        Signature::verify_config_file(file_path)
    }

    pub fn verify_info_file(file_path: &PathBuf, sidids: &[SignatureHolder]) -> Result<bool, String> {
        Signature::verify_info_file(file_path, sidids)
    }

    fn read_file(filename: &str) -> std::io::Result<Vec<u8>> {
        let mut file = File::open(filename)?;
        let mut data = vec![];
        file.read_to_end(&mut data)?;
        Ok(data)
    }
}
