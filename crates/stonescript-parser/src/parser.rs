//! Parser for StoneScript using nom

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while, take_while1, take_while_m_n},
    character::complete::{alpha1, alphanumeric1, anychar, char, digit1, line_ending, multispace0},
    combinator::{map, not, opt, peek, recognize, value, verify},
    multi::{many0, many1, separated_list0, separated_list1},
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult,
};

use crate::ast::*;

/// Context for tracking source positions
#[derive(Debug, Clone)]
struct ParseContext<'a> {
    source: &'a str,
}

impl<'a> ParseContext<'a> {
    fn new(source: &'a str) -> Self {
        Self { source }
    }

    /// Calculate position from byte offset
    fn position_at(&self, offset: usize) -> Position {
        let text = &self.source[..offset.min(self.source.len())];
        let line = text.matches('\n').count();
        let column = text.lines().last().map(|l| l.len()).unwrap_or(0);
        Position::new(line, column)
    }

    /// Create span from two offsets
    fn make_span(&self, start: usize, end: usize) -> Span {
        Span::new(self.position_at(start), self.position_at(end))
    }

    /// Get current offset
    fn offset(&self, remaining: &str) -> usize {
        remaining.as_ptr() as usize - self.source.as_ptr() as usize
    }
}

/// Parse whitespace (spaces and tabs, but not newlines)
fn ws(input: &str) -> IResult<&str, &str> {
    take_while(|c| c == ' ' || c == '\t')(input)
}

/// Parse optional whitespace
fn ws0(input: &str) -> IResult<&str, &str> {
    ws(input)
}

/// Parse required whitespace
fn ws1(input: &str) -> IResult<&str, &str> {
    take_while1(|c| c == ' ' || c == '\t')(input)
}

/// Parse optional whitespace including newlines (multispace)
fn ws_multi(input: &str) -> IResult<&str, &str> {
    take_while(|c: char| c.is_whitespace())(input)
}

/// Parse a line comment starting with //
fn line_comment(input: &str) -> IResult<&str, &str> {
    preceded(tag("//"), take_while(|c| c != '\n' && c != '\r'))(input)
}

/// Parse a block comment /* */
fn block_comment(input: &str) -> IResult<&str, &str> {
    delimited(tag("/*"), take_until("*/"), tag("*/"))(input)
}

/// Parse any comment
fn comment<'a>(input: &'a str, ctx: &ParseContext<'a>) -> IResult<&'a str, Statement> {
    let start = ctx.offset(input);
    let (input, s) = alt((line_comment, block_comment))(input)?;
    let end = ctx.offset(input);
    let span = ctx.make_span(start, end);
    Ok((input, Statement::Comment(s.to_string(), span)))
}

/// Parse an identifier (variable name, property name, etc.)
fn identifier(input: &str) -> IResult<&str, String> {
    map(
        recognize(pair(
            alt((alpha1, tag("_"))),
            many0(alt((alphanumeric1, tag("_")))),
        )),
        |s: &str| s.to_string(),
    )(input)
}

