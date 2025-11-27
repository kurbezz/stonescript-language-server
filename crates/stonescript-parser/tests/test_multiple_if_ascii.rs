// Test case: Multiple consecutive if blocks with ASCII at same level
// Issue: Parser fails when there are multiple if statements at the same
// indentation level, each containing ASCII blocks, followed by a statement
// at the parent level

#[test]
fn test_multiple_if_ascii_blocks() {
    let input = r#"var HairFrame

?bighead
  ?HP < (maxHP / 2)
    HairFrame = -1
    >h-4,-3,#FFFFFF,ascii
########
###)`''(
##(. _-`
asciiend

  ?HairFrame >= 0 & HairFrame < 2
    >h-4,-3,#FFFFFF,ascii
#####.-
###_/(
##(. _)
asciiend

  ?(HairFrame > 1 & HairFrame <= 3) | (HairFrame > 9 & HairFrame <= 11)
    >h-4,-3,#FFFFFF,ascii
####.-
###_)\
##(. _)
asciiend

  ?(HairFrame > 3 & HairFrame <= 5) | (HairFrame > 7 & HairFrame <= 9)
    >h-4,-3,#FFFFFF,ascii
####(
###_)\
##(. _)
asciiend

  ?(HairFrame > 5 & HairFrame <= 7)
    >h-4,-3,#FFFFFF,ascii
#####\
###__/)
##(. _/
asciiend
HairFrame++

?HairFrame = 12
  HairFrame = 0
"#;

    let result = stonescript_parser::parse_source(input);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}
