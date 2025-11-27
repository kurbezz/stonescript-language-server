//! Diagnostic tests for array/ASCII block interaction

#[test]
fn test_standalone_fullwidth_ascii() {
    // Standalone ASCII block with fullwidth brackets
    let input = r#"var x = ［ascii
AAA
asciiend］"#;
    
    let result = stonescript_parser::parse_source(input);
    assert!(result.is_ok(), "Standalone fullwidth ASCII failed: {:?}", result.err());
}

#[test]
fn test_array_with_normal_brackets() {
    // Array with normal brackets and ASCII blocks
    let input = r#"var x = [ascii
AAA
asciiend,ascii
BBB
asciiend]"#;
    
    let result = stonescript_parser::parse_source(input);
    assert!(result.is_ok(), "Normal bracket array failed: {:?}", result.err());
}

#[test]
fn test_array_with_fullwidth_brackets_no_inner_brackets() {
    // Fullwidth array brackets, but ASCII blocks without their own brackets
    let input = r#"var x = ［ascii
AAA
asciiend,ascii
BBB
asciiend］"#;
    
    let result = stonescript_parser::parse_source(input);
    if let Err(ref e) = result {
        eprintln!("Error: {}", e);
    }
    assert!(result.is_ok(), "Fullwidth array with ASCII (no inner brackets) failed");
}

#[test]
fn test_fullwidth_array_string_elements() {
    // Fullwidth array with regular string elements
    let input = r#"var x = ［"AAA","BBB"］"#;
    
    let result = stonescript_parser::parse_source(input);
    assert!(result.is_ok(), "Fullwidth array with strings failed: {:?}", result.err());
}

#[test]
fn test_comparison_normal_vs_fullwidth() {
    // Side by side: does fullwidth behave same as normal?
    let normal = r#"var x = [ascii
AAA
asciiend,ascii
BBB
asciiend]"#;
    
    let fullwidth = r#"var x = ［ascii
AAA
asciiend,ascii
BBB
asciiend］"#;
    
    let result_normal = stonescript_parser::parse_source(normal);
    let result_fullwidth = stonescript_parser::parse_source(fullwidth);
    
    assert!(result_normal.is_ok(), "Normal bracket version failed: {:?}", result_normal.err());
    if let Err(ref e) = result_fullwidth {
        eprintln!("Fullwidth version error: {}", e);
    }
    assert!(result_fullwidth.is_ok(), "Fullwidth bracket version failed (but normal passed!)");
}
