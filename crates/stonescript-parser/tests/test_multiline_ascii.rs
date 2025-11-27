// Test multi-line ASCII blocks

#[test]
fn test_multiline_ascii() {
    // Test 1: Two lines, second has _/
    let input1 = r#">ascii
###
##_/
asciiend
"#;
    println!("Test 1 (two lines, second has _/):");
    let result1 = stonescript_parser::parse_source(input1);
    println!("Result: {:?}\n", result1);
    assert!(result1.is_ok(), "Failed: {:?}", result1.err());
    
    // Test 2: Three lines, last has _/
    let input2 = r#">ascii
###
###
##_/
asciiend
"#;
    println!("Test 2 (three lines, last has _/):");
    let result2 = stonescript_parser::parse_source(input2);
    println!("Result: {:?}\n", result2);
    assert!(result2.is_ok(), "Failed: {:?}", result2.err());
    
    // Test 3: The exact failing pattern from the test
    let input3 = r#">h-4,-3,ascii
#####\
###__/)
##(. _/
asciiend
"#;
    println!("Test 3 (exact failing pattern):");
    let result3 = stonescript_parser::parse_source(input3);
    println!("Result: {:?}\n", result3);
    assert!(result3.is_ok(), "Failed: {:?}", result3.err());
}
