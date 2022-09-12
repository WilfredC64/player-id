// Copyright (C) 2019 - 2022 Wilfred Bos
// Licensed under the MIT license. See the LICENSE file for the terms and conditions.

#![allow(dead_code)]

#[path = "./utils/sid_file.rs"] mod sid_file;
mod bndm;
mod sidid;

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use sidid::{SidId};
pub use sidid::{SidIdHolder, SidInfo};

pub struct PlayerId {
}

impl PlayerId {
    pub fn find_players_in_buffer(buffer: &[u8], sid_ids: &Vec<SidIdHolder>, scan_for_multiple: bool) -> Vec<(String, Vec<usize>)> {
        SidId::find(buffer, 0, sid_ids, scan_for_multiple)
    }

    pub fn find_players_in_file(filename: &str, sid_ids: &Vec<SidIdHolder>, scan_for_multiple: bool) -> Vec<(String, Vec<usize>)> {
        let sid_data = Self::read_file(filename);
        if let Ok(sid_data) = sid_data {
            let start_offset = if sid_file::is_sid_file(&sid_data) {
                sid_file::get_data_offset(&sid_data)
            } else {
                0
            };

            SidId::find(&sid_data, start_offset, sid_ids, scan_for_multiple)
        } else {
            vec![]
        }
    }

    pub fn find_player_info(sid_infos: &[SidInfo], player_name: &str) -> Option<SidInfo> {
        SidId::find_player_info(sid_infos, player_name)
    }

    pub fn get_config_path(filename: Option<String>) -> Result<PathBuf, String> {
        SidId::get_config_path(filename)
    }

    pub fn is_config_file(filename: &str) -> bool {
        let path = SidId::get_config_path(Some(filename.to_string()));
        if let Ok(path) = path {
            SidId::is_config_file(&path)
        } else {
            false
        }
    }

    pub fn load_config_file(file_path: &PathBuf, player_name: Option<String>) -> Result<Vec<SidIdHolder>, String> {
        SidId::read_sidid_config(file_path, player_name)
    }

    pub fn load_info_file(file_path: &PathBuf) -> Result<Vec<SidInfo>, String> {
        SidId::read_sidid_info(file_path)
    }

    pub fn verify_config_file(file_path: &PathBuf) -> Result<bool, String> {
        SidId::verify_sidid_config(file_path)
    }

    pub fn verify_info_file(file_path: &PathBuf, sidids: &[SidIdHolder]) -> Result<bool, String> {
        SidId::verify_sidid_info(file_path, sidids)
    }

    fn read_file(filename: &str) -> std::io::Result<Vec<u8>> {
        let mut file = File::open(filename)?;
        let mut data = vec![];
        file.read_to_end(&mut data)?;
        Ok(data)
    }
}
