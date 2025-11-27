//! Test color literal parsing

#[test]
fn test_color_hex() {
    let result = stonescript_parser::parse_source("var color = #66CC00");
    assert!(result.is_ok(), "Failed to parse color hex literal: {:?}", result.err());
}

#[test]
fn test_color_named() {
    let result = stonescript_parser::parse_source("var color = #white");
    assert!(result.is_ok(), "Failed to parse named color literal: {:?}", result.err());
}

#[test]
fn test_color_with_comment() {
    let result = stonescript_parser::parse_source("var color = #66CC00  // comment");
    assert!(result.is_ok(), "Failed to parse color literal with comment: {:?}", result.err());
}