/// Parse a bare string (unquoted string that may contain Unicode)
/// Used for assignments like: var MAX_SOLID_BAR = ██████████████
fn bare_string<'a>(input: &'a str, ctx: &ParseContext<'a>) -> IResult<&'a str, Expression> {
    let start = ctx.offset(input);

    // Capture everything until whitespace or newline
    // For bare Unicode strings, we want to capture the whole sequence
    let mut end_pos = 0;
    let chars: Vec<char> = input.chars().collect();

    // First, check if this looks like it should NOT be a bare string
    // by checking the first character
    if chars.is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    let first_char = chars[0];

    // Don't capture if it starts with ASCII alphanumeric or underscore
    // (should be identifier) or if it starts with a digit (should be number)
    if first_char.is_ascii_alphanumeric() || first_char == '_' {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Capture until whitespace or newline for Unicode strings
    while end_pos < chars.len() {
        let ch = chars[end_pos];
        if ch.is_whitespace() || ch == '\r' || ch == '\n' {
            break;
        }
        end_pos += 1;
    }

    if end_pos == 0 {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Convert character count to byte offset
    let byte_offset: usize = chars.iter().take(end_pos).map(|c| c.len_utf8()).sum();
    let content = &input[..byte_offset];
    let remaining = &input[byte_offset..];

    let end = ctx.offset(remaining);
    Ok((
        remaining,
        Expression::String(content.to_string(), ctx.make_span(start, end)),
    ))
}

/// Parse a path string (for new/import): Games/Fishing/FishingGame
fn path_string(input: &str) -> IResult<&str, String> {
    recognize(separated_list1(
        char('/'),
        recognize(pair(
            alt((alpha1, tag("_"))),
            many0(alt((alphanumeric1, tag("_")))),
        )),
    ))(input)
    .map(|(i, s)| (i, s.to_string()))
}

/// Parse new expression: new Games/Fishing/FishingGame
fn new_expression<'a>(input: &'a str, ctx: &ParseContext<'a>) -> IResult<&'a str, Expression> {
    let start = ctx.offset(input);
    let (input, _) = tag("new")(input)?;
    let (input, _) = ws1(input)?;
    let (input, path) = path_string(input)?;
    let end = ctx.offset(input);
    Ok((
        input,
        Expression::New {
            path,
            span: ctx.make_span(start, end),
        },
    ))
}

/// Parse import statement: import Cosmetics/TrainAdventure/Main
fn import_statement<'a>(input: &'a str, ctx: &ParseContext<'a>) -> IResult<&'a str, Statement> {
    let start = ctx.offset(input);
    let (input, _) = tag("import")(input)?;
    let (input, _) = ws1(input)?;
    let (input, path) = path_string(input)?;
    let end = ctx.offset(input);
    Ok((
        input,
        Statement::Import {
            path,
            span: ctx.make_span(start, end),
        },
    ))
}

/// Parse an integer literal
fn integer<'a>(input: &'a str, ctx: &ParseContext<'a>) -> IResult<&'a str, Expression> {
    let start = ctx.offset(input);
    let (input, s) = recognize(pair(opt(char('-')), digit1))(input)?;
    let end = ctx.offset(input);
    let span = ctx.make_span(start, end);
    Ok((input, Expression::Integer(s.parse().unwrap(), span)))
}

/// Parse a float literal
fn float<'a>(input: &'a str, ctx: &ParseContext<'a>) -> IResult<&'a str, Expression> {
    let start = ctx.offset(input);
    let (input, s) = recognize(tuple((opt(char('-')), digit1, char('.'), digit1)))(input)?;
    let end = ctx.offset(input);
    let span = ctx.make_span(start, end);
    Ok((input, Expression::Float(s.parse().unwrap(), span)))
}

/// Parse a number (float or integer)
fn number<'a>(input: &'a str, ctx: &ParseContext<'a>) -> IResult<&'a str, Expression> {
    alt((|i| float(i, ctx), |i| integer(i, ctx)))(input)
}

/// Parse binary operator
fn binary_operator(input: &str) -> IResult<&str, BinaryOperator> {
    alt((
        value(BinaryOperator::GreaterEqual, tag(">=")),
        value(BinaryOperator::LessEqual, tag("<=")),
        value(BinaryOperator::NotEqual, tag("!=")),
        // StoneScript also supports spaced '!' as NotEqual (e.g., "index ! -1")
        value(BinaryOperator::NotEqual, char('!')),
        value(BinaryOperator::Equal, char('=')),
        value(BinaryOperator::Greater, char('>')),
        value(BinaryOperator::Less, char('<')),
        value(BinaryOperator::And, char('&')),
        value(BinaryOperator::Or, char('|')),
        value(BinaryOperator::Add, char('+')),
        value(BinaryOperator::Subtract, char('-')),
        value(BinaryOperator::Multiply, char('*')),
        value(
            BinaryOperator::Divide,
            terminated(char('/'), not(char('/'))),
        ),
        value(BinaryOperator::Modulo, char('%')),
    ))(input)
}

/// Parse assignment operator (=, +=, -=, *=, /=, %=)
fn assignment_operator(input: &str) -> IResult<&str, AssignmentOperator> {
    use crate::ast::AssignmentOperator;
    alt((
        value(AssignmentOperator::AddAssign, tag("+=")),
        value(AssignmentOperator::SubtractAssign, tag("-=")),
        value(AssignmentOperator::MultiplyAssign, tag("*=")),
        value(AssignmentOperator::DivideAssign, tag("/=")),
        value(AssignmentOperator::ModuloAssign, tag("%=")),
        value(AssignmentOperator::Assign, char('=')),
    ))(input)
}

/// Parse an lvalue expression (valid on left side of assignment)
/// Only allows: identifier, property access, index access (no function calls)
fn lvalue_expression<'a>(input: &'a str, ctx: &ParseContext<'a>) -> IResult<&'a str, Expression> {
    let start = ctx.offset(input);
    let (mut input, mut expr) = {
        let s = ctx.offset(input);
        let (i, id) = identifier(input)?;
        let e = ctx.offset(i);
        (i, Expression::Identifier(id, ctx.make_span(s, e)))
    };

    loop {
        let original_input = input;

        // Try property access (.property)
        if let Ok((next_input, _)) = char::<_, nom::error::Error<&str>>('.')(input) {
            if let Ok((next_input, prop)) = identifier(next_input) {
                let end = ctx.offset(next_input);
                expr = Expression::Property {
                    object: Box::new(expr),
                    property: prop,
                    span: ctx.make_span(start, end),
                };
                input = next_input;
                continue;
            }
        }

        // Try index access ([index])
        if let Ok((next_input, _)) = char::<_, nom::error::Error<&str>>('[')(input) {
            if let Ok((next_input, _)) = ws0(next_input) {
                if let Ok((next_input, index_expr)) = expression(next_input, ctx) {
                    if let Ok((next_input, _)) = ws0(next_input) {
                        if let Ok((next_input, _)) =
                            char::<_, nom::error::Error<&str>>(']')(next_input)
                        {
                            let end = ctx.offset(next_input);
                            expr = Expression::IndexAccess {
                                object: Box::new(expr),
                                index: Box::new(index_expr),
                                span: ctx.make_span(start, end),
                            };
                            input = next_input;
                            continue;
                        }
                    }
                }
            }
        }

        // No more lvalue postfix operations (stop before function calls)
        if input == original_input {
            break;
        }
    }

    Ok((input, expr))
}

/// Parse boolean literal (true or false)
fn boolean_literal<'a>(input: &'a str, ctx: &ParseContext<'a>) -> IResult<&'a str, Expression> {
    let start = ctx.offset(input);

    // Try "true"
    if let Ok((rest, _)) = tag::<_, _, nom::error::Error<&str>>("true")(input) {
        let end = ctx.offset(rest);
        return Ok((rest, Expression::Boolean(true, ctx.make_span(start, end))));
    }

    // Try "false"
    if let Ok((rest, _)) = tag::<_, _, nom::error::Error<&str>>("false")(input) {
        let end = ctx.offset(rest);
        return Ok((rest, Expression::Boolean(false, ctx.make_span(start, end))));
    }

    Err(nom::Err::Error(nom::error::Error::new(
        input,
        nom::error::ErrorKind::Tag,
    )))
}

/// Parse property access (e.g., loc.stars, foe.hp)
/// Parse a base expression (identifier, number, or parenthesized expression)
fn base_expression<'a>(input: &'a str, ctx: &ParseContext<'a>) -> IResult<&'a str, Expression> {
    alt((
        |i| quoted_string(i, ctx),
        |i| color_literal(i, ctx),
        |i| ascii_block(i, ctx),
        |i| array_literal(i, ctx),
        |i| new_expression(i, ctx),
        |i| boolean_literal(i, ctx),
        |i| interpolated_expression(i, ctx),
        |i| {
            let start = ctx.offset(i);
            let (i, id) = identifier(i)?;
            let end = ctx.offset(i);
            Ok((i, Expression::Identifier(id, ctx.make_span(start, end))))
        },
        |i| number(i, ctx),
        // Parenthesized expression
        |i| {
            let (i, _) = char('(')(i)?;
            let (i, _) = ws0(i)?;
            let (i, expr) = expression(i, ctx)?;
            let (i, _) = ws0(i)?;
            let (i, _) = char(')')(i)?;
            Ok((i, expr))
        },
        // Bare string as fallback (for Unicode strings like ████████)
        |i| bare_string(i, ctx),
    ))(input)
}

/// Parse interpolated expression (@expr@)
fn interpolated_expression<'a>(
    input: &'a str,
    ctx: &ParseContext<'a>,
) -> IResult<&'a str, Expression> {
    let (input, _) = char('@')(input)?;
    let (input, expr) = expression(input, ctx)?;
    let (input, _) = char('@')(input)?;
    Ok((input, expr))
}

