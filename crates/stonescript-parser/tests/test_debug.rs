#[test]
fn test_simple_var_debug() {
    // This should work - let's verify
    let result = stonescript_parser::parse_source("var x = 5");
    println!("Simple var result: {:?}", result);
    assert!(result.is_ok(), "Simple case failed: {:?}", result.err());
}

#[test]
fn test_var_with_comment_debug() {
    // This fails - let's see the exact error
    let result = stonescript_parser::parse_source("var x = 5 // comment");
    if let Err(e) = &result {
        println!("Error: {}", e);
        
        // Try parsing just "var x"
        let result2 = stonescript_parser::parse_source("var x");
        println!("Just 'var x': {:?}", result2);
    }
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}
