#[test]
fn test_unicode_block_characters() {
    let input = r#"var MAX_SOLID_BAR = ██████████████████████████████████████████████████
"#;
    
    let result = stonescript_parser::parse_source(input);
    assert!(result.is_ok(), "Failed to parse Unicode block characters: {:?}", result.err());
}

#[test]
fn test_unicode_underscore_bar() {
    let input = r#"var MAX_EMPTY_BAR = __________________________________________________
"#;
    
    let result = stonescript_parser::parse_source(input);
    assert!(result.is_ok(), "Failed to parse underscore bar: {:?}", result.err());
}

#[test]
fn test_array_with_unicode() {
    let input = r##"var colorList = [
  "#0A0000", "#040A04", "#000008"
]
"##;
    
    let result = stonescript_parser::parse_source(input);
    assert!(result.is_ok(), "Failed to parse array with color codes: {:?}", result.err());
}

#[test]
fn test_nested_array_with_identifiers() {
    let input = r#"var multiLineArray = [
  [ice crossbow],
  [aether crossbow],
  [trisk, star]
]
"#;
    
    let result = stonescript_parser::parse_source(input);
    assert!(result.is_ok(), "Failed to parse nested array with identifiers: {:?}", result.err());
}
