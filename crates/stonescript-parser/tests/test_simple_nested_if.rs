// Simpler test to understand the indentation issue

#[test]
fn test_simple_nested_if_with_outer_statement() {
    let input = r#"?bighead
  ?HP < 50
    x = 1
  ?HP > 100
    y = 2
HairFrame++
"#;

    let result = stonescript_parser::parse_source(input);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}
