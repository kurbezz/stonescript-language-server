// Test reading Games/KillerRPG.txt directly

use std::fs;
use std::path::PathBuf;

#[test]
fn test_killerrpg_file() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../../test_scripts/Games/KillerRPG.txt");
    
    let input = fs::read_to_string(&path).expect("Failed to read file");
    
    let result = stonescript_parser::parse_source(&input);
    result.expect("Failed to parse Games/KillerRPG.txt");
}
