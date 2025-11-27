// Debug KillerRPG content

use std::fs;
use std::path::PathBuf;

#[test]
fn debug_killerrpg_hex() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../../test_scripts/Games/KillerRPG.txt");
    
    let content = fs::read_to_string(&path).expect("Failed to read file");
    
    // Find the line with St(＂BuffDCR＂,BuffDCR)
    for (i, line) in content.lines().enumerate() {
        if line.contains("St(＂BuffDCR＂,BuffDCR)") {
            println!("Line {}: {}", i + 1, line);
            println!("Hex: {:X?}", line.as_bytes());
        }
    }
}
