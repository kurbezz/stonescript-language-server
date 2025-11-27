// Direct test of var_assignment parser function

use stonescript_parser::parser::*;

#[test]
fn test_var_assignment_with_comment_direct() {
    // Can't access internal parser functions directly
    // So just run through parse_source
    let tests = vec![
        ("var x", true),
        ("var x = 5", true),
        ("var x // c", true),
        ("var x = 5//c", true),  // No space before comment  
        ("var x = 5 //c", true), // One space before comment
        ("var x = 5  //c", true), // Two spaces before comment
    ];
    
    for (input, should_pass) in tests {
        let result = stonescript_parser::parse_source(input);
        if should_pass {
            assert!(result.is_ok(), "Failed to parse '{}': {:?}", input, result.err());
        } else {
            assert!(result.is_err(), "Should have failed to parse '{}'", input);
        }
    }
}
