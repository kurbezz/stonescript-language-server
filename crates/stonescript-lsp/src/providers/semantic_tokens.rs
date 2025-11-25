//! Semantic tokens provider

use tower_lsp::lsp_types::*;
use tree_sitter::Tree;

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
            ],
        }
    }

    pub fn legend(&self) -> SemanticTokensLegend {
        SemanticTokensLegend {
            token_types: self.token_types.clone(),
            token_modifiers: vec![],
        }
    }

    pub fn provide_semantic_tokens(&self, tree: &Tree, source: &str) -> SemanticTokens {
        let mut data = Vec::new();
        let mut prev_line = 0;
        let mut prev_col = 0;

        let mut cursor = tree.walk();
        self.visit_node(&cursor.node(), &mut cursor, source, &mut data, &mut prev_line, &mut prev_col);

        SemanticTokens {
            result_id: None,
            data,
        }
    }

    fn visit_node(
        &self,
        node: &tree_sitter::Node,
        cursor: &mut tree_sitter::TreeCursor,
        source: &str,
        data: &mut Vec<SemanticToken>,
        prev_line: &mut u32,
        prev_col: &mut u32,
    ) {
        let token_type = match node.kind() {
            "identifier" => Some(0), // VARIABLE
            "number" | "float" => Some(4), // NUMBER
            "string" => Some(5), // STRING
            "var" | "func" | "for" | "return" | "break" | "continue" => Some(2), // KEYWORD
            "+" | "-" | "*" | "/" | "=" | "!" | "<" | ">" | "&" | "|" => Some(3), // OPERATOR
            _ => None,
        };

        if let Some(token_type_idx) = token_type {
            let start_pos = node.start_position();
            let length = node.end_byte() - node.start_byte();

            let delta_line = start_pos.row as u32 - *prev_line;
            let delta_col = if delta_line == 0 {
                start_pos.column as u32 - *prev_col
            } else {
                start_pos.column as u32
            };

            data.push(SemanticToken {
                delta_line,
                delta_start: delta_col,
                length: length as u32,
                token_type: token_type_idx,
                token_modifiers_bitset: 0,
            });

            *prev_line = start_pos.row as u32;
            *prev_col = start_pos.column as u32;
        }

        if cursor.goto_first_child() {
            loop {
                self.visit_node(&cursor.node(), cursor, source, data, prev_line, prev_col);
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
            cursor.goto_parent();
        }
    }
}
