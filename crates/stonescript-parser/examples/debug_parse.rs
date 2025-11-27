use std::fs;
use stonescript_parser::parser::parse;

fn main() {
    let content = fs::read_to_string("test_scripts/repro.txt").expect("Failed to read file");
    match parse(&content) {
        Ok(program) => println!("Parsed successfully: {:?}", program),
        Err(e) => println!("Parse error: {}", e),
    }
}
