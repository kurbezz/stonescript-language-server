// Test fullwidth quotes in function call

#[test]
fn test_fullwidth_quotes() {
    let input = r#"var BuffDCR = Sg(＂BuffDCR＂,0)"#;
    println!("Input hex: {:X?}", input.as_bytes());
    let result = stonescript_parser::parse_source(input);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}
