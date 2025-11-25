//! Signature help provider

use tower_lsp::lsp_types::*;
use tree_sitter::{Tree, Point};
use crate::data::native_functions::{get_function, ALL_FUNCTIONS};
use crate::utils::ScopeAnalyzer;

pub struct SignatureHelpProvider;

impl SignatureHelpProvider {
    pub fn new() -> Self {
        Self
    }

    pub fn provide_signature_help(
        &self,
        tree: &Tree,
        position: Position,
        source: &str,
        scope: &ScopeAnalyzer,
    ) -> Option<SignatureHelp> {
        let point = Point::new(position.line as usize, position.character as usize);
        
        // Find if we're inside a call_expression
        let node = tree.root_node()
            .named_descendant_for_point_range(point, point)?;
        
        // Walk up to find call_expression
        let mut current = Some(node);
        while let Some(n) = current {
            if n.kind() == "call_expression" {
                return self.signature_for_call_expression(&n, source, scope);
            }
            current = n.parent();
        }

        None
    }

    fn signature_for_call_expression(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        scope: &ScopeAnalyzer,
    ) -> Option<SignatureHelp> {
        let func_node = node.child_by_field_name("function")?;
        
        // Check if it's a member expression (namespace function)
        if func_node.kind() == "member_expression" {
            let object_node = func_node.child_by_field_name("object")?;
            let property_node = func_node.child_by_field_name("property")?;
            
            let namespace = object_node.utf8_text(source.as_bytes()).ok()?;
            let func_name = property_node.utf8_text(source.as_bytes()).ok()?;
            
            if let Some(func) = get_function(namespace, func_name) {
                let params: Vec<ParameterInformation> = func.parameters.iter()
                    .map(|p| ParameterInformation {
                        label: ParameterLabel::Simple(format!("{}: {:?}", p.name, p.typ)),
                        documentation: None,
                    })
                    .collect();

                let label = format!("{}.{}({})",
                    func.namespace,
                    func.name,
                    func.parameters.iter()
                        .map(|p| format!("{}: {:?}", p.name, p.typ))
                        .collect::<Vec<_>>()
                        .join(", ")
                );

                return Some(SignatureHelp {
                    signatures: vec![SignatureInformation {
                        label,
                        documentation: Some(Documentation::String(func.description.to_string())),
                        parameters: Some(params),
                        active_parameter: None,
                    }],
                    active_signature: Some(0),
                    active_parameter: None,
                });
            }
        } else if func_node.kind() == "identifier" {
            // User function
            let func_name = func_node.utf8_text(source.as_bytes()).ok()?;
            
            if let Some(func) = scope.get_function(func_name) {
                let params: Vec<ParameterInformation> = func.parameters.iter()
                    .map(|p| ParameterInformation {
                        label: ParameterLabel::Simple(p.clone()),
                        documentation: None,
                    })
                    .collect();

                return Some(SignatureHelp {
                    signatures: vec![SignatureInformation {
                        label: format!("{}({})", func.name, func.parameters.join(", ")),
                        documentation: None,
                        parameters: Some(params),
                        active_parameter: None,
                    }],
                    active_signature: Some(0),
                    active_parameter: None,
                });
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_help_provider() {
        let provider = SignatureHelpProvider::new();
        let _ = provider;
    }
}
