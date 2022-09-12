// Copyright (C) 2019 - 2022 Wilfred Bos
// Licensed under the MIT license. See the LICENSE file for the terms and conditions.

use std::path::Path;

pub fn get_hvsc_root(filename: &str) -> Option<String> {
    let mut path = Path::new(filename);
    if path.is_file() {
        path = path.parent().unwrap();
    }

    let stil_txt = path.join("STIL.txt");
    if stil_txt.exists() {
        return Some(path.parent().unwrap().to_str().unwrap().to_string());
    }

    let stil_txt = path.join("C64Music");
    if stil_txt.join("DOCUMENTS").join("STIL.txt").exists() {
        return Some(stil_txt.to_str().unwrap().to_string());
    }

    loop {
        let stil_txt = path.join("DOCUMENTS").join("STIL.txt");
        if stil_txt.exists() {
            return Some(path.to_str().unwrap().to_string());
        }

        if path.parent().is_none() {
            break;
        }
        path = path.parent().unwrap();
    }
    None
}
