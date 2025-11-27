// Test ASCII block with backslash at end of line

#[test]
fn test_ascii_with_trailing_backslash() {
    let input = r#">h-4,-3,ascii
#####\
###__/)
##(. _/
asciiend
x = 1
"#;

    let result = stonescript_parser::parse_source(input);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}
