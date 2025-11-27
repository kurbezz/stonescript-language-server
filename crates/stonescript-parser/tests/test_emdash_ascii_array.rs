//! Specific test for em dash (—) in ASCII arrays

#[test]
fn test_single_emdash_in_ascii() {
    let input = r#"var x = [ascii
|—|
asciiend]"#;
    
    let result = stonescript_parser::parse_source(input);
    assert!(result.is_ok(), "Failed single em dash: {:?}", result.err());
}

#[test]
fn test_multiple_emdash_in_ascii() {
    let input = r#"var x = [ascii
|—————————|
asciiend]"#;
    
    let result = stonescript_parser::parse_source(input);
    assert!(result.is_ok(), "Failed multiple em dashes: {:?}", result.err());
}

#[test]
fn test_emdash_in_two_block_array() {
    let input = r#"var x = [ascii
|—|
asciiend,ascii
|X|
asciiend]"#;
    
    let result = stonescript_parser::parse_source(input);
    if let Err(ref e) = result {
        eprintln!("Error: {}", e);
    }
    assert!(result.is_ok(), "Failed em dash in two block array");
}

#[test]
fn test_emdash_with_multiple_lines_two_blocks() {
    let input = r#"var x = [ascii
AAA
|—————————|
BBB
asciiend,ascii
CCC
asciiend]"#;
    
    let result = stonescript_parser::parse_source(input);
    if let Err(ref e) = result {
        eprintln!("Error with multiple lines: {}", e);
    }
    assert!(result.is_ok(), "Failed em dash with multiple lines");
}
