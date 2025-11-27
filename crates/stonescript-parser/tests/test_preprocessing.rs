#[test]
fn test_preprocessing() {
    let input = "var color = #66CC00     //headphones base color";
    let processed = stonescript_parser::parser::preprocess_line_continuations(input);
    println!("Original: {:?}", input);
    println!("Processed: {:?}", processed);
    assert_eq!(input, processed, "Preprocessing should not change this input");
}
