#[test]
fn test_elseif_newline_spaces() {
    let input = "?x\n  >1\n:?y\n  >2\n  "; // Newline then spaces
    let result = stonescript_parser::parse_source(input);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}
