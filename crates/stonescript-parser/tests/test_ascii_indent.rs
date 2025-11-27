#[test]
fn test_ascii_multiline_in_conditional() {
    let input = "?true\n  >0,0,#white,ascii\n#\ntest\nasciiend\n";
    
    let result = stonescript_parser::parse_source(input);
    if let Err(e) = &result {
        println!("Error: {}", e);
    }
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}
