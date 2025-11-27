// Test Pets/Dog.txt failure block

#[test]
fn test_pets_dog_block() {
    let input = r#">o@myX@,@myZ@,@petColor@,ascii
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
