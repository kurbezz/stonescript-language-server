// Test case: Statement after asciiend at same indentation level
// Issue: Parser fails to parse statements that come after asciiend
// when they are at the same indentation level within an if block

#[test]
fn test_statement_after_asciiend() {
    let input = r#"var HairFrame

?loc.begin 
  HairFrame = 0

?bighead
  ?HP < (maxHP / 2)
    HairFrame = -1
    >h-4,-3,#FFFFFF,ascii
########
###)`''(
##(. _-`
asciiend
HairFrame++

?HairFrame = 12
  HairFrame = 0
"#;

    let result = stonescript_parser::parse_source(input);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

