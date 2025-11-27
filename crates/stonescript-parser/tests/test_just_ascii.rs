// Test just the ASCII block parsing directly

#[test]
fn test_just_ascii_block() {
    let input = r#"ascii
#####\
###__/)
##(. _/
asciiend"#;

    let result = stonescript_parser::parse_source(input);
    println!("Result: {:?}", result);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}
