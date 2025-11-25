use stonescript_parser;

fn main() {
    let source = "func test(a, b)\n  return a + b";
    
    println!("Parsing: {:?}", source);
    
    let tree = stonescript_parser::parse(source).expect("Failed to parse");
    let root = tree.root_node();
    
    println!("\n=== AST ===\n{}", root.to_sexp());
    println!("\n=== Pretty Print ===");
    print_tree(&root, source, 0);
}

fn print_tree(node: &tree_sitter::Node, source: &str, indent: usize) {
    let indent_str = " ".repeat(indent * 2);
    let kind = node.kind();
    
    if node.child_count() == 0 {
        let text = node.utf8_text(source.as_bytes()).unwrap();
        println!("{}{}:  {:?}", indent_str, kind, text);
    } else {
        println!("{}{}", indent_str, kind);
        for i in 0..node.child_count() {
            let child = node.child(i).unwrap();
            print_tree(&child, source, indent + 1);
        }
    }
}
