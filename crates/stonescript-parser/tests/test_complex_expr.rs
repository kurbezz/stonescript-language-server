#[test]
fn test_complex_expression_in_var() {
    let input = "var x = _time % total_time < interval * 4";
    let result = stonescript_parser::parse_source(input);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}
