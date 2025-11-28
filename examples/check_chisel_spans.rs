use stonescript_parser::parse;
use stonescript_parser::{Expression, Statement};
use std::fs;

fn main() {
    let source = fs::read_to_string("test_scripts/Chisel.txt").expect("Failed to read Chisel.txt");
    let ast = parse(&source).expect("Failed to parse");
    
    // Find GenerateSaveLoadButtons function
    for (idx, stmt) in ast.statements.iter().enumerate() {
        if let Statement::FunctionDefinition { name, body, span, .. } = stmt {
            if name == "GenerateSaveLoadButtons" {
                println!("Found function '{}' at top-level index {}", name, idx);
                println!("Function span: line {}..{}, col {}..{}", 
                         span.start.line, span.end.line, 
                         span.start.column, span.end.column);
                println!("Function body has {} statements", body.len());
                
                // Print first few statements
                for (stmt_idx, body_stmt) in body.iter().take(5).enumerate() {
                    if let Statement::Assignment { target, span, .. } = body_stmt {
                        if let Expression::Identifier(var_name, _) = target {
                            println!("  [{}] var {} at span line {}..{}, col {}..{}", 
                                     stmt_idx, var_name,
                                     span.start.line, span.end.line,
                                     span.start.column, span.end.column);
                        }
                    }
                }
                break;
            }
        }
    }
}
