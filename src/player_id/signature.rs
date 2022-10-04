// Copyright (C) 2019 - 2022 Wilfred Bos
// Licensed under the MIT license. See the LICENSE file for the terms and conditions.

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Lines, Read};
use std::path::PathBuf;

use encoding_rs::WINDOWS_1252;
use encoding_rs_io::{DecodeReaderBytesBuilder, DecodeReaderBytes};
use str_utils::*;
use substring::Substring;

use super::bndm::{BndmConfig, find_pattern};

const CMD_WILDCARD: u16 = 0x100;

type LinesDecoded = Lines<BufReader<DecodeReaderBytes<File, Vec<u8>>>>;

pub struct SignatureConfig {
    pub bndm_configs: Vec<BndmConfig>,
    pub signature_name: String
}

pub struct SignatureMatch {
    pub signature_name: String,
    pub indexes: Vec<usize>,
}

#[derive(Clone)]
pub struct SignatureInfo {
    pub info_lines: Vec<String>,
    pub signature_name: String
}

pub struct Signature {}

impl Signature {
    pub fn find_signatures(source: &[u8], start_offset: usize, signatures: &Vec<SignatureConfig>, scan_for_multiple: bool) -> Vec<SignatureMatch> {
        let mut matches = vec![];

        for signature in signatures {
            let mut indexes = vec![];
            let mut last_index = start_offset;
            let mut index_found = true;

            for config in  &signature.bndm_configs {
                let index = find_pattern(&source[last_index..], config);

                if let Some(index) = index {
                    indexes.push(last_index + index);
                    last_index += index + config.pattern.len();
                } else {
                    index_found = false;
                    break;
                }
            }

            if index_found {
                matches.push(SignatureMatch { signature_name: signature.signature_name.to_owned(), indexes });

                if !scan_for_multiple {
                    break;
                }
            }
        }
        matches.dedup_by(|a, b| a.signature_name.eq(&b.signature_name));
        matches
    }

    pub fn find_signature_info(signature_infos: &[SignatureInfo], signature_name: &str) -> Option<SignatureInfo> {
        signature_infos.iter().find(|info| info.signature_name.eq_ignore_ascii_case(signature_name)).cloned()
    }

    pub fn read_config_file(file_path: &PathBuf, signature_name_to_filter: Option<&String>) -> Result<Vec<SignatureConfig>, String> {
        if !Self::is_config_file(file_path) {
            return Err("Not a config file.".to_string());
        }

        let mut signatures = vec![];

        if let Ok(lines) = Self::read_lines(file_path) {
            let mut signature_name = "".to_string();

            let mut signature_lines = vec![];
            for line in lines.flatten() {
                let signature_text = line.trim();

                if Self::is_signature_min_length(signature_text) {
                    if Self::is_signature_name(signature_text) {
                        Self::process_multi_signatures(signature_name_to_filter, &mut signatures, &signature_name, &mut signature_lines);
                        signature_name = signature_text.to_string();
                    } else {
                        signature_lines.push(signature_text.to_string());
                        if signature_text.ends_with_ignore_ascii_case_with_uppercase("END") {
                            Self::process_single_signature(signature_name_to_filter, &mut signatures, &signature_name, &mut signature_lines);
                        }
                    }
                } else {
                    Self::process_multi_signatures(signature_name_to_filter, &mut signatures, &signature_name, &mut signature_lines);
                    signature_name = "".to_string();
                }
            }
            Self::process_multi_signatures(signature_name_to_filter, &mut signatures, &signature_name, &mut signature_lines);
        }
        Ok(signatures)
    }

    pub fn read_info_file(file_path: &PathBuf) -> Result<Vec<SignatureInfo>, String> {
        if !Self::is_info_file(file_path) {
            return Err("Not an info file.".to_string());
        }

        let mut signature_infos = vec![];
        if let Ok(lines) = Self::read_lines(file_path) {
            let mut signature_name = "".to_string();

            let mut info_lines = vec![];
            for line in lines.flatten() {
                if Self::is_signature_min_length(&line) {
                    if Self::is_info_tag(&line) {
                        info_lines.push(line);
                    } else if Self::is_signature_name(&line) {
                        if !signature_name.is_empty() {
                            signature_infos.push(SignatureInfo { signature_name, info_lines: info_lines.to_owned() });
                        }
                        info_lines.clear();
                        signature_name = line;
                    } else {
                        signature_infos.push(SignatureInfo { signature_name, info_lines: info_lines.to_owned() });
                        info_lines.clear();
                        signature_name = "".to_string();
                    }
                } else {
                    signature_infos.push(SignatureInfo { signature_name, info_lines: info_lines.to_owned() });
                    info_lines.clear();
                    signature_name = "".to_string();
                }
            }

            if !info_lines.is_empty() {
                signature_infos.push(SignatureInfo { signature_name, info_lines: info_lines.to_owned() });
            }
        }
        Ok(signature_infos)
    }

