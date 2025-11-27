// Test Pets/Dog.txt failure block with CRLF

#[test]
fn test_pets_dog_crlf() {
    let input = ">o@myX@,@myZ@,@petColor@,ascii\r\n#\r\n#,\r\n('-._######;(,|\r\n `'´ `-.__.' ''-,\r\n####/       `--´\r\n#,-´_.´`.__ !\r\n#'´'#####, `.`. \r\n#########`-' `-'\r\nasciiend\r\n";
    let result = stonescript_parser::parse_source(input);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}
