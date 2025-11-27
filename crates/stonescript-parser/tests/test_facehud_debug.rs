// Debug parsing line by line
// DISABLED: This test uses private ParseContext API that is not exposed anymore
/*
#[test]
fn test_facehud_debug() {
    let input = r#">`@screen.w -13@,1,ascii
# .'     `.
#/ __`-´__ \
#  <o\ /o>  
#   _____   
#\  `———´  /
# `._   _.´
asciiend
"#;

    let ctx = stonescript_parser::ParseContext::new(input);
    let mut remaining = input;
    
    loop {
        println!("Parsing line starting with: {:?}", remaining.lines().next());
        match stonescript_parser::statement(remaining, &ctx) {
            Ok((next, stmt)) => {
                println!("Parsed statement: {:?}", stmt);
                remaining = next;
                
                // Consume newlines
                while remaining.starts_with('\n') || remaining.starts_with('\r') {
                    remaining = &remaining[1..];
                }
                
                if remaining.is_empty() {
                    break;
                }
            }
            Err(e) => {
                println!("Failed to parse statement: {:?}", e);
                break;
            }
        }
    }
}
*/