/// Parse postfix operations (property access, index access, function calls)
fn postfix_expression<'a>(input: &'a str, ctx: &ParseContext<'a>) -> IResult<&'a str, Expression> {
    let start = ctx.offset(input);
    let (mut input, mut expr) = base_expression(input, ctx)?;

    loop {
        let original_input = input;

        // Try property access (.property)
        if let Ok((next_input, _)) = char::<_, nom::error::Error<&str>>('.')(input) {
            if let Ok((next_input, prop)) = identifier(next_input) {
                let end = ctx.offset(next_input);
                expr = Expression::Property {
                    object: Box::new(expr),
                    property: prop,
                    span: ctx.make_span(start, end),
                };
                input = next_input;
                continue;
            }
        }

        // Try index access ([index])
        if let Ok((next_input, _)) = char::<_, nom::error::Error<&str>>('[')(input) {
            if let Ok((next_input, _)) = ws_multi(next_input) {
                if let Ok((next_input, index_expr)) = expression(next_input, ctx) {
                    if let Ok((next_input, _)) = ws_multi(next_input) {
                        if let Ok((next_input, _)) =
                            char::<_, nom::error::Error<&str>>(']')(next_input)
                        {
                            let end = ctx.offset(next_input);
                            expr = Expression::IndexAccess {
                                object: Box::new(expr),
                                index: Box::new(index_expr),
                                span: ctx.make_span(start, end),
                            };
                            input = next_input;
                            continue;
                        }
                    }
                }
            }
        }

        // Try function call (arguments)
        if let Ok((next_input, _)) = char::<_, nom::error::Error<&str>>('(')(input) {
            if let Ok((next_input, _)) = ws_multi(next_input) {
                if let Ok((next_input, args)) =
                    separated_list0(delimited(ws_multi, char(','), ws_multi), |i| {
                        expression(i, ctx)
                    })(next_input)
                {
                    if let Ok((next_input, _)) = ws_multi(next_input) {
                        if let Ok((next_input, _)) =
                            char::<_, nom::error::Error<&str>>(')')(next_input)
                        {
                            let end = ctx.offset(next_input);
                            expr = Expression::FunctionCall {
                                function: Box::new(expr),
                                args,
                                span: ctx.make_span(start, end),
                            };
                            input = next_input;
                            continue;
                        }
                    }
                }
            }
        }

        // No more postfix operations
        if input == original_input {
            break;
        }
    }

    Ok((input, expr))
}

/// Deprecated: kept for backward compatibility, now just calls postfix_expression
fn property_access<'a>(input: &'a str, ctx: &ParseContext<'a>) -> IResult<&'a str, Expression> {
    postfix_expression(input, ctx)
}

/// Deprecated: kept for backward compatibility, now just calls postfix_expression
fn function_call<'a>(input: &'a str, ctx: &ParseContext<'a>) -> IResult<&'a str, Expression> {
    postfix_expression(input, ctx)
}

/// Parse quoted string literal
fn quoted_string<'a>(input: &'a str, ctx: &ParseContext<'a>) -> IResult<&'a str, Expression> {
    let start = ctx.offset(input);

    // Check for either ASCII quote (") or fullwidth quote (＂)
    let quote_char = if input.starts_with('"') {
        '"'
    } else if input.starts_with('＂') {
        '＂'
    } else {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Char,
        )));
    };

    let input = &input[quote_char.len_utf8()..];

    let mut content = String::new();
    let mut remaining = input;

    loop {
        if remaining.is_empty() {
            return Err(nom::Err::Error(nom::error::Error::new(
                remaining,
                nom::error::ErrorKind::Tag,
            )));
        }

        if remaining.starts_with(quote_char) {
            let rest = &remaining[quote_char.len_utf8()..];
            let end = ctx.offset(rest);
            return Ok((rest, Expression::String(content, ctx.make_span(start, end))));
        }

        // Handle escape sequences
        if remaining.starts_with('\\') && remaining.len() > 1 {
            let next_char = remaining.chars().nth(1).unwrap();
            match next_char {
                'n' => content.push('\n'),
                'r' => content.push('\r'),
                't' => content.push('\t'),
                '\\' => content.push('\\'),
                '"' => content.push('"'),
                '＂' => content.push('＂'),
                _ => {
                    content.push('\\');
                    content.push(next_char);
                }
            }
            remaining = &remaining[1 + next_char.len_utf8()..];
        } else {
            let ch = remaining.chars().next().unwrap();
            content.push(ch);
            remaining = &remaining[ch.len_utf8()..];
        }
    }
}

/// Parse ascii block (multiline string)
/// Supports both regular and fullwidth brackets: ascii...asciiend or ［ascii...asciiend］
fn ascii_block<'a>(input: &'a str, ctx: &ParseContext<'a>) -> IResult<&'a str, Expression> {
    let start = ctx.offset(input);

    // Check if this starts with 'ascii' keyword (not consuming any brackets)
    let (input, _) = tag("ascii")(input)?;

    // Allow optional line ending after 'ascii' keyword
    let (input, _) = opt(line_ending)(input)?;
    let (input, content) = take_until("asciiend")(input)?;
    let (input, _) = tag("asciiend")(input)?;

    // Don't consume any closing bracket here - let the caller handle brackets
    // This allows ascii blocks to work both standalone and inside arrays

    let end = ctx.offset(input);
    Ok((
        input,
        Expression::String(content.to_string(), ctx.make_span(start, end)),
    ))
}

