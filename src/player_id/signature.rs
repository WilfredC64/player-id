// Copyright (C) 2019 - 2022 Wilfred Bos
// Licensed under the MIT license. See the LICENSE file for the terms and conditions.

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Lines, Read};
use std::path::PathBuf;

use encoding_rs::WINDOWS_1252;
use encoding_rs_io::{DecodeReaderBytesBuilder, DecodeReaderBytes};

use super::bndm::{BndmConfig, find_pattern};

const CMD_WILDCARD: u16 = 0x100;

type LinesDecoded = Lines<BufReader<DecodeReaderBytes<File, Vec<u8>>>>;

pub struct SignatureHolder {
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
    pub fn find_signatures(source: &[u8], start_offset: usize, signatures: &Vec<SignatureHolder>, scan_for_multiple: bool) -> Vec<SignatureMatch> {
        let mut matches = vec![];

        let mut signature_names_added = HashMap::new();

        for signature in signatures {
            let configs = &signature.bndm_configs;
            let mut indexes = vec![];

            let mut index_found = false;
            let mut last_index = start_offset;
            for config in configs {
                let index = find_pattern(&source[last_index..], config);
                if let Some(index) = index {
                    index_found = true;
                    indexes.push(last_index + index - start_offset);
                    last_index += index + config.pattern.len();
                } else {
                    index_found = false;
                    indexes.clear();
                    break;
                }
            }

            if index_found && !signature_names_added.contains_key(&signature.signature_name) {
                signature_names_added.insert(signature.signature_name.to_owned(), true);
                matches.push(SignatureMatch { signature_name: signature.signature_name.to_owned(), indexes });

                if !scan_for_multiple {
                    break;
                }
            }
        }

        matches
    }

    pub fn find_signature_info(signature_infos: &[SignatureInfo], signature_name: &str) -> Option<SignatureInfo> {
        signature_infos.iter().find(|info| info.signature_name.eq_ignore_ascii_case(signature_name)).cloned()
    }

