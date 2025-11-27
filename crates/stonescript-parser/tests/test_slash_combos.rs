// Test different character combinations before slash

#[test]
fn test_slash_combinations() {
    // Test 1: Just slash
    let input1 = r#">ascii
##/
asciiend
"#;
    println!("Test 1 (just slash):");
    let result1 = stonescript_parser::parse_source(input1);
    println!("Result: {:?}\n", result1);
    assert!(result1.is_ok(), "Failed: {:?}", result1.err());
    
    // Test 2: Space then slash
    let input2 = r#">ascii
## /
asciiend
"#;
    println!("Test 2 (space slash):");
    let result2 = stonescript_parser::parse_source(input2);
    println!("Result: {:?}\n", result2);
    assert!(result2.is_ok(), "Failed: {:?}", result2.err());
    
    // Test 3: Underscore then slash
    let input3 = r#">ascii
##_/
asciiend
"#;
    println!("Test 3 (underscore slash):");
    let result3 = stonescript_parser::parse_source(input3);
    println!("Result: {:?}\n", result3);
    assert!(result3.is_ok(), "Failed: {:?}", result3.err());
    
    // Test 4: Dot space underscore slash (the actual problematic pattern)
    let input4 = r#">ascii
##(. _/
asciiend
"#;
    println!("Test 4 (dot space underscore slash):");
    let result4 = stonescript_parser::parse_source(input4);
    println!("Result: {:?}\n", result4);
    assert!(result4.is_ok(), "Failed: {:?}", result4.err());
}
