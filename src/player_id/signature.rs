// Copyright (C) 2019 - 2024 Wilfred Bos
// Licensed under the MIT license. See the LICENSE file for the terms and conditions.

use bndm::{BndmConfig, find_pattern};

const CMD_WILDCARD: u16 = 0x100;

pub struct SignatureConfig {
    pub bndm_configs: Vec<BndmConfig>,
    pub signature_name: String
}

pub struct SignatureMatch {
    pub signature_name: String,
    pub indexes: Vec<usize>,
}

pub type SignatureInfo = (String, Vec<String>);

pub struct Signature {}

impl Signature {
    pub fn find_signatures(source: &[u8], start_offset: usize, signatures: &Vec<SignatureConfig>, scan_for_multiple: bool) -> Vec<SignatureMatch> {
        let mut matches = vec![];

        for signature in signatures {
            let mut indexes = vec![];
            let mut index_found = true;
            let mut last_index = start_offset;

            for config in &signature.bndm_configs {
                if let Some(index) = find_pattern(&source[last_index..], config) {
                    indexes.push(last_index + index);
                    last_index += index + config.pattern.len();
                } else {
                    index_found = false;
                    break;
                }
            }

            if index_found {
                matches.push(SignatureMatch { signature_name: signature.signature_name.to_string(), indexes });

                if !scan_for_multiple {
                    break;
                }
            }
        }
        matches.dedup_by(|a, b| a.signature_name.eq(&b.signature_name));
        matches
    }

    pub fn find_signature_info<'a>(signature_infos: &'a [SignatureInfo], signature_name: &str) -> Option<&'a SignatureInfo> {
        signature_infos.iter().find(|(signature_info_name, _)| signature_info_name.eq_ignore_ascii_case(signature_name))
    }

    pub fn read_config_lines(config_lines: &Vec<String>, signature_name_to_filter: Option<&String>) -> Result<Vec<SignatureConfig>, String> {
        if !Self::is_config_file(config_lines) {
            return Err("Not an config file.".to_string());
        }

        let mut signatures = vec![];
        let mut signature_name = "".to_string();
        let mut signature_lines = vec![];

        for line in config_lines {
            let signature_text = line.trim();

            if Self::is_signature_min_length(signature_text) {
                if Self::is_signature_name(signature_text) {
                    Self::process_multi_signatures(signature_name_to_filter, &mut signatures, &signature_name, &mut signature_lines);
                    signature_name = signature_text.to_string();
                } else {
                    signature_lines.push(signature_text.to_string());
                    if Self::has_end_marker(signature_text) {
                        Self::process_single_signature(signature_name_to_filter, &mut signatures, &signature_name, &mut signature_lines);
                    }
                }
            } else {
                Self::process_multi_signatures(signature_name_to_filter, &mut signatures, &signature_name, &mut signature_lines);
                signature_name = "".to_string();
            }
        }

        Self::process_multi_signatures(signature_name_to_filter, &mut signatures, &signature_name, &mut signature_lines);
        Ok(signatures)
    }

    pub fn has_end_marker(text: &str) -> bool {
        let text_len = text.len();
        text_len >= 3 && text.as_bytes()[text_len - 3..].eq_ignore_ascii_case(b"END")
    }

    pub fn read_info_lines(lines: &Vec<String>) -> Result<Vec<SignatureInfo>, String> {
        if !Self::is_info_file(lines) {
            return Err("Not an info file.".to_string());
        }

        let mut signature_infos = vec![];
        let mut signature_name = "".to_string();

        let mut info_lines = vec![];
        for line in lines {
            if Self::is_signature_min_length(line) {
                if Self::is_info_tag(line) {
                    info_lines.push(line.to_string());
                } else if Self::is_signature_name(line) {
                    if !signature_name.is_empty() {
                        signature_infos.push((signature_name, info_lines.to_owned()));
                    }
                    info_lines.clear();
                    signature_name = line.to_string();
                } else {
                    signature_infos.push((signature_name, info_lines.to_owned()));
                    info_lines.clear();
                    signature_name = "".to_string();
                }
            } else {
                signature_infos.push((signature_name, info_lines.to_owned()));
                info_lines.clear();
                signature_name = "".to_string();
            }
        }

        if !info_lines.is_empty() {
            signature_infos.push((signature_name, info_lines.to_owned()));
        }
        Ok(signature_infos)
    }

    pub fn is_config_file(config_lines: &[String]) -> bool {
        let mut lines_iter = config_lines.iter();

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
        false
    }

    pub fn is_info_file(info_lines: &[String]) -> bool {
        let mut lines_iter = info_lines.iter();

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
        false
    }

    pub fn is_signature_min_length(signature_text_line: &str) -> bool {
        signature_text_line.len() >= 2
    }

    pub fn is_info_tag(signature_text_line: &str) -> bool {
        if let Some(chars) = signature_text_line.as_bytes().get(..11) {
            (chars[9] == b':' && chars[10] == b' ') || chars == b"           "
        } else {
            false
        }
    }

    pub fn is_signature_name(signature_text_line: &str) -> bool {
        if let Some(chars) = signature_text_line.as_bytes().get(..3) {
            chars[2] != b' ' && (signature_text_line.len() > 3 || !matches!(&chars.to_ascii_uppercase()[..], b"END" | b"AND"))
        } else {
            false
        }
    }

    fn process_multi_signatures(signature_name_to_filter: Option<&String>, signatures: &mut Vec<SignatureConfig>, signature_name: &str, signature_lines: &mut Vec<String>) {
        for signature_line in signature_lines.drain(..) {
            Self::process_signature_line(signature_name_to_filter, signatures, signature_name, &signature_line);
        }
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
                    _ => signature.push(Self::convert_hex_to_bin(&word[..2]))
                }
            }
        }

        if !signature.is_empty() {
            Self::add_signature(&signature, &mut bndm_configs);
        }

        SignatureConfig { signature_name: signature_name.to_string(), bndm_configs }
    }

    fn add_signature(signature: &[u16], bndm_configs: &mut Vec<BndmConfig>) {
        let (wildcard_used, calculated_wildcard) = Self::calculate_wildcard(signature);

        if !wildcard_used || calculated_wildcard.is_some() {
            let mut new_signature = Vec::with_capacity(signature.len());

            for value in signature {
                if *value == CMD_WILDCARD {
                    new_signature.push(calculated_wildcard.unwrap());
                } else {
                    new_signature.push(*value as u8);
                }
            }

            bndm_configs.push(BndmConfig::new(&new_signature, calculated_wildcard));
        }
    }

    fn calculate_wildcard(signature: &[u16]) -> (bool, Option<u8>) {
        const SIGNATURE_MAX_VALUE: u16 = 0x100; // only bytes 0x00 - 0xFF are used, and 0x100 for the wildcard

        let mut bytes_used = [false; SIGNATURE_MAX_VALUE as usize + 1];
        for &value in signature {
            bytes_used[value as usize] = true;
        }

        let mut wildcard = 0u16;
        while wildcard < SIGNATURE_MAX_VALUE && bytes_used[wildcard as usize] {
            wildcard += 1;
        }

        if wildcard == SIGNATURE_MAX_VALUE {
            (true, None)
        } else {
            (bytes_used[CMD_WILDCARD as usize], Some(wildcard as u8))
        }
    }

    fn convert_hex_to_bin(digit_string: &str) -> u16 {
        u16::from_str_radix(digit_string, 16).unwrap_or(0)
    }
}
