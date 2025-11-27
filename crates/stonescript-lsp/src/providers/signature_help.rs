//! Signature help provider

use tower_lsp::lsp_types::*;
use crate::data::native_functions::{get_function, ALL_FUNCTIONS};
use crate::utils::ScopeAnalyzer;
use stonescript_parser::Program;

pub struct SignatureHelpProvider;

impl SignatureHelpProvider {
    pub fn new() -> Self {
        Self
    }

    pub fn provide_signature_help(
        &self,
        _ast: &Program,
        position: Position,
        source: &str,
        _scope: &ScopeAnalyzer,
    ) -> Option<SignatureHelp> {
        // Text-based approach: find function call pattern before cursor
        let line = source.lines().nth(position.line as usize)?;
        let text_before = &line[..position.character.min(line.len() as u32) as usize];
        
        // Look for pattern: namespace.function( or function(
        let regex = regex::Regex::new(r"(\w+)\.(\w+)\s*\($").ok()?;
        if let Some(caps) = regex.captures(text_before) {
            let namespace = caps.get(1)?.as_str();
            let func_name = caps.get(2)?.as_str();
            
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
