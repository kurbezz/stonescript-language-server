//! Parser for StoneScript using nom

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while, take_while1, take_while_m_n},
    character::complete::{alpha1, alphanumeric1, anychar, char, digit1, line_ending, multispace0},
    combinator::{map, opt, peek, recognize, value, verify},
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
        self.source.len() - remaining.len()
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
        value(BinaryOperator::Divide, char('/')),
        value(BinaryOperator::Modulo, char('%')),
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

/// Parse property access (e.g., loc.stars, foe.hp)
/// Parse a base expression (identifier, number, or parenthesized expression)
fn base_expression<'a>(input: &'a str, ctx: &ParseContext<'a>) -> IResult<&'a str, Expression> {
    alt((
        |i| quoted_string(i, ctx),
        |i| color_literal(i, ctx),
        |i| ascii_block(i, ctx),
        |i| array_literal(i, ctx),
        |i| new_expression(i, ctx),
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
    ))(input)
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

        // Try function call (arguments)
        if let Ok((next_input, _)) = char::<_, nom::error::Error<&str>>('(')(input) {
            if let Ok((next_input, _)) = ws0(next_input) {
                if let Ok((next_input, args)) =
                    separated_list0(delimited(ws0, char(','), ws0), |i| {
                        simple_expression(i, ctx)
                    })(next_input)
                {
                    if let Ok((next_input, _)) = ws0(next_input) {
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
            remaining = &remaining[2..];
        } else {
            let ch = remaining.chars().next().unwrap();
            content.push(ch);
            remaining = &remaining[ch.len_utf8()..];
        }
    }
}

/// Parse ascii block (multiline string)
fn ascii_block<'a>(input: &'a str, ctx: &ParseContext<'a>) -> IResult<&'a str, Expression> {
    let start = ctx.offset(input);
    let (input, _) = tag("ascii")(input)?;
    let (input, content) = take_until("asciiend")(input)?;
    let (input, _) = tag("asciiend")(input)?;

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
fn array_literal<'a>(input: &'a str, ctx: &ParseContext<'a>) -> IResult<&'a str, Expression> {
    let start = ctx.offset(input);
    let (input, _) = char('[')(input)?;
    let (input, _) = ws0(input)?;

    let (input, elements) = separated_list0(
        |i| {
            let (i, _) = ws0(i)?;
            let (i, _) = char(',')(i)?;
            let (i, _) = ws0(i)?;
            Ok((i, ()))
        },
        |i| expression(i, ctx),
    )(input)?;

    let (input, _) = ws0(input)?;
    let (input, _) = char(']')(input)?;
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
            let (i, expr) = primary_expression(i, ctx)?;
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
                let text_start = ctx.offset(input) - current_text.len();
                let text_end = ctx.offset(remaining);
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
                let text_start = start;
                let text_end = ctx.offset(remaining);
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
    let (input, _) = ws0(input)?;

    // Try to parse position (`x,y,)
    let (input, position) = opt(delimited(
        char('`'),
        separated_list1(char(','), |i| simple_expression(i, ctx)),
        char(','),
    ))(input)?;

    let (input, _) = ws0(input)?;

    // Parse the rest as interpolated string
    let (input, text) = interpolated_string(input, ctx)?;

    let position = position.and_then(|pos| {
        if pos.len() >= 2 {
            Some((pos[0].clone(), pos[1].clone()))
        } else {
            None
        }
    });

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
fn var_assignment<'a>(input: &'a str, ctx: &ParseContext<'a>) -> IResult<&'a str, Statement> {
    let start = ctx.offset(input);
    let (input, _) = tag("var")(input)?;
    let (input, _) = ws1(input)?;
    let (input, name) = identifier(input)?;
    let (input, _) = ws0(input)?;

    // Check if we have an assignment
    if let Ok((next_input, _)) = char::<_, nom::error::Error<&str>>('=')(input) {
        // We have an equals sign, so we MUST parse an expression
        let (input, _) = ws0(next_input)?;
        match expression(input, ctx) {
            Ok((input, value)) => {
                let end = ctx.offset(input);
                let name_span = ctx.make_span(start, ctx.offset(&input) - value.span().end.column);
                Ok((
                    input,
                    Statement::Assignment {
                        target: Expression::Identifier(name.clone(), name_span),
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
            // Valid lvalue
        }
        _ => {
            // If it's not a valid lvalue, fail the parse
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag,
            )));
        }
    }

    let (input, _) = ws0(input)?;
    let (input, _) = char('=')(input)?;
    let (input, _) = ws0(input)?;
    let (input, value) = expression(input, ctx)?;

    let end = ctx.offset(input);
    Ok((
        input,
        Statement::Assignment {
            target,
            value,
            span: ctx.make_span(start, end),
        },
    ))
}

/// Parse return statement
fn return_statement<'a>(input: &'a str, ctx: &ParseContext<'a>) -> IResult<&'a str, Statement> {
    let start = ctx.offset(input);
    let (input, _) = tag("return")(input)?;
    let (input, _) = ws0(input)?; // Optional space? usually required if value follows

    // Try to parse return value
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

/// Parse for loop: for i = start..end
fn for_loop<'a>(input: &'a str, ctx: &ParseContext<'a>) -> IResult<&'a str, Statement> {
    let start = ctx.offset(input);
    let (input, _) = tag("for")(input)?;
    let (input, _) = ws1(input)?;
    let (input, variable) = identifier(input)?;
    let (input, _) = ws0(input)?;
    let (input, _) = char('=')(input)?;
    let (input, _) = ws0(input)?;

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
}

/// Parse function definition: func Name(arg1, arg2)
fn function_definition<'a>(input: &'a str, ctx: &ParseContext<'a>) -> IResult<&'a str, Statement> {
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
fn single_statement<'a>(input: &'a str, ctx: &ParseContext<'a>) -> IResult<&'a str, Statement> {
    preceded(
        ws0,
        alt((
            |i| import_statement(i, ctx),
            |i| comment(i, ctx),
            |i| return_statement(i, ctx),
            |i| output_statement(i, ctx),
            map(line_ending, |_| Statement::Empty),
            // Try var assignment first (var x = value)
            verify(|i| var_assignment(i, ctx), |_| true),
            // Try regular assignment
            verify(|i| assignment_statement(i, ctx), |_| true),
            |i| command_statement(i, ctx),
            // Try expression statement last (e.g., function calls)
            |i| expression_statement(i, ctx),
        )),
    )(input)
}

/// Parse indented block of statements
fn indented_block<'a>(input: &'a str, ctx: &ParseContext<'a>) -> IResult<&'a str, Vec<Statement>> {
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
            let (next, _) = line_ending(remaining)?;
            remaining = next;
            continue;
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

            // If at base level and line starts with : or :?, this is an else clause
            // Return to let the parent condition_statement handle it
            if current_indent == base
                && (after_spaces.starts_with(":?") || after_spaces.starts_with(':'))
            {
                return Ok((line_start, statements));
            }
        }

        // Try to parse a statement
        match statement(after_spaces, ctx) {
            Ok((next, stmt)) => {
                statements.push(stmt);
                remaining = next;
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
fn condition_statement<'a>(input: &'a str, ctx: &ParseContext<'a>) -> IResult<&'a str, Statement> {
    let start = ctx.offset(input);
    let (input, _) = char('?')(input)?;
    let (input, _) = ws0(input)?;
    let (input, condition) = expression(input, ctx)?;
    let (input, _) = opt(line_ending)(input)?;

    // Parse then block (indented statements)
    let (input, then_block) = indented_block(input, ctx)?;

    // Parse else-if branches
    let (input, else_ifs) = many0(|i| {
        let start = ctx.offset(i);
        let (i, _) = opt(line_ending)(i)?;
        let (i, _) = ws0(i)?;
        let (i, _) = tag(":?")(i)?;
        let (i, _) = ws0(i)?;
        let (i, cond) = expression(i, ctx)?;
        let (i, _) = opt(line_ending)(i)?;
        let (i, block) = indented_block(i, ctx)?;
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
    let (input, else_block) = opt(map(
        tuple((
            opt(line_ending),
            ws0,
            char(':'),
            verify(peek(opt(anychar)), |c: &Option<char>| *c != Some('?')),
            opt(line_ending),
            |i| indented_block(i, ctx),
        )),
        |(_, _, _, _, _, block)| block,
    ))(input)?;

    let end = ctx.offset(input);
    Ok((
        input,
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
fn statement<'a>(input: &'a str, ctx: &ParseContext<'a>) -> IResult<&'a str, Statement> {
    alt((
        |i| function_definition(i, ctx),
        |i| for_loop(i, ctx),
        |i| condition_statement(i, ctx),
        |i| single_statement(i, ctx),
    ))(input)
}

/// Parse a complete program
pub fn parse_program<'a>(input: &'a str, ctx: &ParseContext<'a>) -> IResult<&'a str, Program> {
    let start = ctx.offset(input);
    let (input, statements) = many0(terminated(|i| statement(i, ctx), many0(line_ending)))(input)?;
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
fn preprocess_line_continuations(input: &str) -> String {
    let mut result = String::new();
    let mut lines = input.lines().peekable();
    let has_crlf = input.contains("\r\n");

    while let Some(line) = lines.next() {
        // Check if line starts with ^ (after whitespace)
        let trimmed = line.trim_start();
        if trimmed.starts_with('^') {
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
        } else {
            // Regular line - add it as-is
            result.push_str(line);
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
