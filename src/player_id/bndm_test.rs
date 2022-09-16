use super::*;

#[test]
fn find_pattern_in_middle() {
    let source = b"The quick brown fox jumps over the lazy dog";
    let pattern = b"jumps";
    let config = BndmConfig::new(pattern, None);
    let index = find_pattern(source, &config);

    assert_eq!(index, Some(20));
}

#[test]
fn find_pattern_at_beginning() {
    let source = b"The quick brown fox jumps over the lazy dog";
    let pattern = b"The quick";
    let config = BndmConfig::new(pattern, None);
    let index = find_pattern(source, &config);

    assert_eq!(index, Some(0));
}

#[test]
fn find_pattern_at_end() {
    let source = b"The quick brown fox jumps over the lazy dog";
    let pattern = b"dog";
    let config = BndmConfig::new(pattern, None);
    let index = find_pattern(source, &config);

    assert_eq!(index, Some(40));
}

#[test]
fn find_pattern_no_match() {
    let source = b"The quick brown fox jumps over the lazy dog";
    let pattern = b"cat";
    let config = BndmConfig::new(pattern, None);
    let index = find_pattern(source, &config);

    assert_eq!(index, None);
}

#[test]
fn find_pattern_fully() {
    let source = b"The quick brown fox jumps over the lazy dog";
    let pattern = b"The quick brown fox jumps over the lazy dog";
    let config = BndmConfig::new(pattern, None);
    let index = find_pattern(source, &config);

    assert_eq!(index, Some(0));
}

#[test]
fn find_pattern_partially_end() {
    let source = b"The quick brown fox jumps over the lazy dog";
    let pattern = b"test fox jumps over the lazy";
    let config = BndmConfig::new(pattern, None);
    let index = find_pattern(source, &config);

    assert_eq!(index, None);
}

#[test]
fn find_pattern_partially_beginning() {
    let source = b"The quick brown fox jumps over the lazy dog";
    let pattern = b"fox jumps over the test";
    let config = BndmConfig::new(pattern, None);
    let index = find_pattern(source, &config);

    assert_eq!(index, None);
}

#[test]
fn find_pattern_wildcard_middle() {
    let source = b"The quick brown fox jumps over the lazy dog";
    let pattern = b"j?mps";
    let config = BndmConfig::new(pattern, Some(b'?'));
    let index = find_pattern(source, &config);

    assert_eq!(index, Some(20));
}

#[test]
fn find_pattern_wildcard_end() {
    let source = b"The quick brown fox jumps over the lazy dog";
    let pattern = b"jump?";
    let config = BndmConfig::new(pattern, Some(b'?'));
    let index = find_pattern(source, &config);

    assert_eq!(index, Some(20));
}

#[test]
fn find_pattern_wildcard_beginning() {
    let source = b"The quick brown fox jumps over the lazy dog";
    let pattern = b"?umps";
    let config = BndmConfig::new(pattern, Some(b'?'));
    let index = find_pattern(source, &config);

    assert_eq!(index, Some(20));
}

#[test]
fn find_pattern_wildcard_multiple() {
    let source = b"The quick brown fox jumps over the lazy dog";
    let pattern = b"?um?s";
    let config = BndmConfig::new(pattern, Some(b'?'));
    let index = find_pattern(source, &config);

    assert_eq!(index, Some(20));
}

#[test]
fn find_pattern_wildcard_only() {
    let source = b"The quick brown fox jumps over the lazy dog";
    let pattern = b"??????";
    let config = BndmConfig::new(pattern, Some(b'?'));
    let index = find_pattern(source, &config);

    assert_eq!(index, Some(0));
}

#[test]
fn find_pattern_wildcard_beginning_first_word() {
    let source = b"The quick brown fox jumps over the lazy dog";
    let pattern = b"??he";
    let config = BndmConfig::new(pattern, Some(b'?'));
    let index = find_pattern(source, &config);

    assert_eq!(index, Some(30));
}

#[test]
fn find_pattern_match_1char() {
    let source = b"The quick brown fox jumps over the lazy dog";
    let pattern = b"q";
    let config = BndmConfig::new(pattern, None);
    let index = find_pattern(source, &config);

    assert_eq!(index, Some(4));
}

#[test]
fn find_pattern_match_0char() {
    let source = b"The quick brown fox jumps over the lazy dog";
    let pattern = b"";
    let config = BndmConfig::new(pattern, None);
    let index = find_pattern(source, &config);

    assert_eq!(index, None);
}

#[test]
fn find_pattern_match_2chars() {
    let source = b"The quick brown fox jumps over the lazy dog";
    let pattern = b"do";
    let config = BndmConfig::new(pattern, None);
    let index = find_pattern(source, &config);

    assert_eq!(index, Some(40));
}

