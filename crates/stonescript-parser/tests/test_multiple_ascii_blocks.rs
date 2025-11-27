//! Test for parsing multiple consecutive ASCII blocks

use stonescript_parser::parse_source;

#[test]
fn test_multiple_ascii_blocks_in_sequence() {
    let input = r#"
//First block
>`10,10,ascii
# Test 1
# Line 2
asciiend

//Second block
>`20,20,ascii
# Test 2
# Line 2
asciiend

//Third block
>`30,30,ascii
# Test 3
# Line 3
asciiend
"#;

    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Should parse multiple ASCII blocks: {:?}",
        result.err()
    );
}

#[test]
fn test_ascii_blocks_in_conditionals() {
    let input = r#"
?state = 1
  >`10,10,ascii
# Block 1
asciiend

?state = 2
  >`20,20,ascii
# Block 2
asciiend
"#;

    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Should parse ASCII blocks in conditionals: {:?}",
        result.err()
    );
}

#[test]
fn test_ascii_blocks_with_complex_content() {
    let input = r#"
?fstate = neutral
  >`@screen.w -13@,1,ascii
# .'     `.
#/ __   __ \
#  <o) (o>
#
#\   ———   /
# `._   _.´
asciiend

?fstate = scowl
  >`@screen.w -13@,1,ascii
# .'     `.
#/ ._`-´_. \
#  <o\ /o>
#    ___
#\  ´———`  /
# `._   _.´
asciiend
"#;

    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Should parse ASCII blocks with Unicode content: {:?}",
        result.err()
    );
}

#[test]
fn test_ascii_blocks_with_colors() {
    let input = r#"
>`10,10,#red,ascii
# Red text
asciiend

>`20,20,#blue,ascii
# Blue text
asciiend
"#;

    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Should parse ASCII blocks with color parameters: {:?}",
        result.err()
    );
}

#[test]
fn test_nested_ascii_blocks_in_if_elseif() {
    let input = r#"
?time % 4 <= 1
  >o@myX@,@myZ@,ascii
# Frame 1
asciiend
:?time % 4 > 1
  >o@myX@,@myZ@,ascii
# Frame 2
asciiend
"#;

    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Should parse ASCII blocks in if-elseif chains: {:?}",
        result.err()
    );
}

#[test]
fn test_ascii_blocks_after_other_statements() {
    let input = r#"
x = 10
y = 20

>`@x@,@y@,ascii
# Block
asciiend

z = 30
"#;

    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Should parse ASCII blocks mixed with other statements: {:?}",
        result.err()
    );
}

#[test]
fn test_ascii_block_with_backslash() {
    let input = r#"
>`10,10,ascii
# Line with \ backslash
# Another \ here
asciiend
"#;

    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Should parse ASCII block with backslash: {:?}",
        result.err()
    );
}

#[test]
fn test_ascii_block_with_special_chars() {
    // Test from actual FaceHUD.txt
    let input = r#"
?fstate = neutral
  >`10,1,ascii
# .'     `.
#/ __   __ \
#  <o) (o>
asciiend
"#;

    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Should parse ASCII block with backslash at line end: {:?}",
        result.err()
    );
}

#[test]
fn test_two_ascii_blocks_with_backslashes() {
    let input = r#"
?state = 1
  >`10,1,ascii
#/ __   __ \
#  <o) (o>
asciiend

?state = 2
  >`10,1,ascii
#/ ._`-´_. \
#  <o\ /o>
asciiend
"#;

    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Should parse two ASCII blocks with backslashes: {:?}",
        result.err()
    );
}
