/// Tests for preprocessing issues found during testing
/// These tests are isolated to track specific problems

#[test]
fn test_preprocess_preserves_trailing_newline_lf() {
    let input = "line1\nline2\n";
    let processed = stonescript_parser::parser::preprocess_line_continuations(input);
    assert_eq!(processed, "line1\nline2\n", "Should preserve trailing LF");
}

#[test]
fn test_preprocess_preserves_trailing_newline_crlf() {
    let input = "line1\r\nline2\r\n";
    let processed = stonescript_parser::parser::preprocess_line_continuations(input);
    assert_eq!(
        processed, "line1\r\nline2\r\n",
        "Should preserve trailing CRLF"
    );
}

#[test]
fn test_preprocess_no_trailing_newline() {
    let input = "line1\nline2";
    let processed = stonescript_parser::parser::preprocess_line_continuations(input);
    assert_eq!(
        processed, "line1\nline2",
        "Should not add trailing newline when not present"
    );
}

#[test]
fn test_preprocess_single_line_with_newline() {
    let input = "single line\n";
    let processed = stonescript_parser::parser::preprocess_line_continuations(input);
    assert_eq!(
        processed, "single line\n",
        "Should preserve trailing newline on single line"
    );
}

#[test]
fn test_preprocess_single_line_without_newline() {
    let input = "single line";
    let processed = stonescript_parser::parser::preprocess_line_continuations(input);
    assert_eq!(
        processed, "single line",
        "Should not add newline to single line without one"
    );
}

#[test]
fn test_preprocess_continuation_with_trailing_newline() {
    let input = "line1\n^continued\n";
    let processed = stonescript_parser::parser::preprocess_line_continuations(input);
    assert_eq!(
        processed, "line1continued\n",
        "Should handle continuation and preserve trailing newline"
    );
}

#[test]
fn test_preprocess_empty_string() {
    let input = "";
    let processed = stonescript_parser::parser::preprocess_line_continuations(input);
    assert_eq!(processed, "", "Empty string should remain empty");
}

#[test]
fn test_preprocess_only_newline() {
    let input = "\n";
    let processed = stonescript_parser::parser::preprocess_line_continuations(input);
    assert_eq!(processed, "\n", "Single newline should be preserved");
}
