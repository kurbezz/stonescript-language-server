//! Test for em dash (—) character handling in various contexts

#[test]
fn test_em_dash_in_output() {
    let input = r#">`0,0,#00FF00,
^Double Mutagens Event available tomorrow"#;
    
    let result = stonescript_parser::parse_source(input);
    assert!(result.is_ok(), "Failed to parse output with text: {:?}", result.err());
}

#[test]
fn test_fullwidth_quotes() {
    // Testing fullwidth quotes used in KillerRPG
    let input = r#"var x = Sg(＂test＂,0)"#;
    
    let result = stonescript_parser::parse_source(input);
    assert!(result.is_ok(), "Failed to parse fullwidth quotes: {:?}", result.err());
}

#[test]
fn test_killerrpg_event_block() {
    // Exact pattern from KillerRPG around line 50-80
    let input = r#"var Td = time.Day
var number = 0.09999
number = Mr(number*10)/10
?Td > 13 & Td < 16
 number = 0.19999
 number = Mr(number*10)/10
:?Td = 6
 >`0,0,#00FF00,
 ^Double Mutagens Event available tomorrow

var Rt = ［ascii
.__________________________.
|—————————REWARDS——————————|
asciiend］"#;
    
    let result = stonescript_parser::parse_source(input);
    if let Err(ref e) = result {
        eprintln!("Error parsing KillerRPG event block: {}", e);
    }
    assert!(result.is_ok(), "Failed to parse KillerRPG event block");
}
