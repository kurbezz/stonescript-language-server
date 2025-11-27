#[test]
fn test_exact_headphones_line() {
    // This is the exact line from Headphones.txt line 10
    let result = stonescript_parser::parse_source("var color = #66CC00     //headphones base color");
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}
