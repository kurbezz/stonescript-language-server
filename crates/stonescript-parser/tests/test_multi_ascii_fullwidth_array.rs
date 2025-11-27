// Minimal test case for the multi-ascii array with fullwidth brackets issue
// Based on Games/KillerRPG.txt line 88-110

#[test]
fn test_multi_ascii_array_fullwidth_closing_bracket() {
    let input = r#"var Rt = ［ascii
Test 1
asciiend,ascii
Test 2
asciiend］
"#;
    
    let result = stonescript_parser::parse_source(input);
    if let Err(e) = &result {
        eprintln!("Parse error: {}", e);
    }
    assert!(result.is_ok(), "Failed to parse multi-ascii array with fullwidth closing bracket");
}
