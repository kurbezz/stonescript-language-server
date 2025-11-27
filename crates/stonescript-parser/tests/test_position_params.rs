// Test if position parameters cause the issue

#[test]
fn test_position_params_with_backslash() {
    // Test 1: With position params, backslash line
    let input1 = r#">h-4,-3,ascii
#####\
##_/
asciiend
"#;
    println!("Test 1 (with position, backslash):");
    let result1 = stonescript_parser::parse_source(input1);
    println!("Result: {:?}\n", result1);
    assert!(result1.is_ok(), "Failed: {:?}", result1.err());
    
    // Test 2: Without position params, backslash line
    let input2 = r#">ascii
#####\
##_/
asciiend
"#;
    println!("Test 2 (no position, backslash):");
    let result2 = stonescript_parser::parse_source(input2);
    println!("Result: {:?}\n", result2);
    assert!(result2.is_ok(), "Failed: {:?}", result2.err());
}
