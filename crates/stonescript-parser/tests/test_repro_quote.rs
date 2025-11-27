use stonescript_parser;

#[test]
fn test_repro_escaped_quote() {
    let source = r#"
func Main()
    char2 = draw.GetSymbol(x2, y2)
    ? char1 = "\"" | char1 = "^"
        x = 1
"#;
    let result = stonescript_parser::parse_source(source);
    if let Err(e) = result {
        panic!("Parse error: {}", e);
    }
}
