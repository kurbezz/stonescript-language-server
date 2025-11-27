//! Semantic tokens provider

use stonescript_parser::{Expression, Program, Statement};
use tower_lsp::lsp_types::*;

pub struct SemanticTokensProvider {
    token_types: Vec<SemanticTokenType>,
}

impl SemanticTokensProvider {
    pub fn new() -> Self {
        Self {
            token_types: vec![
                SemanticTokenType::VARIABLE,
                SemanticTokenType::FUNCTION,
                SemanticTokenType::KEYWORD,
                SemanticTokenType::OPERATOR,
                SemanticTokenType::NUMBER,
                SemanticTokenType::STRING,
                SemanticTokenType::COMMENT,
                SemanticTokenType::PARAMETER,
                SemanticTokenType::PROPERTY,
            ],
        }
    }

    pub fn legend(&self) -> SemanticTokensLegend {
        SemanticTokensLegend {
            token_types: self.token_types.clone(),
            token_modifiers: vec![],
        }
    }

    pub fn provide_semantic_tokens(&self, ast: &Program, _source: &str) -> SemanticTokens {
        let mut tokens = Vec::new();

        for statement in &ast.statements {
            self.collect_statement_tokens(statement, &mut tokens);
        }

        // Sort tokens by position (line, then column)
        tokens.sort_by(|a, b| a.line.cmp(&b.line).then(a.start.cmp(&b.start)));

        // Convert to LSP delta-encoded format
        let data = self.encode_tokens(tokens);

        SemanticTokens {
            result_id: None,
            data,
        }
    }

    fn collect_statement_tokens(&self, statement: &Statement, tokens: &mut Vec<Token>) {
        match statement {
            Statement::Condition {
                condition,
                then_block,
                else_ifs,
                else_block,
                span: _,
            } => {
                // Add keyword token for '?'
                if let Some(span) = statement.span() {
                    tokens.push(Token {
                        line: span.start.line as u32,
                        start: span.start.column as u32,
                        length: 1,
                        token_type: 2, // KEYWORD
                    });
                }

                self.collect_expression_tokens(condition, tokens);

                for stmt in then_block {
                    self.collect_statement_tokens(stmt, tokens);
                }

                for else_if in else_ifs {
                    self.collect_expression_tokens(&else_if.condition, tokens);
                    for stmt in &else_if.block {
                        self.collect_statement_tokens(stmt, tokens);
                    }
                }

                if let Some(else_stmts) = else_block {
                    for stmt in else_stmts {
                        self.collect_statement_tokens(stmt, tokens);
                    }
                }
            }

            Statement::Command { name, args, span } => {
                // Command name as keyword
                tokens.push(Token {
                    line: span.start.line as u32,
                    start: span.start.column as u32,
                    length: name.len() as u32,
                    token_type: 2, // KEYWORD
                });

                for arg in args {
                    self.collect_expression_tokens(arg, tokens);
                }
            }

            Statement::Assignment {
                target,
                op: _,
                value,
                span: _,
            } => {
                self.collect_expression_tokens(target, tokens);
                self.collect_expression_tokens(value, tokens);
            }

            Statement::Output {
                position,
                text,
                span,
            } => {
                // Output operator '>' as keyword
                tokens.push(Token {
                    line: span.start.line as u32,
                    start: span.start.column as u32,
                    length: 1,
                    token_type: 2, // KEYWORD
                });

                if let Some((x, y)) = position {
                    self.collect_expression_tokens(x, tokens);
                    self.collect_expression_tokens(y, tokens);
                }
                self.collect_expression_tokens(text, tokens);
            }

            Statement::ExpressionStatement {
                expression,
                span: _,
            } => {
                self.collect_expression_tokens(expression, tokens);
            }

            Statement::FunctionDefinition {
                name,
                params: _,
                body,
                span,
            } => {
                // Function name
                tokens.push(Token {
                    line: span.start.line as u32,
                    start: span.start.column as u32,
                    length: name.len() as u32,
                    token_type: 1, // FUNCTION
                });

                // Parameters would need position info from parser
                // For now we skip them as we don't have their exact positions

                for stmt in body {
                    self.collect_statement_tokens(stmt, tokens);
                }
            }

            Statement::Return { value, span } => {
                // 'return' keyword
                tokens.push(Token {
                    line: span.start.line as u32,
                    start: span.start.column as u32,
                    length: 6,     // "return".len()
                    token_type: 2, // KEYWORD
                });

                if let Some(expr) = value {
                    self.collect_expression_tokens(expr, tokens);
                }
            }

            Statement::For {
                variable: _,
                range,
                body,
                span,
            } => {
                // 'for' keyword
                tokens.push(Token {
                    line: span.start.line as u32,
                    start: span.start.column as u32,
                    length: 3,     // "for".len()
                    token_type: 2, // KEYWORD
                });

                self.collect_expression_tokens(&range.0, tokens);
                self.collect_expression_tokens(&range.1, tokens);

                for stmt in body {
                    self.collect_statement_tokens(stmt, tokens);
                }
            }

            Statement::ForIn {
                variable: _,
                collection,
                body,
                span,
            } => {
                // 'for' keyword
                tokens.push(Token {
                    line: span.start.line as u32,
                    start: span.start.column as u32,
                    length: 3,
                    token_type: 2, // KEYWORD
                });

                self.collect_expression_tokens(collection, tokens);

                for stmt in body {
                    self.collect_statement_tokens(stmt, tokens);
                }
            }

            Statement::While {
                condition,
                body,
                span,
            } => {
                // 'while' keyword
                tokens.push(Token {
                    line: span.start.line as u32,
                    start: span.start.column as u32,
                    length: 5,     // "while".len()
                    token_type: 2, // KEYWORD
                });

                self.collect_expression_tokens(condition, tokens);

                for stmt in body {
                    self.collect_statement_tokens(stmt, tokens);
                }
            }

            Statement::Import { path: _, span } => {
                // 'import' keyword
                tokens.push(Token {
                    line: span.start.line as u32,
                    start: span.start.column as u32,
                    length: 6,     // "import".len()
                    token_type: 2, // KEYWORD
                });

                // Import path as string
                // We'd need better position tracking for the path itself
            }

            Statement::Comment(_text, span) => {
                tokens.push(Token {
                    line: span.start.line as u32,
                    start: span.start.column as u32,
                    length: (span.end.column - span.start.column) as u32,
                    token_type: 6, // COMMENT
                });
            }

            Statement::Empty => {}
        }
    }

