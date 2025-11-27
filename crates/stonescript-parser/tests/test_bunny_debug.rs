#[test]
fn test_bunny_file() {
    let input = std::fs::read_to_string(
        "/Users/kurbezz/Projects/stonescript/stone-script-lsp/test_scripts/Pets/Bunny.txt"
    ).unwrap();
    
    let result = stonescript_parser::parse_source(&input);
    if let Err(e) = &result {
        println!("Error: {}", e);
        // Print first 500 chars of remaining input
        if let Some(remaining) = e.strip_prefix("Failed to parse completely. Remaining: \"") {
            let end = remaining.find('\"').unwrap_or(500.min(remaining.len()));
            println!("Remaining (first 500 chars): {}", &remaining[..end.min(500)]);
        }
    }
    assert!(result.is_ok(), "Failed to parse Bunny.txt: {:?}", result.err());
}
