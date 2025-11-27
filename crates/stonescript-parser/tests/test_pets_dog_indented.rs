// Test Pets/Dog.txt failure block with indentation

#[test]
fn test_pets_dog_indented() {
    let input = r#"?condition
  :?stateTime/2 = 4
    >o@myX@,@myZ@,@petColor@,ascii
#
#,
('-._######;(,|
 `'´ `-.__.' ''-,
####/       `--´
#,-´_.´`.__ !
#'´'#####, `.`.
#########`-' `-'
asciiend
"#;
    let result = stonescript_parser::parse_source(input);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}