    fn collect_expression_tokens(&self, expression: &Expression, tokens: &mut Vec<Token>) {
        match expression {
            Expression::Integer(_, span) => {
                tokens.push(Token {
                    line: span.start.line as u32,
                    start: span.start.column as u32,
                    length: (span.end.column - span.start.column) as u32,
                    token_type: 4, // NUMBER
                });
            }

            Expression::Float(_, span) => {
                tokens.push(Token {
                    line: span.start.line as u32,
                    start: span.start.column as u32,
                    length: (span.end.column - span.start.column) as u32,
                    token_type: 4, // NUMBER
                });
            }

            Expression::Boolean(_, span) => {
                tokens.push(Token {
                    line: span.start.line as u32,
                    start: span.start.column as u32,
                    length: (span.end.column - span.start.column) as u32,
                    token_type: 2, // KEYWORD (true/false are keywords)
                });
            }

            Expression::String(_, span) => {
                tokens.push(Token {
                    line: span.start.line as u32,
                    start: span.start.column as u32,
                    length: (span.end.column - span.start.column) as u32,
                    token_type: 5, // STRING
                });
            }

            Expression::Identifier(name, span) => {
                tokens.push(Token {
                    line: span.start.line as u32,
                    start: span.start.column as u32,
                    length: name.len() as u32,
                    token_type: 0, // VARIABLE
                });
            }

            Expression::Property {
                object,
                property,
                span,
            } => {
                self.collect_expression_tokens(object, tokens);

                // Property access
                tokens.push(Token {
                    line: span.start.line as u32,
                    start: (span.end.column - property.len()) as u32,
                    length: property.len() as u32,
                    token_type: 8, // PROPERTY
                });
            }

            Expression::FunctionCall {
                function,
                args,
                span: _,
            } => {
                // Mark the function name
                match function.as_ref() {
                    Expression::Identifier(name, fn_span) => {
                        tokens.push(Token {
                            line: fn_span.start.line as u32,
                            start: fn_span.start.column as u32,
                            length: name.len() as u32,
                            token_type: 1, // FUNCTION
                        });
                    }
                    Expression::Property {
                        object,
                        property,
                        span: prop_span,
                    } => {
                        self.collect_expression_tokens(object, tokens);
                        tokens.push(Token {
                            line: prop_span.start.line as u32,
                            start: (prop_span.end.column - property.len()) as u32,
                            length: property.len() as u32,
                            token_type: 1, // FUNCTION (method call)
                        });
                    }
                    _ => {
                        self.collect_expression_tokens(function, tokens);
                    }
                }

                for arg in args {
                    self.collect_expression_tokens(arg, tokens);
                }
            }

            Expression::BinaryOp {
                left,
                op: _,
                right,
                span: _,
            } => {
                self.collect_expression_tokens(left, tokens);
                self.collect_expression_tokens(right, tokens);
            }

            Expression::UnaryOp {
                op: _,
                operand,
                span: _,
            } => {
                self.collect_expression_tokens(operand, tokens);
            }

            Expression::Interpolation(_parts, span) => {
                // The whole interpolated string
                tokens.push(Token {
                    line: span.start.line as u32,
                    start: span.start.column as u32,
                    length: (span.end.column - span.start.column) as u32,
                    token_type: 5, // STRING
                });
            }

            Expression::New { path: _, span } => {
                // 'new' keyword
                tokens.push(Token {
                    line: span.start.line as u32,
                    start: span.start.column as u32,
                    length: 3,     // "new".len()
                    token_type: 2, // KEYWORD
                });
            }

            Expression::Array { elements, span: _ } => {
                for elem in elements {
                    self.collect_expression_tokens(elem, tokens);
                }
            }

            Expression::IndexAccess {
                object,
                index,
                span: _,
            } => {
                self.collect_expression_tokens(object, tokens);
                self.collect_expression_tokens(index, tokens);
            }
        }
    }

    fn encode_tokens(&self, tokens: Vec<Token>) -> Vec<SemanticToken> {
        let mut encoded = Vec::new();
        let mut prev_line = 0;
        let mut prev_start = 0;

        for token in tokens {
            let delta_line = token.line - prev_line;
            let delta_start = if delta_line == 0 {
                token.start - prev_start
            } else {
                token.start
            };

            encoded.push(SemanticToken {
                delta_line,
                delta_start,
                length: token.length,
                token_type: token.token_type,
                token_modifiers_bitset: 0,
            });

            prev_line = token.line;
            prev_start = token.start;
        }

        encoded
    }
}

#[derive(Debug, Clone)]
struct Token {
    line: u32,
    start: u32,
    length: u32,
    token_type: u32,
}
