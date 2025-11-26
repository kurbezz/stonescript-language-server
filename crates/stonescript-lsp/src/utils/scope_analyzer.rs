//! Scope analysis using tree-sitter AST

use tree_sitter::{Node, Tree};
use std::collections::HashMap;

/// A variable in scope
#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub start_byte: usize,
    pub end_byte: usize,
    pub scope_id: usize,
}

/// A function declaration
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<String>,
    pub start_byte: usize,
    pub end_byte: usize,
    pub return_type: Option<crate::data::Type>,
    pub doc_comment: Option<String>,
    pub body_start_byte: Option<usize>,
    pub body_end_byte: Option<usize>,
}

/// Scope information
#[derive(Debug)]
pub struct Scope {
    pub id: usize,
    pub parent: Option<usize>,
    pub variables: HashMap<String, Variable>,
    pub start_byte: usize,
    pub end_byte: usize,
}

/// Analyzer for variable and function scopes
pub struct ScopeAnalyzer {
    scopes: Vec<Scope>,
    functions: HashMap<String, Function>,
    next_scope_id: usize,
}

impl ScopeAnalyzer {
    pub fn new() -> Self {
        // Global scope
        let global_scope = Scope {
            id: 0,
            parent: None,
            variables: HashMap::new(),
            start_byte: 0,
            end_byte: usize::MAX,
        };

        Self {
            scopes: vec![global_scope],
            functions: HashMap::new(),
            next_scope_id: 1,
        }
    }

    /// Analyze a tree-sitter parse tree
    pub fn analyze(&mut self, tree: &Tree, source: &str) {
        self.analyze_node(tree.root_node(), 0, source);
    }

    fn analyze_node(&mut self, node: Node, current_scope: usize, source: &str) {
        match node.kind() {
            "variable_declaration" => {
                self.handle_variable_declaration(node, current_scope, source);
            }
            "function_declaration" => {
                self.handle_function_declaration(node, current_scope, source);
                // Continue analyzing children to handle function_body
                self.analyze_children(node, current_scope, source);
            }
            "for_loop" => {
                // For loops create new scope
                let loop_scope = self.create_scope(current_scope, node.start_byte(), node.end_byte());
                self.handle_for_loop(node, loop_scope, source);
            }
            "function_body" => {
                // Function body creates new scope and inherits parameters
                let block_scope = self.create_scope(current_scope, node.start_byte(), node.end_byte());
                
                // Add function parameters to the function body scope
                if let Some(parent) = node.parent() {
                    if parent.kind() == "function_declaration" {
                        self.add_function_parameters_to_scope(parent, block_scope, source);
                    }
                }
                
                self.analyze_children(node, block_scope, source);
            }
            "block" => {
                // Blocks create new scopes
                let block_scope = self.create_scope(current_scope, node.start_byte(), node.end_byte());
                self.analyze_children(node, block_scope, source);
            }
            _ => {
                // Continue with same scope
                self.analyze_children(node, current_scope, source);
            }
        }
    }

