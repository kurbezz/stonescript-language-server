#[test]
fn test_escaped_quote_in_string() {
    let input = r#"var char1 = "\""
"#;
    
    let result = stonescript_parser::parse_source(input);
    assert!(result.is_ok(), "Failed to parse escaped quote: {:?}", result.err());
}

#[test]
fn test_conditional_with_escaped_quote() {
    let input = r#"? char1 = "\"" | char1 = "^"
    return true
"#;
    
    let result = stonescript_parser::parse_source(input);
    assert!(result.is_ok(), "Failed to parse conditional with escaped quote: {:?}", result.err());
}

#[test]
fn test_backslash_in_string() {
    let input = r#"var path = "C:\folder\file.txt"
"#;
    
    let result = stonescript_parser::parse_source(input);
    assert!(result.is_ok(), "Failed to parse backslash in string: {:?}", result.err());
}
