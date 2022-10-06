// Copyright (C) 2020 - 2022 Wilfred Bos
// Licensed under the MIT license. See the LICENSE file for the terms and conditions.

pub const MIN_SID_HEADER_SIZE: usize = 0x76;

const DATA_OFFSET_OFFSET: usize = 0x06;
const LOAD_ADDRESS_OFFSET: usize = 0x08;

pub fn is_sid_file(source: &[u8]) -> bool {
    source.len() >= MIN_SID_HEADER_SIZE &&
        (source[0] == b'R' || source[0] == b'P') && source[1] == b'S' && source[2] == b'I' && source[3] == b'D'
}

pub fn get_data_offset(source: &[u8]) -> usize {
    if source.len() >= MIN_SID_HEADER_SIZE {
        let header_size = ((source[DATA_OFFSET_OFFSET] as usize) << 8) + source[DATA_OFFSET_OFFSET + 1] as usize;
        if header_size >= MIN_SID_HEADER_SIZE && header_size <= source.len() {
            let has_load_address_in_data = source[LOAD_ADDRESS_OFFSET] == 0 && source[LOAD_ADDRESS_OFFSET + 1] == 0;
            return if has_load_address_in_data {
                header_size + 2
            } else {
                header_size
            }
        }
    }
    0
}