/// Parse color literal (e.g., #white, #FF00FF, #123abc)
fn color_literal<'a>(input: &'a str, ctx: &ParseContext<'a>) -> IResult<&'a str, Expression> {
    let start = ctx.offset(input);
    let (input, _) = char('#')(input)?;

    // Try to parse hex color (6 or 3 digits) or named color
    let (input, color_value) = alt((
        // Hex colors: #RRGGBB or #RGB
        recognize(tuple((take_while_m_n(6, 6, |c: char| {
            c.is_ascii_hexdigit()
        }),))),
        recognize(tuple((take_while_m_n(3, 3, |c: char| {
            c.is_ascii_hexdigit()
        }),))),
        // Named colors (alphabetic)
        recognize(alpha1),
    ))(input)?;

    let end = ctx.offset(input);
    let color_string = format!("#{}", color_value);
    Ok((
        input,
        Expression::String(color_string, ctx.make_span(start, end)),
    ))
}

/// Parse array literal (e.g., [], [1, 2, 3])
/// Supports both regular and fullwidth brackets: [...] or ［...］
fn array_literal<'a>(input: &'a str, ctx: &ParseContext<'a>) -> IResult<&'a str, Expression> {
    let start = ctx.offset(input);

    // Check for fullwidth opening bracket ［ or regular [
    let has_fullwidth_open = input.starts_with('［');
    let input = if has_fullwidth_open {
        &input['［'.len_utf8()..]
    } else if input.starts_with('[') {
        &input[1..]
    } else {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Char,
        )));
    };

    let (input, _) = ws_multi(input)?;

    let (input, elements) = separated_list0(
        |i| {
            let (i, _) = ws_multi(i)?;
            let (i, _) = char(',')(i)?;
            let (i, _) = ws_multi(i)?;
            Ok((i, ()))
        },
        |i| expression(i, ctx),
    )(input)?;

    let (input, _) = ws_multi(input)?;

    // Check for fullwidth closing bracket ］ or regular ]
    let input = if input.starts_with('］') {
        &input['］'.len_utf8()..]
    } else if input.starts_with(']') {
        &input[1..]
    } else {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Char,
        )));
    };

    let end = ctx.offset(input);

    Ok((
        input,
        Expression::Array {
            elements,
            span: ctx.make_span(start, end),
        },
    ))
}

/// Parse a primary expression (now uses postfix_expression)
fn primary_expression<'a>(input: &'a str, ctx: &ParseContext<'a>) -> IResult<&'a str, Expression> {
    postfix_expression(input, ctx)
}

/// Parse unary expression
fn unary_expression<'a>(input: &'a str, ctx: &ParseContext<'a>) -> IResult<&'a str, Expression> {
    alt((
        // Prefix unary (!, -)
        |i| {
            let start = ctx.offset(i);
            let (i, op) = alt((
                value(UnaryOperator::Not, char('!')),
                value(UnaryOperator::Negate, char('-')),
            ))(i)?;
            // Allow whitespace after unary operator
            let (i, _) = ws0(i)?;
            // Parse another unary expression to support !! and other combinations
            let (i, expr) = unary_expression(i, ctx)?;
            let end = ctx.offset(i);
            Ok((
                i,
                Expression::UnaryOp {
                    op,
                    operand: Box::new(expr),
                    span: ctx.make_span(start, end),
                },
            ))
        },
        // Postfix unary (++, --)
        |i| {
            let start = ctx.offset(i);
            let (i, expr) = primary_expression(i, ctx)?;
            let (i, op) = alt((
                value(UnaryOperator::Increment, tag("++")),
                value(UnaryOperator::Decrement, tag("--")),
            ))(i)?;
            let end = ctx.offset(i);
            Ok((
                i,
                Expression::UnaryOp {
                    op,
                    operand: Box::new(expr),
                    span: ctx.make_span(start, end),
                },
            ))
        },
        |i| primary_expression(i, ctx),
    ))(input)
}

/// Parse a simple expression (no binary operators yet)
fn simple_expression<'a>(input: &'a str, ctx: &ParseContext<'a>) -> IResult<&'a str, Expression> {
    unary_expression(input, ctx)
}

/// Parse binary expression
/// Parse binary expression (handles chained operators like a + b + c)
fn binary_expression<'a>(input: &'a str, ctx: &ParseContext<'a>) -> IResult<&'a str, Expression> {
    let start = ctx.offset(input);
    let (mut input, mut left) = simple_expression(input, ctx)?;

    loop {
        let (next_input, _) = ws0(input)?;

        if let Ok((next_input, op)) = binary_operator(next_input) {
            let (next_input, _) = ws0(next_input)?;
            let (next_input, right) = simple_expression(next_input, ctx)?;
            let end = ctx.offset(next_input);

            left = Expression::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span: ctx.make_span(start, end),
            };
            input = next_input;
        } else {
            break;
        }
    }

    Ok((input, left))
}

/// Parse any expression
fn expression<'a>(input: &'a str, ctx: &ParseContext<'a>) -> IResult<&'a str, Expression> {
    binary_expression(input, ctx)
}

/// Parse string with interpolation (@variable@)
fn interpolated_string<'a>(input: &'a str, ctx: &ParseContext<'a>) -> IResult<&'a str, Expression> {
    let start = ctx.offset(input);
    let mut parts = Vec::new();
    let mut remaining = input;
    let mut current_text = String::new();

    loop {
        if remaining.is_empty() || remaining.starts_with('\n') || remaining.starts_with('\r') {
            if !current_text.is_empty() {
                let text_end = ctx.offset(remaining);
                let text_start = text_end - current_text.len();
                parts.push(InterpolationPart::Text(
                    current_text,
                    ctx.make_span(text_start, text_end),
                ));
            }
            break;
        }

        if remaining.starts_with('@') {
            // Save accumulated text
            if !current_text.is_empty() {
                let text_end = ctx.offset(remaining);
                let text_start = text_end - current_text.len();
                parts.push(InterpolationPart::Text(
                    current_text.clone(),
                    ctx.make_span(text_start, text_end),
                ));
                current_text.clear();
            }

            // Skip first @
            remaining = &remaining[1..];

            // Parse expression until next @
            if let Some(end_pos) = remaining.find('@') {
                let expr_str = &remaining[..end_pos];
                if let Ok((_, expr)) = expression(expr_str, ctx) {
                    parts.push(InterpolationPart::Expression(Box::new(expr)));
                }
                remaining = &remaining[end_pos + 1..];
            } else {
                break;
            }
        } else {
            let c = remaining.chars().next().unwrap();
            current_text.push(c);
            remaining = &remaining[c.len_utf8()..];
        }
    }

    if parts.is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    let end = ctx.offset(remaining);
    let consumed_len = input.len() - remaining.len();
    Ok((
        &input[consumed_len..],
        Expression::Interpolation(parts, ctx.make_span(start, end)),
    ))
}

