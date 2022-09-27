use super::*;

#[test]
fn split_file_path_no_path() {
    let input = "*.sid";
    let (base_path, file) = Config::split_file_path(input);

    assert_eq!(base_path, ".".to_string());
    assert_eq!(file, "*.sid".to_string());
}

#[test]
fn split_file_path_root_path() {
    let input = "/*.sid";
    let (base_path, file) = Config::split_file_path(input);

    assert_eq!(base_path, "/".to_string());
    assert_eq!(file, "*.sid".to_string());
}

#[test]
fn split_file_path_sub_path_directly() {
    let input = "test/*.sid";
    let (base_path, file) = Config::split_file_path(input);

    assert_eq!(base_path, "test".to_string());
    assert_eq!(file, "*.sid".to_string());
}

#[test]
fn split_file_path_sub_path_relative() {
    let input = "./test/*.sid";
    let (base_path, file) = Config::split_file_path(input);

    assert_eq!(base_path, "test".to_string());
    assert_eq!(file, "*.sid".to_string());
}

#[test]
fn split_file_path_multi_sub_path_relative() {
    let input = "./test/123/*.sid";
    let (base_path, file) = Config::split_file_path(input);

    assert_eq!(base_path, "test/123".to_string());
    assert_eq!(file, "*.sid".to_string());
}

#[test]
fn split_file_path_back_slash_forward_slash() {
    let input = ".\\test/*.sid";
    let (base_path, file) = Config::split_file_path(input);

    assert_eq!(base_path, "test".to_string());
    assert_eq!(file, "*.sid".to_string());
}

#[test]
fn split_file_path_forward_slash_back_slash() {
    let input = "./test\\*.sid";
    let (base_path, file) = Config::split_file_path(input);

    assert_eq!(base_path, "test".to_string());
    assert_eq!(file, "*.sid".to_string());
}

#[test]
fn split_file_path_empty() {
    let input = "";
    let (base_path, file) = Config::split_file_path(input);

    assert_eq!(base_path, ".".to_string());
    assert_eq!(file, "".to_string());
}

#[test]
fn split_file_path_forward_slash_only() {
    let input = "/";
    let (base_path, file) = Config::split_file_path(input);

    assert_eq!(base_path, "/".to_string());
    assert_eq!(file, "".to_string());
}

#[test]
fn split_file_path_current_dir() {
    let input = "./";
    let (base_path, file) = Config::split_file_path(input);

    assert_eq!(base_path, ".".to_string());
    assert_eq!(file, "".to_string());
}

#[test]
fn split_file_path_dot() {
    let input = ".";
    let (base_path, file) = Config::split_file_path(input);

    assert_eq!(base_path, ".".to_string());
    assert_eq!(file, ".".to_string());
}
