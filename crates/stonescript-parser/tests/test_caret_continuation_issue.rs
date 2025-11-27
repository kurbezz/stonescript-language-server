//! Test for line continuation with caret (^) issue
//!
//! Issue: Parser fails on multi-line function calls using ^ at start of continuation line
//! Found in: UI/ScrollText.txt and many other files
//!
//! Example failing code:
//! var result=string.IndexOf(input,
//! ^                       " ",next)

use stonescript_parser::parse_source;

#[test]
fn test_caret_continuation_simple() {
    let source = r#"var result = string.IndexOf(input,
^                       " ",next)
x = 1"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse simple caret continuation: {:?}",
        result.err()
    );
}

#[test]
fn test_caret_continuation_in_assignment() {
    let source = r#"var x = SomeFunction(arg1,
^                    arg2,
^                    arg3)
y = 1"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse caret continuation in assignment: {:?}",
        result.err()
    );
}

#[test]
fn test_caret_continuation_with_string() {
    let source = r#"tempArr = string.Split(temp,
^                       true)
x = 1"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse caret continuation with string: {:?}",
        result.err()
    );
}

#[test]
fn test_caret_continuation_return_statement() {
    let source = r#"func Test()
    return string.IndexOf(input,
^            tempArr[0],next)"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse caret continuation in return: {:?}",
        result.err()
    );
}

#[test]
fn test_caret_continuation_nested_calls() {
    let source = r#"var result = Outer(Inner(a,
^                          b),
^                   c)
x = 1"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse nested caret continuation: {:?}",
        result.err()
    );
}

#[test]
fn test_caret_continuation_in_condition() {
    let source = r#"? SomeFunction(arg1,
^              arg2) > 0
    x = 1"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse caret continuation in condition: {:?}",
        result.err()
    );
}

#[test]
fn test_caret_with_leading_spaces() {
    let source = r#"var x = func(a,
^        b,
^        c)
y = 1"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse caret with leading spaces: {:?}",
        result.err()
    );
}

#[test]
fn test_scroll_text_pattern() {
    // Simplified version of the actual failing pattern from ScrollText.txt
    let source = r#"func SetNewTarget(smartScroll, instance, input, inputSize, len)
    var temp = ""
    var tempArr = []
    var result = string.IndexOf(input,
^                       " ",next)
    ? result = -1
        return 0
    ? result <= next + 2
        temp = string.Sub(input,result)
        tempArr = string.Split(temp,
^                       true)
        ? tempArr.Count() = 0
            return 0
        :
            return string.IndexOf(input,
^            tempArr[0],next)

    return next"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse ScrollText pattern: {:?}",
        result.err()
    );
}

#[test]
fn test_caret_continuation_with_expression() {
    let source = r#"var output = string.Sub(input, indexST[instance],
^                        len-temp) + string.Sub(input,0,temp)
x = 1"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse caret continuation with expression: {:?}",
        result.err()
    );
}

#[test]
fn test_caret_in_function_call_chain() {
    let source = r#"var x = obj.Method1(a,
^                   b).Method2(c,
^                             d)
y = 1"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse caret in method chain: {:?}",
        result.err()
    );
}

#[test]
fn test_caret_with_mixed_indentation() {
    let source = r#"func Test()
    var result = LongFunctionName(param1,
^                                 param2,
^                                 param3)
    return result"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse caret with mixed indentation: {:?}",
        result.err()
    );
}

#[test]
fn test_caret_continuation_array_access() {
    let source = r#"var value = array[CalculateIndex(x,
^                                  y,
^                                  z)]
result = value"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse caret continuation with array access: {:?}",
        result.err()
    );
}

#[test]
fn test_caret_continuation_output_statement() {
    let source = r#">FormatOutput(value1,
^            value2,
^            value3)
x = 1"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse caret continuation in output: {:?}",
        result.err()
    );
}

#[test]
fn test_multiple_caret_continuations() {
    let source = r#"func Complex()
    var a = Func1(x,
^                 y)
    var b = Func2(p,
^                 q)
    var c = Func3(m,
^                 n)
    return a + b + c"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse multiple caret continuations: {:?}",
        result.err()
    );
}

#[test]
fn test_caret_with_string_literal() {
    let source = r#"var text = string.Format("Hello {0}",
^                          name)
x = 1"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse caret with string literal: {:?}",
        result.err()
    );
}

#[test]
fn test_caret_in_comparison() {
    let source = r#"? string.IndexOf(text,
^                 " ",
^                 pos) = goal
    x = 1"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse caret in comparison: {:?}",
        result.err()
    );
}