/// Parse output statement (>)
fn output_statement<'a>(input: &'a str, ctx: &ParseContext<'a>) -> IResult<&'a str, Statement> {
    let start = ctx.offset(input);
    let (input, _) = char('>')(input)?;
    // Optional 'o' prefix (e.g. >o x,y,ascii)
    let (input, _) = opt(char('o'))(input)?;
    let (input, _) = ws0(input)?;

    // Unified position parsing:
    // Supports:
    // >x,y,ascii
    // >`x,y,ascii
    // >x,y,color,ascii
    // >`x,y,color,ascii
    // >`x,y,text

    let (input, position) = opt(|input| {
        // Optional backtick
        let (input, _) = opt(char('`'))(input)?;

        // Parse x
        let (input, x) = expression(input, ctx)?;
        let (input, _) = tuple((ws0, char(','), ws0))(input)?;

        // Parse y
        let (input, y) = expression(input, ctx)?;
        let (input, _) = tuple((ws0, char(','), ws0))(input)?;

        // Check for optional color
        let peek = input.trim_start();
        let (input, _) = if peek.starts_with('#') || peek.starts_with('@') {
            // Parse color
            let (input, _) = expression(input, ctx)?;
            let (input, _) = tuple((ws0, char(','), ws0))(input)?;
            (input, ())
        } else {
            (input, ())
        };

        Ok((input, (x, y)))
    })(input)?;

    let (input, _) = ws0(input)?;

    // Try to parse ASCII block first, then fall back to interpolated string
    let (input, text) = alt((|i| ascii_block(i, ctx), |i| interpolated_string(i, ctx)))(input)?;

    let end = ctx.offset(input);
    Ok((
        input,
        Statement::Output {
            position,
            text,
            span: ctx.make_span(start, end),
        },
    ))
}

/// Parse variable declaration or assignment
fn var_assignment<'a>(input: &'a str, ctx: &'a ParseContext<'a>) -> IResult<&'a str, Statement> {
    let start = ctx.offset(input);
    let (input, _) = tag("var")(input)?;
    let (input, _) = ws1(input)?;
    let (input, name) = identifier(input)?;
    let (input, _) = ws0(input)?;

    // Check if we have an assignment
    if let Ok((next_input, _)) = char::<_, nom::error::Error<&str>>('=')(input) {
        // We have an equals sign, so we MUST parse an expression
        let (input, _) = ws_multi(next_input)?;
        match expression(input, ctx) {
            Ok((input, value)) => {
                let end = ctx.offset(input);
                let name_span = ctx.make_span(start, ctx.offset(&input) - value.span().end.column);
                Ok((
                    input,
                    Statement::Assignment {
                        target: Expression::Identifier(name.clone(), name_span),
                        op: AssignmentOperator::Assign,
                        value,
                        span: ctx.make_span(start, end),
                    },
                ))
            }
            Err(e) => Err(e),
        }
    } else {
        // No assignment, just declaration
        let end = ctx.offset(input);
        let name_span = ctx.make_span(start, end);
        Ok((
            input,
            Statement::Assignment {
                target: Expression::Identifier(name.clone(), name_span),
                op: AssignmentOperator::Assign,
                value: Expression::Integer(0, ctx.make_span(end, end)), // Default value
                span: ctx.make_span(start, end),
            },
        ))
    }
}

/// Parse variable assignment
fn assignment_statement<'a>(input: &'a str, ctx: &ParseContext<'a>) -> IResult<&'a str, Statement> {
    let start = ctx.offset(input);

    // Parse left side as lvalue (identifier, property access, or index access only)
    let (input, target) = lvalue_expression(input, ctx)?;

    // Validate that target is a valid lvalue
    match &target {
        Expression::Identifier(_, _)
        | Expression::Property { .. }
        | Expression::IndexAccess { .. } => {
            // Valid lvalue, continue
        }
        _ => {
            // Invalid lvalue
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag,
            )));
        }
    }

    let (input, _) = ws0(input)?;
    let (input, op) = assignment_operator(input)?;
    let (input, _) = ws_multi(input)?;
    let (input, value) = expression(input, ctx)?;

    let end = ctx.offset(input);
    Ok((
        input,
        Statement::Assignment {
            target,
            op,
            value,
            span: ctx.make_span(start, end),
        },
    ))
}

/// Parse return statement
fn return_statement<'a>(input: &'a str, ctx: &ParseContext<'a>) -> IResult<&'a str, Statement> {
    let start = ctx.offset(input);
    let (input, _) = tag("return")(input)?;

    // Try to parse return value (requires whitespace before value if present)
    let (input, value) = opt(preceded(ws1, |i| expression(i, ctx)))(input)?;

    let end = ctx.offset(input);
    Ok((
        input,
        Statement::Return {
            value,
            span: ctx.make_span(start, end),
        },
    ))
}

