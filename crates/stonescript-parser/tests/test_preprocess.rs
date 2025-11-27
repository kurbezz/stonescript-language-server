#[test]
fn test_preprocess_crlf() {
    let input = "line1\r\nline2\r\n";
    let processed = stonescript_parser::parser::preprocess_line_continuations(input);
    assert_eq!(processed, "line1\r\nline2\r\n");
}
