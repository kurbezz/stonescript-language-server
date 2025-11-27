// Test with ASCII blocks in nested ifs

#[test]
fn test_nested_if_with_ascii_and_outer_statement() {
    let input = r#"?bighead
  ?HP < 50
    >h-4,-3,ascii
###
asciiend
  ?HP > 100
    >h-4,-3,ascii
###
asciiend
HairFrame++
"#;

    let result = stonescript_parser::parse_source(input);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}