/// Parse for loop: for i = start..end OR for e : collection
fn for_loop<'a>(input: &'a str, ctx: &'a ParseContext<'a>) -> IResult<&'a str, Statement> {
    let start = ctx.offset(input);
    let (input, _) = tag("for")(input)?;
    let (input, _) = ws1(input)?;
    let (input, variable) = identifier(input)?;
    let (input, _) = ws0(input)?;

    // Check if it's a range-based loop (=) or collection-based loop (:)
    let (input, separator) = alt((char('='), char(':')))(input)?;
    let (input, _) = ws0(input)?;

    if separator == '=' {
        // Range-based for loop: for i = start..end
        // Parse start expression
        let (input, start_expr) = expression(input, ctx)?;
        let (input, _) = ws0(input)?;
        let (input, _) = tag("..")(input)?;
        let (input, _) = ws0(input)?;
        // Parse end expression
        let (input, end_expr) = expression(input, ctx)?;

        let (input, _) = opt(line_ending)(input)?;

        // Parse body
        let (input, body) = indented_block(input, ctx)?;

        let end = ctx.offset(input);
        Ok((
            input,
            Statement::For {
                variable,
                range: (start_expr, end_expr),
                body,
                span: ctx.make_span(start, end),
            },
        ))
    } else {
        // Collection-based for loop: for e : collection
        let (input, collection) = expression(input, ctx)?;

        let (input, _) = opt(line_ending)(input)?;

        // Parse body
        let (input, body) = indented_block(input, ctx)?;

        let end = ctx.offset(input);
        Ok((
            input,
            Statement::ForIn {
                variable,
                collection,
                body,
                span: ctx.make_span(start, end),
            },
        ))
    }
}

/// Parse function definition: func Name(arg1, arg2)
fn function_definition<'a>(
    input: &'a str,
    ctx: &'a ParseContext<'a>,
) -> IResult<&'a str, Statement> {
    let start = ctx.offset(input);
    let (input, _) = tag("func")(input)?;
    let (input, _) = ws1(input)?;
    let (input, name) = identifier(input)?;
    let (input, _) = ws0(input)?;
    let (input, _) = char('(')(input)?;
    let (input, _) = ws0(input)?;

    // Parse parameters
    let (input, params) = separated_list0(delimited(ws0, char(','), ws0), identifier)(input)?;

    let (input, _) = ws0(input)?;
    let (input, _) = char(')')(input)?;
    let (input, _) = opt(line_ending)(input)?;

    // Parse body
    let (input, body) = indented_block(input, ctx)?;

    let end = ctx.offset(input);
    Ok((
        input,
        Statement::FunctionDefinition {
            name,
            params,
            body,
            span: ctx.make_span(start, end),
        },
    ))
}

/// Parse expression as statement (for function calls, property access, etc.)
fn expression_statement<'a>(input: &'a str, ctx: &ParseContext<'a>) -> IResult<&'a str, Statement> {
    let start = ctx.offset(input);
    let (input, expr) = expression(input, ctx)?;
    let end = ctx.offset(input);
    Ok((
        input,
        Statement::ExpressionStatement {
            expression: expr,
            span: ctx.make_span(start, end),
        },
    ))
}

/// Parse command statement (e.g., equip, loadout, activate)
fn command_statement<'a>(input: &'a str, ctx: &ParseContext<'a>) -> IResult<&'a str, Statement> {
    let start = ctx.offset(input);
    let (input, name) = identifier(input)?;
    let (input, _) = ws1(input)?;
    let (input, args) = separated_list0(ws1, |i| simple_expression(i, ctx))(input)?;

    let end = ctx.offset(input);
    Ok((
        input,
        Statement::Command {
            name,
            args,
            span: ctx.make_span(start, end),
        },
    ))
}

/// Parse a single line statement (without indentation handling)
fn single_statement<'a>(input: &'a str, ctx: &'a ParseContext<'a>) -> IResult<&'a str, Statement> {
    preceded(
        ws0,
        alt((
            |i| import_statement(i, ctx),
            |i| comment(i, ctx),
            |i| return_statement(i, ctx),
            |i| output_statement(i, ctx),
            map(line_ending, |_| Statement::Empty),
            // Try var assignment first (var x = value)
            |i| var_assignment(i, ctx),
            // Try regular assignment
            |i| assignment_statement(i, ctx),
            |i| command_statement(i, ctx),
            // Try expression statement last (e.g., function calls)
            |i| expression_statement(i, ctx),
        )),
    )(input)
}

/// Parse indented block of statements
fn indented_block<'a>(
    input: &'a str,
    ctx: &'a ParseContext<'a>,
) -> IResult<&'a str, Vec<Statement>> {
    let mut statements = Vec::new();
    let mut remaining = input;
    let mut base_indent: Option<usize> = None;

    loop {
        // Skip empty lines (including lines with only whitespace)
        loop {
            // Count leading spaces/tabs on this line
            let spaces_count = remaining
                .chars()
                .take_while(|c| *c == ' ' || *c == '\t')
                .count();
            let after_spaces = &remaining[spaces_count..];

            // Check if this is an empty line (only whitespace before line ending)
            if after_spaces.starts_with('\n') || after_spaces.starts_with('\r') {
                // Skip the entire line (whitespace + line ending)
                let (next, _) = line_ending(after_spaces)?;
                remaining = next;
            } else {
                // Not an empty line, exit the loop
                break;
            }
        }

        if remaining.is_empty() {
            break;
        }

        // Remember position before processing this line (after empty lines)
        let line_start = remaining;

        // Count current line indentation
        let current_indent = remaining
            .chars()
            .take_while(|c| *c == ' ' || *c == '\t')
            .count();

        // Check if we have content on this line
        let after_spaces = &remaining[current_indent..];

        if after_spaces.is_empty()
            || after_spaces.starts_with('\n')
            || after_spaces.starts_with('\r')
        {
            // Empty line, skip it
            if let Ok((next, _)) = line_ending::<_, nom::error::Error<&str>>(remaining) {
                remaining = next;
                continue;
            } else {
                // EOF with spaces (or just EOF)
                // We are done with the block.
                break;
            }
        }

        // Set base indentation from first non-empty line
        if base_indent.is_none() {
            if current_indent == 0 {
                // No indentation, empty block - return to line start
                return Ok((line_start, statements));
            }
            base_indent = Some(current_indent);
        }

        // If indentation decreased below base level, we're done with this block
        if let Some(base) = base_indent {
            if current_indent < base {
                // Return to the start of this line (before indentation)
                return Ok((line_start, statements));
            }
        }

        // Try to parse a statement
        match statement(after_spaces, ctx) {
            Ok((next, stmt)) => {
                let is_block = matches!(
                    stmt,
                    Statement::FunctionDefinition { .. }
                        | Statement::For { .. }
                        | Statement::ForIn { .. }
                        | Statement::Condition { .. }
                );

                statements.push(stmt);

                let mut current_input = next;
                if !is_block {
                    // Consume trailing comment and line ending
                    // This is needed because some statements (like assignment) don't consume the newline
                    // and we need to be ready for the next line in the block
                    let (n, _) = trailing_comment(current_input).unwrap_or((current_input, ()));
                    let (n, _) =
                        opt(line_ending::<_, nom::error::Error<&str>>)(n).unwrap_or((n, None));
                    current_input = n;
                }

                remaining = current_input;
            }
            Err(_e) => {
                // If we can't parse a statement, return to line start
                return Ok((line_start, statements));
            }
        }
    }

    Ok((remaining, statements))
}

