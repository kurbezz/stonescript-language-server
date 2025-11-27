// Test reading Pets/Dog.txt directly

use std::fs;
use std::path::PathBuf;

#[test]
fn test_pets_dog_file() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../../test_scripts/Pets/Dog.txt");
    
    let input = fs::read_to_string(&path).expect("Failed to read file");
    
    let result = stonescript_parser::parse_source(&input);
    if let Err(e) = &result {
        println!("Parse error: {:?}", e);
    }
    assert!(result.is_ok(), "Failed to parse Pets/Dog.txt");
}
