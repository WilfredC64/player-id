// Copyright (C) 2019 - 2022 Wilfred Bos
// Licensed under the MIT license. See the LICENSE file for the terms and conditions.

use std::{cmp, mem};

const MASKS_TABLE_SIZE: usize = 256;
const WORD_SIZE_IN_BITS: usize = mem::size_of::<usize>() * 8;

pub struct BndmConfig {
    pub masks: [usize; MASKS_TABLE_SIZE],
    pub wildcard: Option<u8>,
    pub pattern: Vec<u8>
}

impl BndmConfig {
    pub fn new(search_pattern: &[u8], wildcard: Option<u8>) -> BndmConfig {
        let len = get_pattern_length_within_cpu_word(search_pattern);

        let wildcard_mask = if let Some(wildcard) = wildcard {
            calculate_wildcard_mask(&search_pattern[..len], wildcard)
        } else {
            0
        };

        BndmConfig {
            masks: generate_masks(&search_pattern[..len], wildcard_mask),
            wildcard,
            pattern: search_pattern.to_owned()
        }
    }
}

pub fn find_pattern(source: &[u8], config: &BndmConfig) -> Option<usize> {
    match config.pattern.len() {
        0 => None,
        1 => source.iter().position(|&x| x == config.pattern[0]),
        x if x > source.len() => None,
        x if x > WORD_SIZE_IN_BITS => find_large_pattern(source, config),
        _ => find_small_pattern(source, config)
    }
}

/// finds patterns up to CPU word size
fn find_small_pattern(source: &[u8], config: &BndmConfig) -> Option<usize> {
    let len = config.pattern.len() - 1;
    let end = source.len() - len;
    let masks = &config.masks;
    let mut i = 0;

    while i < end {
        let mut j = len;

        let mut d = masks[source[i + j] as usize];
        d = (d << 1) & masks[source[i + j - 1] as usize];
        while d != 0 {
            j -= 1;
            if j == 0 {
                return Some(i);
            }
            d = (d << 1) & masks[source[i + j - 1] as usize];
        }

        i += j;
    }
    None
}

/// finds patterns larger than CPU word size
fn find_large_pattern(source: &[u8], config: &BndmConfig) -> Option<usize> {
    let len = config.pattern.len() - 1;
    let end = source.len() - len;
    let masks = &config.masks;
    let pattern = &config.pattern;
    let wildcard = &config.wildcard;
    let mut i = 0;

    while i < end {
        let mut j = WORD_SIZE_IN_BITS - 1;

        let mut d = masks[source[i + j] as usize];
        d = (d << 1) & masks[source[i + j - 1] as usize];
        while d != 0 {
            j -= 1;
            if j == 0 {
                if find_remaining(&source[i + WORD_SIZE_IN_BITS..], &pattern[WORD_SIZE_IN_BITS..], wildcard) {
                    return Some(i);
                }
                j = 1;
            }
            d = (d << 1) & masks[source[i + j - 1] as usize];
        }
        i += j;
    }
    None
}

fn find_remaining(source: &[u8], search_pattern: &[u8], wildcard: &Option<u8>) -> bool {
    if let Some(wildcard) = wildcard {
        search_pattern.iter().enumerate().all(|(index, pattern_byte)| source[index] == *pattern_byte || *pattern_byte == *wildcard)
    } else {
        search_pattern.iter().enumerate().all(|(index, pattern_byte)| source[index] == *pattern_byte)
    }
}

fn get_pattern_length_within_cpu_word(search_pattern: &[u8]) -> usize {
    cmp::min(search_pattern.len(), WORD_SIZE_IN_BITS)
}

fn calculate_wildcard_mask(search_pattern: &[u8], wildcard: u8) -> usize {
    let len = search_pattern.len();
    let mut mask = 0;
    if len > 0 {
        let bit_select = 1 << (len - 1);

        for (i, pattern_byte) in search_pattern.iter().enumerate() {
            if *pattern_byte == wildcard {
                mask |= bit_select >> i;
            }
        }
    }
    mask
}

fn generate_masks(search_pattern: &[u8], default_mask: usize) -> [usize; MASKS_TABLE_SIZE] {
    let len = search_pattern.len();
    let mut masks = [default_mask; MASKS_TABLE_SIZE];
    if len > 0 {
        let bit_select = 1 << (len - 1);

        search_pattern.iter().enumerate().for_each(|(i, pattern_byte)| masks[*pattern_byte as usize] |= bit_select >> i);
    }
    masks
}

#[cfg(test)]
#[path = "./bndm_test.rs"]
mod bndm_test;