    fn analyze_children(&mut self, node: Node, scope: usize, source: &str) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.analyze_node(child, scope, source);
        }
    }

    fn create_scope(&mut self, parent: usize, start_byte: usize, end_byte: usize) -> usize {
        let scope_id = self.next_scope_id;
        self.next_scope_id += 1;

        self.scopes.push(Scope {
            id: scope_id,
            parent: Some(parent),
            variables: HashMap::new(),
            start_byte,
            end_byte,
        });

        scope_id
    }

    fn handle_variable_declaration(&mut self, node: Node, scope_id: usize, source: &str) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = name_node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
            
            let variable = Variable {
                name: name.clone(),
                start_byte: node.start_byte(),
                end_byte: node.end_byte(),
                scope_id,
            };

            if let Some(scope) = self.scopes.get_mut(scope_id) {
                scope.variables.insert(name, variable);
            }
        }
    }

    fn handle_function_declaration(&mut self, node: Node, _scope_id: usize, source: &str) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = name_node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
            
            let mut parameters = Vec::new();
            let mut body_start_byte = None;
            let mut body_end_byte = None;
            
            // Find parameter_list node and function_body
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "parameter_list" {
                    // Extract identifiers from parameter list
                    let mut param_cursor = child.walk();
                    for param in child.children(&mut param_cursor) {
                        if param.kind() == "identifier" {
                            if let Ok(param_name) = param.utf8_text(source.as_bytes()) {
                                parameters.push(param_name.to_string());
                            }
                        }
                    }
                } else if child.kind() == "function_body" {
                    body_start_byte = Some(child.start_byte());
                    body_end_byte = Some(child.end_byte());
                }
            }

            // Extract doc comment from preceding comment
            let doc_comment = Self::extract_doc_comment(node, source);

            let function = Function {
                name: name.clone(),
                parameters,
                start_byte: node.start_byte(),
                end_byte: node.end_byte(),
                return_type: None, // Will be inferred later if needed
                doc_comment,
                body_start_byte,
                body_end_byte,
            };

            self.functions.insert(name, function);
        }
    }

    fn extract_doc_comment(node: Node, source: &str) -> Option<String> {
        // Look for comment nodes before the function declaration
        let mut current = node.prev_sibling()?;
        let mut comments = Vec::new();
        
        // Collect consecutive comment lines before the function
        loop {
            if current.kind() == "comment" {
                if let Ok(text) = current.utf8_text(source.as_bytes()) {
                    // Remove "//" prefix and trim
                    let comment_text = text.strip_prefix("//").unwrap_or(text).trim();
                    comments.push(comment_text.to_string());
                }
                
                // Check if there's another comment above
                if let Some(prev) = current.prev_sibling() {
                    current = prev;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        if comments.is_empty() {
            None
        } else {
            // Reverse because we collected from bottom to top
            comments.reverse();
            Some(comments.join("\n"))
        }
    }

    fn handle_for_loop(&mut self, node: Node, scope_id: usize, source: &str) {
        // Extract loop variable
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                // First identifier is the loop variable
                let name = child.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                let variable = Variable {
                    name: name.clone(),
                    start_byte: child.start_byte(),
                    end_byte: child.end_byte(),
                    scope_id,
                };

                if let Some(scope) = self.scopes.get_mut(scope_id) {
                    scope.variables.insert(name, variable);
                }
                break;
            }
        }

        // Analyze loop body
        self.analyze_children(node, scope_id, source);
    }

    fn add_function_parameters_to_scope(&mut self, func_node: Node, scope_id: usize, source: &str) {
        // Find parameter_list in the function declaration
        let mut cursor = func_node.walk();
        for child in func_node.children(&mut cursor) {
            if child.kind() == "parameter_list" {
                // Extract each parameter identifier
                let mut param_cursor = child.walk();
                for param in child.children(&mut param_cursor) {
                    if param.kind() == "identifier" {
                        if let Ok(param_name) = param.utf8_text(source.as_bytes()) {
                            let variable = Variable {
                                name: param_name.to_string(),
                                start_byte: param.start_byte(),
                                end_byte: param.end_byte(),
                                scope_id,
                            };

                            if let Some(scope) = self.scopes.get_mut(scope_id) {
                                scope.variables.insert(param_name.to_string(), variable);
                            }
                        }
                    }
                }
                break;
            }
        }
    }

    /// Find all variables visible at a given byte position
    pub fn find_variables_at(&self, byte: usize) -> Vec<&Variable> {
        let mut variables = Vec::new();

        // Find the innermost scope containing this position
        if let Some(scope_id) = self.find_scope_at(byte) {
            // Collect variables from this scope and all parent scopes
            let mut current_scope_id = Some(scope_id);
            while let Some(sid) = current_scope_id {
                if let Some(scope) = self.scopes.get(sid) {
                    for var in scope.variables.values() {
                        variables.push(var);
                    }
                    current_scope_id = scope.parent;
                }
            }
        }

        variables
    }

    fn find_scope_at(&self, byte: usize) -> Option<usize> {
        // Find the innermost (most specific) scope
        self.scopes
            .iter()
            .filter(|s| byte >= s.start_byte && byte <= s.end_byte)
            .max_by_key(|s| s.id)
            .map(|s| s.id)
    }

    /// Get all functions
    pub fn get_functions(&self) -> Vec<&Function> {
        self.functions.values().collect()
    }

    /// Get function by name
    pub fn get_function(&self, name: &str) -> Option<&Function> {
        self.functions.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_scope() {
        let source = "var x = 10\nvar y = 20";
        let tree = stonescript_parser::parse(source).unwrap();
        
        let mut analyzer = ScopeAnalyzer::new();
        analyzer.analyze(&tree, source);
        
        let vars = analyzer.find_variables_at(5);
        assert!(vars.iter().any(|v| v.name == "x"));
    }

    #[test]
    fn test_function_detection() {
        let source = "func test(a, b)\n  return a + b";
        let tree = stonescript_parser::parse(source).unwrap();
        
        let mut analyzer = ScopeAnalyzer::new();
        analyzer.analyze(&tree, source);
        
        let functions = analyzer.get_functions();
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "test");
        assert_eq!(functions[0].parameters.len(), 2);
    }

    #[test]
    fn test_function_parameters_in_scope() {
        let source = "func add(a, b)\n  return a + b";
        let tree = stonescript_parser::parse(source).unwrap();
        
        // Print AST structure
        println!("\n=== AST Structure ===");
        print_ast(&tree.root_node(), source, 0);
        
        let mut analyzer = ScopeAnalyzer::new();
        analyzer.analyze(&tree, source);
        
        // Print scopes
        println!("\n=== Scopes ===");
        for scope in &analyzer.scopes {
            println!("Scope {}: bytes {}..{}, parent: {:?}, vars: {:?}", 
                scope.id, scope.start_byte, scope.end_byte, scope.parent,
                scope.variables.keys().collect::<Vec<_>>());
        }
        
        // Find the position of 'a' in 'return a + b' (around byte 24)
        let a_position = source.find("return a").unwrap() + 7; // position of 'a'
        let b_position = source.find("+ b").unwrap() + 2; // position of 'b'
        
        println!("\nLooking for variables at position {} (should be 'a')", a_position);
        println!("Looking for variables at position {} (should be 'b')", b_position);
        
        // Check that 'a' is visible at its usage position
        let vars_at_a = analyzer.find_variables_at(a_position);
        println!("Variables at position {}: {:?}", a_position, vars_at_a.iter().map(|v| &v.name).collect::<Vec<_>>());
        assert!(vars_at_a.iter().any(|v| v.name == "a"), "Parameter 'a' should be visible in function body");
        
        // Check that 'b' is visible at its usage position
        let vars_at_b = analyzer.find_variables_at(b_position);
        println!("Variables at position {}: {:?}", b_position, vars_at_b.iter().map(|v| &v.name).collect::<Vec<_>>());
        assert!(vars_at_b.iter().any(|v| v.name == "b"), "Parameter 'b' should be visible in function body");
    }
}

fn print_ast(node: &tree_sitter::Node, source: &str, depth: usize) {
    let indent = "  ".repeat(depth);
    let text = node.utf8_text(source.as_bytes()).unwrap_or("<error>");
    let text_preview = if text.len() > 30 {
        format!("{}...", &text[..30].replace("\n", "\\n"))
    } else {
        text.replace("\n", "\\n")
    };
    
    println!("{}{} [{}..{}] \"{}\"", 
        indent, 
        node.kind(), 
        node.start_byte(),
        node.end_byte(),
        text_preview
    );
    
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        print_ast(&child, source, depth + 1);
    }
}
