use std::path::PathBuf;

fn main() {
    let _dir: PathBuf = ["tree-sitter-stonescript", "src"].iter().collect();
    let tree_sitter_dir = PathBuf::from("../../../tree-sitter-stonescript");
    
    cc::Build::new()
        .include(&tree_sitter_dir.join("src"))
        .file(tree_sitter_dir.join("src").join("parser.c"))
        .file(tree_sitter_dir.join("src").join("scanner.c"))
        .warnings(false)
        .compile("tree-sitter-stonescript");
    
    println!("cargo:rerun-if-changed=../../../tree-sitter-stonescript/src/parser.c");
    println!("cargo:rerun-if-changed=../../../tree-sitter-stonescript/src/scanner.c");
}
