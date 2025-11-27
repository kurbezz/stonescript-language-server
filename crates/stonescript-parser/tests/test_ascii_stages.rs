// Test to understand exactly where parsing stops

#[test]
fn test_ascii_parsing_stages() {
    // Stage 1: Just the problematic line
    let input1 = r#">h-4,-3,ascii
##(. _/
asciiend
"#;
    println!("Stage 1 (just problematic line):");
    let result1 = stonescript_parser::parse_source(input1);
    println!("Result: {:?}\n", result1);
    
    // Stage 2: With a line before
    let input2 = r#">h-4,-3,ascii
#####\
##(. _/
asciiend
"#;
    println!("Stage 2 (with backslash line before):");
    let result2 = stonescript_parser::parse_source(input2);
    println!("Result: {:?}\n", result2);
    
    // Stage 3: Inside if block
    let input3 = r#"?x = 1
  >h-4,-3,ascii
##(. _/
asciiend
"#;
    println!("Stage 3 (inside if block):");
    let result3 = stonescript_parser::parse_source(input3);
    println!("Result: {:?}\n", result3);
}
