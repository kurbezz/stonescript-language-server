//! Test parsing first 120 lines of KillerRPG.txt

use std::fs;

#[test]
fn test_killerrpg_first_120_lines() {
    let input = fs::read_to_string("../../test_scripts/Games/KillerRPG.txt")
        .expect("Failed to read KillerRPG.txt");
    
    // Take first 120 lines
    let lines: Vec<&str> = input.lines().take(120).collect();
    let fragment = lines.join("\n");
    
    eprintln!("Fragment length: {} chars, {} lines", fragment.len(), lines.len());
    eprintln!("Last 200 chars: {:?}", &fragment[fragment.len().saturating_sub(200)..]);
    
    let result = stonescript_parser::parse_source(&fragment);
    if let Err(ref e) = result {
        eprintln!("Parse error: {}", e);
    }
    assert!(result.is_ok(), "Failed to parse first 120 lines of KillerRPG.txt");
}
