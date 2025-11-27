// Test with actual content from KillerRPG.txt
use std::fs;

#[test]
fn test_killerrpg_actual_snippet() {
    // Read the actual snippet from the file
    let input = fs::read_to_string("/tmp/killerrpg_snippet.txt")
        .expect("Failed to read snippet");
    
    let result = stonescript_parser::parse_source(&input);
    if let Err(e) = &result {
        eprintln!("Parse error: {}", e);
        eprintln!("Input length: {} bytes", input.len());
        
        // Find where it failed
        if let Some(remaining) = e.to_string().split("Remaining: \"").nth(1) {
            let preview = &remaining.chars().take(100).collect::<String>();
            eprintln!("First 100 chars of remaining: {}", preview);
        }
    }
    assert!(result.is_ok(), "Failed to parse KillerRPG snippet");
}
