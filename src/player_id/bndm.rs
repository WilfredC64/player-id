// Copyright (C) 2019 - 2023 Wilfred Bos
// Licensed under the MIT license. See the LICENSE file for the terms and conditions.

use std::cmp::min;

const MASKS_TABLE_SIZE: usize = 256;
const WORD_SIZE_IN_BITS: usize = usize::BITS as usize;

pub struct BndmConfig {
    pub masks: [usize; MASKS_TABLE_SIZE],
    pub wildcard: Option<u8>,
    pub pattern: Vec<u8>
}

impl BndmConfig {
    pub fn new(search_pattern: &[u8], wildcard: Option<u8>) -> BndmConfig {
        let len = get_pattern_length_within_cpu_word(search_pattern.len());

        BndmConfig {
            masks: generate_masks(&search_pattern[..len], wildcard),
            wildcard,
            pattern: search_pattern.to_owned()
        }
    }
}

pub fn find_pattern(source: &[u8], config: &BndmConfig) -> Option<usize> {
    match config.pattern.len() {
        0 => None,
        1 => config.wildcard
                .map_or(false, |w| w == config.pattern[0]).then_some(0)
                .or_else(|| source.iter().position(|&s| s == config.pattern[0])),
        _ => find_pattern_bndm(source, config)
    }
}

fn find_pattern_bndm(source: &[u8], config: &BndmConfig) -> Option<usize> {
    if config.pattern.len() > source.len() {
        return None;
    }

    let len = get_pattern_length_within_cpu_word(config.pattern.len()) - 1;
    let end = source.len() - config.pattern.len();
    let df = 1 << len;
    let mut i = 0;

    while i <= end {
        let mut j = len;
        let mut last = len;

        let mut d = get_mask(source, config, i + j);
        d = (d << 1) & get_mask(source, config, i + j - 1);
        while d != 0 {
            j -= 1;
            if d & df != 0 {
                if j == 0 {
                    if find_remaining(source, config, i + WORD_SIZE_IN_BITS) {
                        return Some(i);
                    }
                    j += 1;
                }
                last = j;
            }
            d = (d << 1) & get_mask(source, config, i + j - 1);
        }

        i += last;
    }
    None
}

fn get_mask(source: &[u8], config: &BndmConfig, index: usize) -> usize {
    unsafe {
        *config.masks.get_unchecked(*source.get_unchecked(index) as usize)
    }
}

fn find_remaining(source: &[u8], config: &BndmConfig, start_index: usize) -> bool {
    config.pattern.iter().skip(WORD_SIZE_IN_BITS).enumerate().all(|(index, &pattern_byte)| unsafe {
        *source.get_unchecked(start_index + index) == pattern_byte || config.wildcard.map_or(false, |w| pattern_byte == w)
    })
}

fn get_pattern_length_within_cpu_word(search_pattern_length: usize) -> usize {
    min(search_pattern_length, WORD_SIZE_IN_BITS)
}

fn calculate_wildcard_mask(search_pattern: &[u8], wildcard: Option<u8>) -> usize {
    wildcard.map_or(0, |wildcard| search_pattern.iter()
        .fold(0, |mask, &pattern_byte| (mask << 1) | (pattern_byte == wildcard) as usize))
}

fn generate_masks(search_pattern: &[u8], wildcard: Option<u8>) -> [usize; MASKS_TABLE_SIZE] {
    let default_mask = calculate_wildcard_mask(search_pattern, wildcard);
    let mut masks = [default_mask; MASKS_TABLE_SIZE];

    search_pattern.iter().rev().enumerate()
        .for_each(|(i, &pattern_byte)| masks[pattern_byte as usize] |= 1 << i);

    masks
}

#[cfg(test)]
#[path = "./bndm_test.rs"]
mod bndm_test;
