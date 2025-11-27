// Test FaceHUD content

#[test]
fn test_facehud_ascii() {
    let input = r#">`@screen.w -13@,1,ascii
# .'     `.
#/ __`-´__ \
#  <o\ /o>  
#   _____   
#\  `———´  /
# `._   _.´
asciiend
"#;
    println!("Input:\n{}", input);
    let result = stonescript_parser::parse_source(input);
    println!("Result: {:?}", result);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}
