#[test]
fn test_ascii_in_output() {
    let input = ">0,0,#white,ascii\ntest\nasciiend\n";
    
    let result = stonescript_parser::parse_source(input);
    if let Err(e) = &result {
        println!("Error: {}", e);
    }
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_ascii_with_interpolation() {
    let input = "var x = 5\n>@x@,0,#white,ascii\ntest\nasciiend\n";
    
    let result = stonescript_parser::parse_source(input);
    if let Err(e) = &result {
        println!("Error: {}", e);
    }
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}