    pub fn read_config_file(file_path: &PathBuf, signature_name_to_filter: Option<String>) -> Result<Vec<SignatureHolder>, String> {
        if !Self::is_config_file(file_path) {
            return Err("Not a config file.".to_string());
        }

        let signature_name_to_filter = signature_name_to_filter.unwrap_or_default();
        let mut signatures = vec![];

        if let Ok(lines) = Self::read_lines(file_path) {
            let mut signature_name = "".to_string();

            let mut signature_lines = vec![];
            for line in lines.flatten() {
                let signature_text = line.trim();

                if Self::is_signature_min_length(signature_text) {
                    if Self::is_signature_name(signature_text) {
                        Self::process_multi_signatures(&signature_name_to_filter, &mut signatures, &signature_name, &mut signature_lines);
                        signature_name = signature_text.to_string();
                    } else {
                        signature_lines.push(signature_text.to_string());
                        if signature_text.len() >= 3 && signature_text[signature_text.len() - 3..].eq_ignore_ascii_case("END") {
                            Self::process_single_signature(&signature_name_to_filter, &mut signatures, &signature_name, &mut signature_lines);
                        }
                    }
                } else {
                    Self::process_multi_signatures(&signature_name_to_filter, &mut signatures, &signature_name, &mut signature_lines);
                    signature_name = "".to_string();
                }
            }
            Self::process_multi_signatures(&signature_name_to_filter, &mut signatures, &signature_name, &mut signature_lines);
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
                    if Self::is_info_tag(&line){
                        info_lines.push(line);
                    } else if Self::is_signature_name(&line) {
                        if !signature_name.is_empty() {
                            signature_infos.push(SignatureInfo { signature_name, info_lines: info_lines.to_owned() });
                        }
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
            let reader = BufReader::new(
                DecodeReaderBytesBuilder::new()
                    .encoding(Some(WINDOWS_1252))
                    .build(file));
            let chunk = reader.take(1000);
            let lines = chunk.lines()
                .map(|line| line.unwrap_or_default())
                .collect::<Vec<_>>();
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
            let reader = BufReader::new(
                DecodeReaderBytesBuilder::new()
                    .encoding(Some(WINDOWS_1252))
                    .build(file));
            let chunk = reader.take(1000);
            let lines = chunk.lines()
                .map(|line| line.unwrap_or_default())
                .collect::<Vec<_>>();
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

    fn process_multi_signatures(signature_name_to_filter: &str, signatures: &mut Vec<SignatureHolder>, signature_name: &str, signature_lines: &mut Vec<String>) {
        for signature_line in &*signature_lines {
            Self::process_signature_line(signature_name_to_filter, signatures, signature_name, signature_line);
        }
        signature_lines.clear();
    }

    fn process_single_signature(signature_name_to_filter: &str, signatures: &mut Vec<SignatureHolder>, signature_name: &str, signature_lines: &mut Vec<String>) {
        Self::process_signature_line(signature_name_to_filter, signatures, signature_name, &signature_lines.join(" "));
        signature_lines.clear();
    }

    fn process_signature_line(signature_name_to_filter: &str, signatures: &mut Vec<SignatureHolder>, signature_name: &str, signature_text: &str) {
        let signature = Self::process_signature_value(signature_name, signature_text);
        if signature_name_to_filter.is_empty() || signature_name_to_filter.eq_ignore_ascii_case(signature_name) {
            signatures.push(signature);
        }
    }

    fn process_signature_value(signature_name: &str, signature_text: &str) -> SignatureHolder {
        let mut signature = vec![];
        let mut bndm_configs = vec![];

        for word in signature_text.to_ascii_uppercase().split_ascii_whitespace() {
            if !word.is_empty() {
                match word {
                    "??" => signature.push(CMD_WILDCARD),
                    "AND" | "&&" | "END" => {
                        Self::add_signature(&signature, &mut bndm_configs);
                        signature.clear();
                    },
                    _ => signature.push(Self::convert_hex_to_bin(word))
                }
            }
        }

        if !signature.is_empty() {
            Self::add_signature(&signature, &mut bndm_configs);
        }

        SignatureHolder { signature_name: signature_name.to_string(), bndm_configs }
    }

    fn is_signature_min_length(signature_text_line: &str) -> bool {
        signature_text_line.len() >= 2
    }

    fn is_info_tag(signature_text_line: &str) -> bool {
        if signature_text_line.len() >= 11 {
            let signature_first_11bytes = signature_text_line[..11].as_bytes();
            if (signature_first_11bytes[9] == b':' && signature_first_11bytes[10] == b' ') ||
                signature_text_line[..11].eq("           ") {
                return true;
            }
        }
        false
    }

    fn is_signature_name(signature_text_line: &str) -> bool {
        if signature_text_line.len() >= 3 {
            let signature_first_3bytes = signature_text_line[..3].as_bytes();
            if signature_first_3bytes[2] != b' ' {
                return signature_text_line.len() > 3 ||
                    (!signature_first_3bytes.eq_ignore_ascii_case(b"END") && !signature_first_3bytes.eq_ignore_ascii_case(b"AND"));
            }
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
                            println!("Signature name cannot be a reserved word: {}", signature_name);
                        }

                        if signature_name.contains(' ') {
                            error = true;
                            println!("Signature name contains spaces or invalid signature value: {}", signature_name);
                        }

                        if signature_names_added.contains_key(&signature_name.to_ascii_uppercase()) {
                            error = true;
                            println!("Signature defined more than once or with different casing: {}", signature_name);
                        }

                        signature_names_added.insert(signature_name.to_ascii_uppercase(), false);
                    } else {
                        if signature_name.is_empty() {
                            error = true;
                            println!("Signature found without a name: {}", signature_text);
                        }

                        signature_lines.push(signature_text.to_string());
                        if signature_text.len() >= 3 && signature_text[signature_text.len() - 3..].eq_ignore_ascii_case("END") {
                            error |= Self::validate_signature_value(&signature_name, &signature_lines.join(" "));
                            signature_lines.clear();
                        }
                        signature_names_added.insert(signature_name.to_ascii_uppercase(), true);
                    }
                    error |= Self::validate_spaces(&signature_name, signature_text, line.len(), signature_text.len())
                } else {
                    if signature_text.is_empty() && !line.is_empty() {
                        error = true;
                        println!("Line found with only spaces");
                    }

                    error |= Self::validate_signature_without_value(&signature_names_added, &signature_name);
                    error |= Self::validate_signature_value_lines(&signature_name, &signature_lines);
                    signature_lines.clear();

                    if !signature_text.is_empty() {
                        error = true;
                        println!("Invalid signature found. Signature name should be at least 3 characters long and signature value line should have at least 2 valid characters: {}", signature_text);
                        signature_names_added.insert(signature_name.to_ascii_uppercase(), true);
                    }

                    if line.is_empty() && last_empty_line_number == line_number - 1 {
                        println!("Two consecutive empty lines found at line: {}", line_number);
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

    pub fn verify_info_file(file_path: &PathBuf, signatures: &[SignatureHolder]) -> Result<bool, String> {
        let mut error = false;
        let mut signature_names_added = HashMap::new();

        let mut signature_name = "".to_string();
        let mut previous_tag = "".to_string();

        let mut line_number = 0;
        if let Ok(lines) = Self::read_lines(file_path) {
            for line in lines.flatten() {
                line_number += 1;
                let signature_text = line.trim_end();
                if signature_text.len() != line.len() {
                    error = true;
                    println!("Space(s) found at the end of the line on line: {}", line_number);
                }

                let signature_text = signature_text.trim();

                if Self::is_info_tag(&line) {
                    Self::validate_info_tag(&signature_name, &line[..11], &previous_tag);
                    let tag = line[..11].trim();
                    if !tag.is_empty() {
                        previous_tag = line[..11].trim().to_owned();
                    }
                } else if Self::is_signature_name(signature_text) {
                    previous_tag = "".to_string();

                    let position = signature_text.find(':');
                    if let Some(position) = position {
                        error = true;
                        println!("Wrong indentation '{}' in: {}", &signature_text[..=position], signature_name);
                        continue;
                    }

                    if signature_names_added.contains_key(signature_text) {
                        error = true;
                        println!("Signature info defined more than once: {}", signature_text);
                    }

                    signature_name = signature_text.to_owned();
                    signature_names_added.insert(signature_text.to_owned(), true);
                }
            }
        }

        for signature_name in signature_names_added {
            if !signatures.iter().any(|signature| signature.signature_name.eq(&signature_name.0)) {
                error = true;
                println!("Signature ID not found in config file: {}", signature_name.0);
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
                println!("Signature name found without a value: {}", signature_name);
            }
        }
        error
    }

    fn validate_spaces(signature_name: &str, signature_value: &str, line_length: usize, signature_size: usize) -> bool {
        let mut error = false;
        if line_length != signature_size {
            error = true;
            println!("Signature contains spaces at beginning or at the end of the line: {}", signature_name);
        } else if signature_value.contains("  ") {
            error = true;
            println!("Signature contains double spaces: {}", signature_name);
        }
        error
    }

    fn validate_signature_value(signature_name: &str, signature_text: &str) -> bool {
        let mut error = false;

        let signature_text_upper = signature_text.to_ascii_uppercase();

        if !signature_text.eq(&signature_text_upper) {
            error = true;
            println!("Signature contains lowercase characters: {}", signature_name);
        }

        let signature_text_without_end = signature_text.replace(" END", "");
        if signature_text_without_end.len() <= 4 {
            error = true;
            println!("Invalid signature found. Signature value should have at least 2 values separated with a space: {}", signature_name);
        }

        if signature_text_without_end.ends_with(" AND") || signature_text_without_end.ends_with(" &&"){
            error = true;
            println!("Signature should not end with an AND or && operator: {}", signature_name);
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
                println!("Signature cannot be larger than 254 bytes: {}", signature_name);
            }
            match word {
                "??" => {
                    if index == 0 || it.peek().is_none() || it.peek().unwrap().1.eq_ignore_ascii_case("END") {
                        error = true;
                        println!("Signature ID or SUB ID (with AND operator) should not begin or end with a wildcard: {}", signature_name);
                    }
                },
                "END" => {
                    if it.peek().is_some() {
                        error = true;
                        println!("Signature END operator can only be present at the end of the line: {}", signature_name);
                    }
                },
                "AND" | "&&" => {
                    if index == 0 {
                        error = true;
                        println!("Signature should not begin with an AND or && operator: {}", signature_name);
                    }
                },
                _ => {
                    let valid_chars = word.bytes().all(|b| matches!(b, b'a'..=b'f' | b'A'..=b'F' | b'0'..=b'9'));
                    if !valid_chars || (!word.is_empty() && word.len() != 2) {
                        error = true;
                        println!("Unsupported value '{}' in signature: {}", word, signature_name);
                    }
                }
            }
        }
        error
    }

    fn validate_info_tag(signature_name: &str, tag: &str, previous_tag: &str) -> bool {
        let tag = tag.trim();
        match tag {
            "" | "AUTHOR:" | "RELEASED:" | "NAME:" | "REFERENCE:" | "COMMENT:" => {
                let error = Self::validate_order(tag, previous_tag);
                if error {
                    println!("Order of tags '{}' '{}' is not valid: {}", tag, previous_tag, signature_name);
                }
                error
            },
            _ => {
                println!("Invalid tag found '{}' in signature: {}", tag, signature_name);
                false
            }
        }
    }

    fn validate_order(tag: &str, previous_tag: &str) -> bool {
        if !tag.is_empty() && !previous_tag.is_empty() {
            let tag_order = Self::get_order(tag);
            let previous_tag_order = Self::get_order(previous_tag);
            tag_order <= previous_tag_order
        } else {
            false
        }
    }

    fn get_order(tag: &str) -> i32 {
        let tag = tag.trim();
        match tag {
            "" => 0,
            "NAME:" => 1,
            "AUTHOR:" => 2,
            "RELEASED:" => 3,
            "REFERENCE:" => 4,
            "COMMENT:" => 5,
            _ => 0
        }
    }
}
