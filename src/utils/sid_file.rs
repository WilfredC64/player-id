// Copyright (C) 2020 - 2023 Wilfred Bos
// Licensed under the MIT license. See the LICENSE file for the terms and conditions.

const MIN_SID_HEADER_SIZE: usize = 0x76;
const DATA_OFFSET_OFFSET: usize = 0x06;
const LOAD_ADDRESS_OFFSET: usize = 0x08;
const LOAD_ADDRESS_SIZE: usize = 2;

pub fn is_sid_file(source: &[u8]) -> bool {
    source.len() >= MIN_SID_HEADER_SIZE && matches!(&source[0..4], b"RSID" | b"PSID")
}

pub fn get_data_offset(source: &[u8]) -> usize {
    let mut data_offset = u16::from_be_bytes([source[DATA_OFFSET_OFFSET], source[DATA_OFFSET_OFFSET + 1]]) as usize;
    if data_offset >= MIN_SID_HEADER_SIZE && data_offset <= source.len() {
        if source[LOAD_ADDRESS_OFFSET] == 0 && source[LOAD_ADDRESS_OFFSET + 1] == 0 {
            data_offset += LOAD_ADDRESS_SIZE;
        }
        return data_offset
    }
    0
}
