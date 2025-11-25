//! Definition provider (go-to-definition)

use tower_lsp::lsp_types::*;
use tree_sitter::{Tree, Point};
use crate::utils::ScopeAnalyzer;

pub struct DefinitionProvider;

impl DefinitionProvider {
    pub fn new() -> Self {
        Self
    }

    pub fn provide_definition(
        &self,
        tree: &Tree,
        position: Position,
        source: &str,
        scope: &ScopeAnalyzer,
        uri: &Url,
    ) -> Option<GotoDefinitionResponse> {
        let point = Point::new(position.line as usize, position.character as usize);
        
        let node = tree.root_node()
            .named_descendant_for_point_range(point, point)?;
        
        if node.kind() == "identifier" {
            let text = node.utf8_text(source.as_bytes()).ok()?;
            
            // Check variables
            let variables = scope.find_variables_at(node.start_byte());
            if let Some(var) = variables.iter().find(|v| v.name == text) {
                return Some(GotoDefinitionResponse::Scalar(Location {
                    uri: uri.clone(),
                    range: self.byte_range_to_lsp(var.start_byte, var.end_byte, source),
                }));
            }

            // Check functions
            if let Some(func) = scope.get_function(text) {
                return Some(GotoDefinitionResponse::Scalar(Location {
                    uri: uri.clone(),
                    range: self.byte_range_to_lsp(func.start_byte, func.end_byte, source),
                }));
            }
        }

        None
    }

    fn byte_range_to_lsp(&self, start: usize, end: usize, source: &str) -> Range {
        let start_pos = self.byte_to_position(start, source);
        let end_pos = self.byte_to_position(end, source);
        Range { start: start_pos, end: end_pos }
    }

    fn byte_to_position(&self, byte: usize, source: &str) -> Position {
        let mut line = 0;
        let mut col = 0;
        
        for (i, ch) in source.chars().enumerate() {
            if i >= byte {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
        }

        Position { line: line as u32, character: col as u32 }
    }
}