    pub fn is_config_file(filename: &PathBuf) -> bool {
        if let Ok(file) = File::open(filename) {
            let lines = Self::get_first_few_lines_from_file(file);
            let mut lines_iter = lines.iter();

            while let Some(line) = lines_iter.next() {
                if line.trim().is_empty() {
                    continue;
                }
                if Self::is_signature_min_length(line) && Self::is_signature_name(line) {
                    if let Some(line) = lines_iter.next() {
                        return Self::is_signature_min_length(line) && !Self::is_signature_name(line);
                    }
                }
                break;
            }
        }
        false
    }

    fn is_info_file(filename: &PathBuf) -> bool {
        if let Ok(file) = File::open(filename) {
            let lines = Self::get_first_few_lines_from_file(file);
            let mut lines_iter = lines.iter();

            while let Some(line) = lines_iter.next() {
                if line.trim().is_empty() {
                    continue;
                }
                if Self::is_signature_min_length(line) && Self::is_signature_name(line) && !Self::is_info_tag(line) {
                    if let Some(line) = lines_iter.next() {
                        return Self::is_signature_min_length(line) && Self::is_info_tag(line);
                    }
                }
                break;
            }
        }
        false
    }

    fn get_first_few_lines_from_file(file: File) -> Vec<String> {
        let reader = BufReader::new(
            DecodeReaderBytesBuilder::new()
                .encoding(Some(WINDOWS_1252))
                .build(file));
        let chunk = reader.take(1000);
        chunk.lines()
            .map(|line| line.unwrap_or_default())
            .collect::<Vec<_>>()
    }

    fn process_multi_signatures(signature_name_to_filter: Option<&String>, signatures: &mut Vec<SignatureConfig>, signature_name: &str, signature_lines: &mut Vec<String>) {
        for signature_line in &*signature_lines {
            Self::process_signature_line(signature_name_to_filter, signatures, signature_name, signature_line);
        }
        signature_lines.clear();
    }

    fn process_single_signature(signature_name_to_filter: Option<&String>, signatures: &mut Vec<SignatureConfig>, signature_name: &str, signature_lines: &mut Vec<String>) {
        Self::process_signature_line(signature_name_to_filter, signatures, signature_name, &signature_lines.join(" "));
        signature_lines.clear();
    }

    fn process_signature_line(signature_name_to_filter: Option<&String>, signatures: &mut Vec<SignatureConfig>, signature_name: &str, signature_text: &str) {
        let signature = Self::process_signature_value(signature_name, signature_text);
        if signature_name_to_filter.is_none() || signature_name_to_filter.unwrap().eq_ignore_ascii_case(signature_name) {
            signatures.push(signature);
        }
    }

    fn process_signature_value(signature_name: &str, signature_text: &str) -> SignatureConfig {
        let mut signature = vec![];
        let mut bndm_configs = vec![];

        for word in signature_text.to_ascii_uppercase().split_ascii_whitespace() {
            if word.len() >= 2 {
                match word {
                    "??" => signature.push(CMD_WILDCARD),
                    "AND" | "&&" | "END" => {
                        Self::add_signature(&signature, &mut bndm_configs);
                        signature.clear();
                    },
                    _ => signature.push(Self::convert_hex_to_bin(word.substring(0, 2)))
                }
            }
        }

        if !signature.is_empty() {
            Self::add_signature(&signature, &mut bndm_configs);
        }

        SignatureConfig { signature_name: signature_name.to_string(), bndm_configs }
    }

    fn is_signature_min_length(signature_text_line: &str) -> bool {
        signature_text_line.len() >= 2
    }

    fn is_info_tag(signature_text_line: &str) -> bool {
        if signature_text_line.len() >= 11 {
            let signature_first_11chars = &signature_text_line.substring(0, 11).as_bytes();
            return (signature_first_11chars[9] == b':' && signature_first_11chars[10] == b' ') ||
                signature_first_11chars.eq(b"           ");
        }
        false
    }

    fn is_signature_name(signature_text_line: &str) -> bool {
        if signature_text_line.len() >= 3 {
            let signature_first_3chars = &signature_text_line.substring(0, 3).as_bytes();
            return signature_first_3chars[2] != b' ' &&
                (signature_text_line.len() > 3 || signature_first_3chars.eq_ignore_ascii_case_with_uppercase_multiple(&[b"END", b"AND"]).is_none());
        }
        false
    }