/// Parse conditional statement with indented block
fn condition_statement<'a>(
    input: &'a str,
    ctx: &'a ParseContext<'a>,
) -> IResult<&'a str, Statement> {
    let start = ctx.offset(input);

    // Calculate the indentation of this condition by looking backwards to find line start
    let condition_indent = {
        let offset_before_question = ctx.offset(input);
        let text_before = &ctx.source[..offset_before_question];

        // Find the last newline before the '?'
        if let Some(last_newline_pos) = text_before.rfind(|c| c == '\n' || c == '\r') {
            // Count spaces/tabs between the newline and the '?'
            let line_start = last_newline_pos + 1;
            let between = &ctx.source[line_start..offset_before_question];
            between
                .chars()
                .take_while(|c| *c == ' ' || *c == '\t')
                .count()
        } else {
            // No previous newline, so we're at the start of the file
            // Count spaces/tabs from the beginning
            text_before
                .chars()
                .take_while(|c| *c == ' ' || *c == '\t')
                .count()
        }
    };

    let (input, _) = char('?')(input)?;
    let (input, _) = ws0(input)?;
    let (input, condition) = expression(input, ctx)?;
    let (input, _) = opt(line_ending)(input)?;

    // Parse then block (indented statements)
    let (input, then_block) = indented_block(input, ctx)?;

    // Parse else-if branches
    let (input, else_ifs) = many0(|i| {
        let start = ctx.offset(i);
        eprintln!("[else-if] Trying to parse else-if at offset {}", start);
        let (i, _) = opt(line_ending)(i)?;
        let (i, _) = ws0(i)?;
        let (i, _) = tag(":?")(i)?;
        eprintln!("[else-if] Found :? at offset {}", ctx.offset(i));
        let (i, _) = ws0(i)?;
        let (i, cond) = expression(i, ctx)?;
        eprintln!("[else-if] Parsed condition");
        let (i, _) = opt(line_ending)(i)?;
        let (i, block) = indented_block(i, ctx)?;
        eprintln!("[else-if] Parsed block");
        let end = ctx.offset(i);
        Ok((
            i,
            ElseIf {
                condition: cond,
                block,
                span: ctx.make_span(start, end),
            },
        ))
    })(input)?;

    // Parse else block
    // After parsing then_block, we might have line endings and indentation before ':'
    // IMPORTANT: Only consume ':' if it's at the same indentation level as the original '?'
    let (final_input, else_block) = {
        // Try to parse else block
        // Skip any line endings
        // Skip any line endings
        let maybe_after_newlines = many0(line_ending::<_, nom::error::Error<&str>>)(input);
        if let Ok((i, _)) = maybe_after_newlines {
            // Count the indentation before ':'
            let colon_indent = i.chars().take_while(|c| *c == ' ' || *c == '\t').count();

            // Only proceed if the indentation matches the condition's indentation
            // We allow the else block to be indented same or more than the condition
            // (e.g. aligned with then block), but not less (which would mean it belongs to an outer block)
            if colon_indent >= condition_indent {
                // Skip whitespace (indentation)
                let maybe_after_ws = ws0(i);
                if let Ok((i, _)) = maybe_after_ws {
                    // Try to match ':'
                    if let Ok((i, _)) = char::<_, nom::error::Error<&str>>(':')(i) {
                        // Check if it's :? (else-if) or plain : (else)
                        let peek_result = ws0(i);
                        if let Ok((peek_i, _)) = peek_result {
                            if peek_i.starts_with('?') {
                                // This is :?, not a plain else - don't consume
                                (input, None)
                            } else {
                                // Found plain else (:)
                                // Skip optional line ending after :
                                let (i, _) = opt(line_ending::<_, nom::error::Error<&str>>)(i)
                                    .unwrap_or((i, None));
                                // Parse else block body
                                if let Ok((i, block)) = indented_block(i, ctx) {
                                    (i, Some(block))
                                } else {
                                    (i, None)
                                }
                            }
                        } else {
                            // Can't peek after :, so it's not an else block
                            (input, None)
                        }
                    } else {
                        // No : found
                        (input, None)
                    }
                } else {
                    // Failed to skip whitespace
                    (input, None)
                }
            } else {
                // Indentation doesn't match - this ':' is not for this condition
                (input, None)
            }
        } else {
            // Failed to skip line endings
            (input, None)
        }
    };

    let end = ctx.offset(final_input);
    Ok((
        final_input,
        Statement::Condition {
            condition,
            then_block,
            else_ifs,
            else_block,
            span: ctx.make_span(start, end),
        },
    ))
}

/// Parse any statement
fn statement<'a>(input: &'a str, ctx: &'a ParseContext<'a>) -> IResult<&'a str, Statement> {
    alt((
        |i| function_definition(i, ctx),
        |i| for_loop(i, ctx),
        |i| condition_statement(i, ctx),
        |i| single_statement(i, ctx),
    ))(input)
}

/// Parse optional trailing comment (comments at end of line)
fn trailing_comment(input: &str) -> IResult<&str, ()> {
    let (input, _) = ws0(input)?;
    let (input, _) = opt(line_comment)(input)?;
    Ok((input, ()))
}

