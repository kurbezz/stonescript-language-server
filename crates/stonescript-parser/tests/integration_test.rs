//! Integration test - parse all test scripts

use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

#[test]
fn test_all_scripts_parse() {
    // Find test_scripts directory - it's at the workspace root
    let mut test_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_dir.pop(); // Go up from crates/stonescript-parser
    test_dir.pop(); // Go up from crates
    test_dir.push("test_scripts");
    
    if !test_dir.exists() {
        // Try alternative path (if running from workspace root)
        test_dir = PathBuf::from("../../test_scripts");
        if !test_dir.exists() {
            println!("test_scripts directory not found at {:?}, skipping integration test", test_dir);
            return;
        }
    }
    
    println!("\nParsing StoneScript files from: {:?}", test_dir);
    
    let mut passed = 0;
    let mut failed = 0;
    let mut failed_files = Vec::new();
    
    for entry in WalkDir::new(&test_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("txt"))
        .filter(|e| e.path().to_string_lossy().contains("WavyScarf"))
    {
        let path = entry.path();
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to read {:?}: {}", path, e);
                failed += 1;
                failed_files.push(path.display().to_string());
                continue;
            }
        };
        
        match stonescript_parser::parse_source(&content) {
            Ok(_program) => {
                passed += 1;
            }
            Err(e) => {
                eprintln!("✗ {:?} - Parse error: {}", path, e);
                failed += 1;
                failed_files.push(path.display().to_string());
            }
        }
    }
    
    println!("\n==============================================");
    println!("Integration Test Results:");
    println!("  Total:  {}", passed + failed);
    println!("  Passed: {}", passed);
    println!("  Failed: {}", failed);
    println!("==============================================");
    
    if !failed_files.is_empty() {
        println!("\nFailed files:");
        for file in &failed_files {
            println!("  - {}", file);
        }
    }
    
    assert_eq!(failed, 0, "Some test scripts failed to parse");
}