    fn add_signature(signature: &[u16], bndm_configs: &mut Vec<BndmConfig>) {
        let (wildcard_used, calculated_wildcard) = Self::calculate_wildcard(signature);

        if !wildcard_used || calculated_wildcard.is_some() {
            let mut new_signature = vec![];

            for value in signature {
                if *value < 0x100 {
                    new_signature.push(*value as u8);
                } else if *value == CMD_WILDCARD {
                    new_signature.push(calculated_wildcard.unwrap());
                }
            }

            bndm_configs.push(BndmConfig::new(&new_signature, calculated_wildcard));
        }
    }

    fn calculate_wildcard(signature: &[u16]) -> (bool, Option<u8>) {
        const SIGNATURE_MAX_VALUE: u16 = 0x100; // only bytes 0x00 - 0xFF are used and 0x100 for the wildcard
        let mut bytes_used = [false; SIGNATURE_MAX_VALUE as usize + 1];
        let mut wildcard = 0u16;

        for value in signature {
            bytes_used[*value as usize] = true;
            if wildcard == *value {
                while wildcard < SIGNATURE_MAX_VALUE && bytes_used[wildcard as usize] {
                    wildcard += 1;
                }
                if wildcard == SIGNATURE_MAX_VALUE {
                    return (true, None);
                }
            }
        }

        (bytes_used[CMD_WILDCARD as usize], Some(wildcard as u8))
    }

    fn convert_hex_to_bin(digit_string: &str) -> u16 {
        u16::from_str_radix(digit_string, 16).unwrap_or(0)
    }

    fn read_lines(filename: &PathBuf) -> io::Result<LinesDecoded> {
        let file = File::open(filename)?;
        let reader = BufReader::new(
            DecodeReaderBytesBuilder::new()
                .encoding(Some(WINDOWS_1252))
                .build(file));
        Ok(reader.lines())
    }

    pub fn verify_config_file(file_path: &PathBuf) -> Result<bool, String> {
        if !Self::is_config_file(file_path) {
            return Err("Not a config file.".to_string());
        }

        let mut error = false;
        let mut signature_names_added = HashMap::new();

        let mut line_number = 1;
        let mut last_empty_line_number = -1;
        if let Ok(lines) = Self::read_lines(file_path) {
            let mut signature_name = "".to_string();
            let mut signature_lines = vec![];

            for line in lines.flatten() {
                let signature_text = line.trim();

                if Self::is_signature_min_length(signature_text) {
                    if Self::is_signature_name(signature_text) {
                        error |= Self::validate_signature_without_value(&signature_names_added, &signature_name);
                        error |= Self::validate_signature_value_lines(&signature_name, &signature_lines);
                        signature_lines.clear();

                        signature_name = signature_text.to_string();

                        if signature_name.eq_ignore_ascii_case("END") ||
                            signature_name.eq_ignore_ascii_case("AND") {
                            error = true;
                            eprintln!("Signature name cannot be a reserved word: {}\r", signature_name);
                        }

                        if signature_name.contains(' ') {
                            error = true;
                            eprintln!("Signature name contains spaces or invalid signature value: {}\r", signature_name);
                        }

                        if signature_names_added.contains_key(&signature_name.to_ascii_uppercase()) {
                            error = true;
                            eprintln!("Signature defined more than once or with different casing: {}\r", signature_name);
                        }

                        signature_names_added.insert(signature_name.to_ascii_uppercase(), false);
                    } else {
                        if signature_name.is_empty() {
                            error = true;
                            eprintln!("Signature found without a name: {}\r", signature_text);
                        }

                        signature_lines.push(signature_text.to_string());
                        if signature_text.ends_with_ignore_ascii_case_with_uppercase("END") {
                            error |= Self::validate_signature_value(&signature_name, &signature_lines.join(" "));
                            signature_lines.clear();
                        }
                        signature_names_added.insert(signature_name.to_ascii_uppercase(), true);
                    }
                    error |= Self::validate_spaces(&signature_name, signature_text, line.len(), signature_text.len())
                } else {
                    if signature_text.is_empty() && !line.is_empty() {
                        error = true;
                        eprintln!("Line found with only spaces\r");
                    }

                    error |= Self::validate_signature_without_value(&signature_names_added, &signature_name);
                    error |= Self::validate_signature_value_lines(&signature_name, &signature_lines);
                    signature_lines.clear();

                    if !signature_text.is_empty() {
                        error = true;
                        eprintln!("Invalid signature found. Signature name should be at least 3 characters long and signature value line should have at least 2 valid characters: {}\r", signature_text);
                        signature_names_added.insert(signature_name.to_ascii_uppercase(), true);
                    }

                    if line.is_empty() && last_empty_line_number == line_number - 1 {
                        error = true;
                        eprintln!("Two consecutive empty lines found at line: {}\r", line_number);
                    }

                    if error {
                        signature_names_added.insert(signature_name.to_ascii_uppercase(), true);
                    } else {
                        signature_name = "".to_string();
                    }

                    last_empty_line_number = line_number;
                }

                line_number += 1;
            }
            error |= Self::validate_signature_without_value(&signature_names_added, &signature_name);
            error |= Self::validate_signature_value_lines(&signature_name, &signature_lines);
        }
        Ok(error)
    }

