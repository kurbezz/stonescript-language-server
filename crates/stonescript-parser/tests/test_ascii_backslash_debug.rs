// Debug test to see what's happening with the backslash

#[test]
fn test_ascii_backslash_debug() {
    let input = r#">h-4,-3,ascii
#####\
###__/)
##(. _/
asciiend
"#;

    println!("Input:\n{}", input);
    println!("Input bytes: {:?}", input.as_bytes());
    
    let result = stonescript_parser::parse_source(input);
    if let Err(e) = result {
        println!("Error: {}", e);
        panic!("Failed to parse");
    }
}
