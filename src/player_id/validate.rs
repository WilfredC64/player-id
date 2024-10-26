// Copyright (C) 2019 - 2024 Wilfred Bos
// Licensed under the MIT license. See the LICENSE file for the terms and conditions.

use std::collections::HashMap;
use crate::player_id::SignatureConfig;
use crate::player_id::signature::Signature;

pub fn verify_config_file(config_lines: &Vec<String>) -> Result<bool, String> {
    let mut error = false;
    let mut signature_names_added = HashMap::new();

    let mut line_number = 1;
    let mut last_empty_line_number = -1;
    let mut signature_name = "".to_string();
    let mut signature_lines = vec![];

    for line in config_lines {
        let signature_text = line.trim();

        if Signature::is_signature_min_length(signature_text) {
            if Signature::is_signature_name(signature_text) {
                error |= validate_signature_without_value(&signature_names_added, &signature_name);
                error |= validate_signature_value_lines(&signature_name, &signature_lines);
                signature_lines.clear();

                signature_name = signature_text.to_string();

                error |= validate_signature_name(&signature_name, &signature_names_added);

                signature_names_added.insert(signature_name.to_ascii_uppercase(), false);
            } else {
                if signature_name.is_empty() {
                    error = true;

                    if signature_text.eq_ignore_ascii_case("END") ||
                        signature_text.eq_ignore_ascii_case("AND") {
                        eprintln!("Signature name cannot be a reserved word at line: {line_number}\r");
                    } else {
                        eprintln!("Signature found without a name: {signature_text}\r");
                    }
                }

                signature_lines.push(signature_text.to_string());
                if Signature::has_end_marker(signature_text) {
                    error |= validate_signature_value(&signature_name, &signature_lines.join(" "));
                    signature_lines.clear();
                }
                signature_names_added.insert(signature_name.to_ascii_uppercase(), true);
            }
            error |= validate_spaces(&signature_name, signature_text, line.len(), signature_text.len())
        } else {
            if signature_text.is_empty() && !line.is_empty() {
                error = true;
                eprintln!("Line found with only spaces\r");
            }

            error |= validate_signature_without_value(&signature_names_added, &signature_name);
            error |= validate_signature_value_lines(&signature_name, &signature_lines);
            signature_lines.clear();

            if !signature_text.is_empty() {
                error = true;
                eprintln!("Invalid signature found. Signature name should be at least 3 characters long and signature value line should have at least 2 valid characters: {signature_text}\r");
                signature_names_added.insert(signature_name.to_ascii_uppercase(), true);
            }

            if line.is_empty() && last_empty_line_number == line_number - 1 {
                error = true;
                eprintln!("Two consecutive empty lines found at line: {line_number}\r");
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

    error |= validate_signature_without_value(&signature_names_added, &signature_name);
    error |= validate_signature_value_lines(&signature_name, &signature_lines);
    Ok(error)
}

pub fn verify_info_file(info_lines: &Vec<String>, signatures: &[SignatureConfig]) -> Result<bool, String> {
    let mut error = false;
    let mut signature_names_added = HashMap::new();

    let mut line_number = 0;
    let mut last_empty_line_number = -1;
    let mut signature_name = "".to_string();
    let mut previous_tag = "".to_string();
    let mut info_line_found = false;
    let mut signature_name_found = false;

    for line in info_lines {
        line_number += 1;
        let signature_text = line.trim_end();
        if signature_text.len() != line.len() {
            error = true;
            eprintln!("Space(s) found at the end of the line on line: {line_number}\r");
        }

        let signature_text = signature_text.trim();

        if Signature::is_info_tag(line) {
            if !signature_name_found {
                error = true;
                eprintln!("Info found without a signature name at line: {line_number}\r");
                previous_tag = "".to_string();
            }

            let tag = line.chars().take(10).collect::<String>();
            let tag = tag.trim();
            error |= validate_info_tag(&signature_name, tag, &previous_tag);

            let value = &line.chars().skip(11).collect::<String>();
            error |= validate_info_tag_value(&signature_name, tag, value);

            if !tag.is_empty() {
                previous_tag = tag.to_string();
            }

            info_line_found = true;
        } else if Signature::is_signature_name(signature_text) {
            error |= validate_signature_exists_in_config(signatures, signature_text);

            if signature_name_found && !info_line_found {
                error = true;
                eprintln!("Signature name found without any info: {signature_name}\r");
            }

            if let Some(position) = signature_text.find(':') {
                error = true;
                eprintln!("Wrong indentation '{}' or invalid tag in: {}\r", &signature_text[..=position], signature_name);
                continue;
            }

            error |= validate_signature_name(signature_text, &signature_names_added);

            previous_tag = "".to_string();
            signature_name = signature_text.to_string();
            signature_names_added.insert(signature_text.to_ascii_uppercase(), true);

            signature_name_found = true;
            info_line_found = false;
        } else {
            if signature_name_found && !info_line_found {
                error = true;
                eprintln!("Signature name found without any info: {signature_name}\r");
            }

            if line.is_empty() && last_empty_line_number == line_number - 1 {
                error = true;
                eprintln!("Two consecutive empty lines found at line: {line_number}\r");
            }
            last_empty_line_number = line_number;

            signature_name_found = false;
            info_line_found = false;
        }
    }

    Ok(error)
}

fn validate_signature_exists_in_config(signatures: &[SignatureConfig], signature_name: &str) -> bool {
    let mut error = false;

    if !signatures.iter().any(|signature| signature.signature_name.eq(signature_name)) {
        error = true;
        eprintln!("Signature ID not found in config file: {signature_name}\r");
    }
    error
}

fn validate_signature_name(signature_name: &str, signature_names_added: &HashMap<String, bool>) -> bool {
    let mut error = false;

    if signature_name.contains(' ') {
        error = true;
        eprintln!("Signature name contains spaces or invalid signature value: {signature_name}\r");
    }

    if signature_names_added.contains_key(&signature_name.to_ascii_uppercase()) {
        error = true;
        eprintln!("Signature defined more than once or with different casing: {signature_name}\r");
    }
    error
}

fn validate_signature_value_lines(signature_name: &str, signature_lines: &Vec<String>) -> bool {
    let mut error = false;
    for signature_line in signature_lines {
        error |= validate_signature_value(signature_name, signature_line);
    }
    error
}

fn validate_signature_without_value(signature_names_added: &HashMap<String, bool>, signature_name: &String) -> bool {
    let mut error = false;
    if !signature_name.is_empty() {
        let has_signature_value = signature_names_added.get(&signature_name.to_ascii_uppercase());
        if !has_signature_value.unwrap() {
            error = true;
            eprintln!("Signature name found without a value: {signature_name}\r");
        }
    }
    error
}

fn validate_spaces(signature_name: &str, signature_value: &str, line_length: usize, signature_size: usize) -> bool {
    let mut error = false;
    if line_length != signature_size {
        error = true;
        eprintln!("Signature contains spaces at beginning or at the end of the line: {signature_name}\r");
    } else if signature_value.contains("  ") {
        error = true;
        eprintln!("Signature contains double spaces: {signature_name}\r");
    }
    error
}

fn validate_signature_value(signature_name: &str, signature_text: &str) -> bool {
    let mut error = false;

    let signature_text_upper = signature_text.to_ascii_uppercase();

    if signature_text.ne(&signature_text_upper) {
        error = true;
        eprintln!("Signature contains lowercase characters: {signature_name}\r");
    }

    let signature_text_without_end = signature_text.replace(" END", "");
    if signature_text_without_end.len() <= 4 {
        error = true;
        eprintln!("Invalid signature found. Signature value should have at least 2 values separated with a space: {signature_name}\r");
    }

    if signature_text_without_end.ends_with(" AND") || signature_text_without_end.ends_with(" &&") {
        error = true;
        eprintln!("Signature should not end with an AND or && operator: {signature_name}\r");
    }

    for signature in signature_text_upper.split(" AND ") {
        for signature in signature.split(" && ") {
            error |= validate_signature_range(signature_name, signature);
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
            eprintln!("Signature cannot be larger than 254 bytes: {signature_name}\r");
        }
        match word {
            "??" => {
                if index == 0 || it.peek().is_none() || it.peek().unwrap().1.eq_ignore_ascii_case("END") {
                    error = true;
                    eprintln!("Signature ID or SUB ID (with AND operator) should not begin or end with a wildcard: {signature_name}\r");
                }
            },
            "END" => {
                if it.peek().is_some() {
                    error = true;
                    eprintln!("Signature END operator can only be present at the end of the line: {signature_name}\r");
                }
            },
            "AND" | "&&" => {
                if index == 0 {
                    error = true;
                    eprintln!("Signature should not begin with an AND or && operator: {signature_name}\r");
                }
            },
            _ => {
                let valid_chars = word.bytes().all(|b| b.is_ascii_hexdigit());
                if !valid_chars || (!word.is_empty() && word.len() != 2) {
                    error = true;
                    eprintln!("Unsupported value '{word}' in signature: {signature_name}\r");
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
        eprintln!("Reference has an invalid URL in signature: {signature_name}\r");
    }
    error
}

fn validate_info_tag(signature_name: &str, tag: &str, previous_tag: &str) -> bool {
    match tag {
        "" | "AUTHOR:" | "RELEASED:" | "NAME:" | "REFERENCE:" | "COMMENT:" => {
            validate_order(signature_name, tag, previous_tag)
        },
        _ => {
            eprintln!("Invalid tag found '{tag}' in signature: {signature_name}\r");
            true
        }
    }
}

fn validate_order(signature_name: &str, tag: &str, previous_tag: &str) -> bool {
    if !previous_tag.is_empty() {
        let tag_order = get_order(tag);
        let previous_tag_order = get_order(previous_tag);

        let mut error = tag_order <= previous_tag_order;
        if error {
            eprintln!("Order of tags '{tag}' '{previous_tag}' is not valid: {signature_name}\r");
        }

        let multi_line_detected_on_non_comment = tag_order == 6 && previous_tag_order < 5;
        if multi_line_detected_on_non_comment {
            error = true;
            eprintln!("Multi-line not allowed for tag '{previous_tag}' in: {signature_name}\r");
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