    pub fn verify_info_file(file_path: &PathBuf, signatures: &[SignatureConfig]) -> Result<bool, String> {
        let mut error = false;
        let mut signature_names_added = HashMap::new();

        let mut line_number = 0;
        if let Ok(lines) = Self::read_lines(file_path) {
            let mut signature_name = "".to_string();
            let mut previous_tag = "".to_string();
            let mut info_line_found = false;
            let mut signature_name_found = false;

            for line in lines.flatten() {
                line_number += 1;
                let signature_text = line.trim_end();
                if signature_text.len() != line.len() {
                    error = true;
                    eprintln!("Space(s) found at the end of the line on line: {}\r", line_number);
                }

                let signature_text = signature_text.trim();

                if Self::is_info_tag(&line) {
                    if !signature_name_found {
                        error = true;
                        eprintln!("Info found without a signature name at line: {}\r", line_number);
                        previous_tag = "".to_string();
                    }

                    let tag = line.chars().take(10).collect::<String>();
                    let tag = tag.trim();
                    error |= Self::validate_info_tag(&signature_name, tag, &previous_tag);

                    let value = &line.chars().skip(11).collect::<String>();
                    error |= Self::validate_info_tag_value(&signature_name, tag, value);

                    if !tag.is_empty() {
                        previous_tag = tag.to_owned();
                    }

                    info_line_found = true;
                } else if Self::is_signature_name(signature_text) {
                    if signature_name_found && !info_line_found {
                        error = true;
                        eprintln!("Signature name found without any info: {}\r", signature_name);
                    }

                    if let Some(position) = signature_text.find(':') {
                        error = true;
                        eprintln!("Wrong indentation '{}' or invalid tag in: {}\r", &signature_text[..=position], signature_name);
                        continue;
                    }

                    if signature_names_added.contains_key(signature_text) {
                        error = true;
                        eprintln!("Signature name defined more than once: {}\r", signature_text);
                    }

                    previous_tag = "".to_string();
                    signature_name = signature_text.to_owned();
                    signature_names_added.insert(signature_text.to_owned(), true);

                    signature_name_found = true;
                    info_line_found = false;
                } else {
                    if signature_name_found && !info_line_found {
                        error = true;
                        eprintln!("Signature name found without any info: {}\r", signature_name);
                    }

                    signature_name_found = false;
                    info_line_found = false;
                }
            }
        }

        for signature_name in signature_names_added {
            if !signatures.iter().any(|signature| signature.signature_name.eq(&signature_name.0)) {
                error = true;
                eprintln!("Signature ID not found in config file: {}\r", signature_name.0);
            }
        }
        Ok(error)
    }

    fn validate_signature_value_lines(signature_name: &str, signature_lines: &Vec<String>) -> bool {
        let mut error = false;
        for signature_line in signature_lines {
            error |= Self::validate_signature_value(signature_name, signature_line);
        }
        error
    }

    fn validate_signature_without_value(signature_names_added: &HashMap<String, bool>, signature_name: &String) -> bool {
        let mut error = false;
        if !signature_name.is_empty() {
            let has_signature_value = signature_names_added.get(&signature_name.to_ascii_uppercase());
            if !has_signature_value.unwrap() {
                error = true;
                eprintln!("Signature name found without a value: {}\r", signature_name);
            }
        }
        error
    }

    fn validate_spaces(signature_name: &str, signature_value: &str, line_length: usize, signature_size: usize) -> bool {
        let mut error = false;
        if line_length != signature_size {
            error = true;
            eprintln!("Signature contains spaces at beginning or at the end of the line: {}\r", signature_name);
        } else if signature_value.contains("  ") {
            error = true;
            eprintln!("Signature contains double spaces: {}\r", signature_name);
        }
        error
    }

