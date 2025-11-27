// Test >o syntax

#[test]
fn test_output_offset() {
    let input = r#">o 1,2,ascii
#
asciiend
"#;
    let result = stonescript_parser::parse_source(input);
    assert!(result.is_ok(), "Failed to parse >o: {:?}", result.err());
    
    let input2 = r#">o@x@,@y@,ascii
#
asciiend
"#;
    let result2 = stonescript_parser::parse_source(input2);
    assert!(result2.is_ok(), "Failed to parse >o with interpolation: {:?}", result2.err());
}
