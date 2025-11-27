// Test output with ASCII in an if block

#[test]
fn test_output_ascii_in_if() {
    let input = r#"?x = 1
  >h-4,-3,ascii
#####\
###__/)
##(. _/
asciiend
"#;

    let result = stonescript_parser::parse_source(input);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}