    fn validate_signature_value(signature_name: &str, signature_text: &str) -> bool {
        let mut error = false;

        let signature_text_upper = signature_text.to_ascii_uppercase();

        if !signature_text.eq(&signature_text_upper) {
            error = true;
            eprintln!("Signature contains lowercase characters: {}\r", signature_name);
        }

        let signature_text_without_end = signature_text.replace(" END", "");
        if signature_text_without_end.len() <= 4 {
            error = true;
            eprintln!("Invalid signature found. Signature value should have at least 2 values separated with a space: {}\r", signature_name);
        }

        if signature_text_without_end.ends_with(" AND") || signature_text_without_end.ends_with(" &&") {
            error = true;
            eprintln!("Signature should not end with an AND or && operator: {}\r", signature_name);
        }

        for signature in signature_text_upper.split(" AND ") {
            for signature in signature.split(" && ") {
                error |= Self::validate_signature_range(signature_name, signature);
            }
        }
        error
    }

    fn validate_signature_range(signature_name: &str, signature_text: &str) -> bool {
        let mut error = false;
        let mut it = signature_text.split_ascii_whitespace().enumerate().peekable();
        while let Some((index, word)) = it.next() {
            if index == 255 {
                error = true;
                eprintln!("Signature cannot be larger than 254 bytes: {}\r", signature_name);
            }
            match word {
                "??" => {
                    if index == 0 || it.peek().is_none() || it.peek().unwrap().1.eq_ignore_ascii_case("END") {
                        error = true;
                        eprintln!("Signature ID or SUB ID (with AND operator) should not begin or end with a wildcard: {}\r", signature_name);
                    }
                },
                "END" => {
                    if it.peek().is_some() {
                        error = true;
                        eprintln!("Signature END operator can only be present at the end of the line: {}\r", signature_name);
                    }
                },
                "AND" | "&&" => {
                    if index == 0 {
                        error = true;
                        eprintln!("Signature should not begin with an AND or && operator: {}\r", signature_name);
                    }
                },
                _ => {
                    let valid_chars = word.bytes().all(|b| matches!(b, b'a'..=b'f' | b'A'..=b'F' | b'0'..=b'9'));
                    if !valid_chars || (!word.is_empty() && word.len() != 2) {
                        error = true;
                        eprintln!("Unsupported value '{}' in signature: {}\r", word, signature_name);
                    }
                }
            }
        }
        error
    }

    fn validate_info_tag_value(signature_name: &str, tag: &str, value: &str) -> bool {
        let mut error = false;

        if let Some(first_char) = value.chars().next() {
            if first_char.is_ascii_whitespace() {
                error = true;
                eprintln!("Value in '{}' is not correctly aligned in: {}\r", tag.trim(), signature_name);
            }
        }

        if tag.eq_ignore_ascii_case("REFERENCE:") && !value.trim().to_ascii_uppercase().starts_with("HTTP") {
            error = true;
            eprintln!("Reference has an invalid URL in signature: {}\r", signature_name);
        }
        error
    }

    fn validate_info_tag(signature_name: &str, tag: &str, previous_tag: &str) -> bool {
        match tag {
            "" | "AUTHOR:" | "RELEASED:" | "NAME:" | "REFERENCE:" | "COMMENT:" => {
                Self::validate_order(signature_name, tag, previous_tag)
            },
            _ => {
                eprintln!("Invalid tag found '{}' in signature: {}\r", tag, signature_name);
                true
            }
        }
    }

    fn validate_order(signature_name: &str, tag: &str, previous_tag: &str) -> bool {
        if !previous_tag.is_empty() {
            let tag_order = Self::get_order(tag);
            let previous_tag_order = Self::get_order(previous_tag);

            let mut error = tag_order <= previous_tag_order;
            if error {
                eprintln!("Order of tags '{}' '{}' is not valid: {}\r", tag, previous_tag, signature_name);
            }

            let multi_line_detected_on_non_comment = tag_order == 6 && previous_tag_order < 5;
            if multi_line_detected_on_non_comment {
                error = true;
                eprintln!("Multi-line not allowed for tag '{}' in: {}\r", previous_tag, signature_name);
            }
            error
        } else {
            false
        }
    }

    fn get_order(tag: &str) -> i32 {
        match tag.trim() {
            "NAME:" => 1,
            "AUTHOR:" => 2,
            "RELEASED:" => 3,
            "REFERENCE:" => 4,
            "COMMENT:" => 5,
            "" => 6,
            _ => 0
        }
    }
}