#[test]
fn find_pattern_match_31chars() {
    let source = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua";
    let pattern = b"consectetur adipiscing elit, se";
    let config = BndmConfig::new(pattern, None);
    let index = find_pattern(source, &config);

    assert_eq!(index, Some(28));
}

#[test]
fn find_pattern_match_32chars() {
    let source = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua";
    let pattern = b"consectetur adipiscing elit, sed";
    let config = BndmConfig::new(pattern, None);
    let index = find_pattern(source, &config);

    assert_eq!(index, Some(28));
}

#[test]
fn find_pattern_match_33chars() {
    let source = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua";
    let pattern = b"consectetur adipiscing elit, sed ";
    let config = BndmConfig::new(pattern, None);
    let index = find_pattern(source, &config);

    assert_eq!(index, Some(28));
}

#[test]
fn find_pattern_match_63chars() {
    let source = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua";
    let pattern = b"consectetur adipiscing elit, sed do eiusmod tempor incididunt u";
    let config = BndmConfig::new(pattern, None);
    let index = find_pattern(source, &config);

    assert_eq!(index, Some(28));
}

#[test]
fn find_pattern_match_64chars() {
    let source = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua";
    let pattern = b"consectetur adipiscing elit, sed do eiusmod tempor incididunt ut";
    let config = BndmConfig::new(pattern, None);
    let index = find_pattern(source, &config);

    assert_eq!(index, Some(28));
}

#[test]
fn find_pattern_match_65chars_middle() {
    let source = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua";
    let pattern = b"consectetur adipiscing elit, sed do eiusmod tempor incididunt ut ";
    let config = BndmConfig::new(pattern, None);
    let index = find_pattern(source, &config);

    assert_eq!(index, Some(28));
}

#[test]
fn find_pattern_match_65chars_end() {
    let source = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua";
    let pattern = b"sed do eiusmod tempor incididunt ut labore et dolore magna aliqua";
    let config = BndmConfig::new(pattern, None);
    let index = find_pattern(source, &config);

    assert_eq!(index, Some(57));
}

#[test]
fn find_pattern_match_65chars_beginning() {
    let source = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua";
    let pattern = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do e";
    let config = BndmConfig::new(pattern, None);
    let index = find_pattern(source, &config);

    assert_eq!(index, Some(0));
}

#[test]
fn find_pattern_match_64chars_middle_wildcard_at_end() {
    let source = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua";
    let pattern = b"consectetur adipiscing elit, sed do eiusmod tempor incididunt u!";
    let config = BndmConfig::new(pattern, Some(b'!'));
    let index = find_pattern(source, &config);

    assert_eq!(index, Some(28));
}

#[test]
fn find_pattern_match_65chars_middle_wildcard_at_end() {
    let source = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua";
    let pattern = b"consectetur adipiscing elit, sed do eiusmod tempor incididunt ut!";
    let config = BndmConfig::new(pattern, Some(b'!'));
    let index = find_pattern(source, &config);

    assert_eq!(index, Some(28));
}

#[test]
fn find_pattern_match_65chars_middle_wildcard_second_last() {
    let source = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua";
    let pattern = b"consectetur adipiscing elit, sed do eiusmod tempor incididunt u! ";
    let config = BndmConfig::new(pattern, Some(b'!'));
    let index = find_pattern(source, &config);

    assert_eq!(index, Some(28));
}

#[test]
fn find_pattern_match_66chars_middle_wildcard_second_last() {
    let source = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua";
    let pattern = b"consectetur adipiscing elit, sed do eiusmod tempor incididunt ut!l";
    let config = BndmConfig::new(pattern, Some(b'!'));
    let index = find_pattern(source, &config);

    assert_eq!(index, Some(28));
}

#[test]
fn find_pattern_longer_than_source() {
    let source = b"Lorem ipsum dolor sit amet, consectetur";
    let pattern = b"consectetur adipiscing elit, sed do eiusmod tempor incididunt";
    let config = BndmConfig::new(pattern, None);
    let index = find_pattern(source, &config);

    assert_eq!(index, None);
}

#[test]
fn find_pattern_in_empty_source() {
    let source = b"";
    let pattern = b"consectetur adipiscing elit, sed do eiusmod tempor incididunt";
    let config = BndmConfig::new(pattern, None);
    let index = find_pattern(source, &config);

    assert_eq!(index, None);
}

#[test]
fn find_empty_pattern_in_empty_source() {
    let source = b"";
    let pattern = b"";
    let config = BndmConfig::new(pattern, None);
    let index = find_pattern(source, &config);

    assert_eq!(index, None);
}