/// Parse a complete program
pub fn parse_program<'a>(input: &'a str, ctx: &'a ParseContext<'a>) -> IResult<&'a str, Program> {
    let start = ctx.offset(input);
    // Parse statements terminated by optional trailing comment + line endings
    let (input, statements) = many0(terminated(
        |i| statement(i, ctx),
        tuple((trailing_comment, many0(line_ending))),
    ))(input)?;
    let (input, _) = multispace0(input)?;

    let end = ctx.offset(input);
    Ok((
        input,
        Program {
            statements,
            span: ctx.make_span(start, end),
        },
    ))
}

/// Preprocess input to handle line continuations (^ prefix)
pub fn preprocess_line_continuations(input: &str) -> String {
    // Check if input ends with newline and which type
    let has_trailing_newline = input.ends_with('\n');
    let has_crlf = input.contains("\r\n");

    let mut result = String::new();
    let mut lines = input.lines().peekable();
    let mut last_line_was_comment = false;

    while let Some(line) = lines.next() {
        // Check if line starts with ^ (after whitespace)
        let trimmed = line.trim_start();

        if trimmed.starts_with('^') && !last_line_was_comment {
            // This is a continuation line - remove ^ and append to previous line
            // Remove the last line ending from result
            if result.ends_with("\r\n") {
                result.truncate(result.len() - 2);
            } else if result.ends_with('\n') {
                result.truncate(result.len() - 1);
            }
            // Remove the ^ and append the rest
            let continuation = &trimmed[1..];
            result.push_str(continuation);

            // Update comment status for the combined line
            // If we appended to a non-comment, it remains non-comment
            // (unless the continuation itself starts with //, but that would be weird syntax like x=1//comment)
            // But strictly speaking, if we append, we are extending the previous line.
            // If previous line was not comment, the combined line is not a comment line (it has code at start).
            last_line_was_comment = false;
        } else {
            // Regular line OR continuation after comment (which we treat as new line)
            if trimmed.starts_with('^') {
                // It was a continuation but previous line was comment, so we treat as new line
                // But we still strip the ^ because it was intended as continuation marker
                let content = &trimmed[1..];
                result.push_str(content);
                last_line_was_comment = content.trim_start().starts_with("//");
            } else {
                result.push_str(line);
                last_line_was_comment = trimmed.starts_with("//");
            }
        }

        // Add line ending if there are more lines
        if lines.peek().is_some() {
            if has_crlf {
                result.push_str("\r\n");
            } else {
                result.push('\n');
            }
        }
    }

    // Preserve trailing newline if it was present in the input
    if has_trailing_newline {
        if has_crlf {
            result.push_str("\r\n");
        } else {
            result.push('\n');
        }
    }

    result
}

/// Main entry point for parsing StoneScript
pub fn parse(input: &str) -> Result<Program, String> {
    // Preprocess to handle line continuations
    let processed = preprocess_line_continuations(input);
    let ctx = ParseContext::new(&processed);
    match parse_program(&processed, &ctx) {
        Ok((remaining, program)) => {
            if remaining.trim().is_empty() {
                Ok(program)
            } else {
                Err(format!(
                    "Failed to parse completely. Remaining: {:?}",
                    remaining
                ))
            }
        }
        Err(e) => Err(format!("Parse error: {:?}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fullwidth_quotes() {
        let input = "var x = storage.get(＂key＂, 0)\n";
        let result = parse(input);
        assert!(
            result.is_ok(),
            "Failed to parse fullwidth quotes: {:?}",
            result
        );
    }

    #[test]
    fn test_line_continuation() {
        let input = "?loc = icy_ridge\n^|loc = cross_bridge\n  x = 1\n";
        let result = parse(input);
        assert!(
            result.is_ok(),
            "Failed to parse line continuation: {:?}",
            result
        );
    }

    #[test]
    fn test_line_continuation_crlf() {
        let input = "?loc = icy_ridge\r\n^|loc = cross_bridge\r\n  x = 1\r\n";
        let result = parse(input);
        assert!(
            result.is_ok(),
            "Failed to parse line continuation with CRLF: {:?}",
            result
        );
    }

    #[test]
    fn test_chained_conditionals() {
        let input = "?x = 1\n  y = 2\n:?x = 3\n  y = 4\n:\n  y = 5\n";
        let ctx = ParseContext::new(input);
        let result = parse_program(input, &ctx);
        assert!(
            result.is_ok(),
            "Failed to parse chained conditionals: {:?}",
            result
        );
        let (remaining, _program) = result.unwrap();
        assert_eq!(remaining, "", "Remaining input: {:?}", remaining);
    }

    #[test]
    fn test_chained_conditionals_crlf() {
        let input = "?x = 1\r\n  y = 2\r\n:?x = 3\r\n  y = 4\r\n:\r\n  y = 5\r\n";
        let ctx = ParseContext::new(input);
        let result = parse_program(input, &ctx);
        assert!(
            result.is_ok(),
            "Failed to parse chained conditionals with CRLF: {:?}",
            result
        );
        let (remaining, _program) = result.unwrap();
        assert_eq!(remaining, "", "Remaining input: {:?}", remaining);
    }

    #[test]
    fn test_parse_identifier() {
        assert_eq!(identifier("foo"), Ok(("", "foo".to_string())));
        assert_eq!(identifier("loc"), Ok(("", "loc".to_string())));
        assert_eq!(identifier("_test"), Ok(("", "_test".to_string())));
    }

    #[test]
    fn test_parse_number() {
        let ctx = ParseContext::new("42");
        let result = number("42", &ctx);
        assert!(result.is_ok());
        match result.unwrap().1 {
            Expression::Integer(42, _) => (),
            _ => panic!("Expected Integer(42)"),
        }
    }

    #[test]
    fn test_parse_property_access() {
        let ctx = ParseContext::new("loc.stars");
        let result = property_access("loc.stars", &ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_binary_expression() {
        let ctx = ParseContext::new("loc.stars > 5");
        let result = binary_expression("loc.stars > 5", &ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_command() {
        let ctx = ParseContext::new("equip shovel");
        let result = command_statement("equip shovel", &ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_comment() {
        let ctx = ParseContext::new("// This is a comment");
        let result = comment("// This is a comment", &ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_simple_program() {
        let input = r#"?loc = caves
  equip shovel"#;

        let result = parse(input);
        assert!(result.is_ok());
    }
}
