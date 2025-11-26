use std::fs;
use std::path::Path;
use stonescript_parser::parse;

#[test]
fn validate_all_scripts() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let test_scripts_dir = Path::new(&manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("test_scripts");

    println!("Looking for scripts in: {:?}", test_scripts_dir);
    assert!(test_scripts_dir.exists(), "test_scripts directory not found");

    let mut count = 0;
    let mut errors = Vec::new();

    visit_dirs(&test_scripts_dir, &mut |entry| {
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "txt") {
            count += 1;
            println!("Testing: {:?}", path);
            match fs::read_to_string(&path) {
                Ok(source) => {
                    if let Some(tree) = parse(&source) {
                        let root = tree.root_node();
                        if root.has_error() {
                            errors.push(format!("Parse error in {:?}", path));
                            // Optional: print more details about where the error is
                            print_errors(root, &source, &path);
                        }
                    } else {
                        errors.push(format!("Failed to create tree for {:?}", path));
                    }
                }
                Err(e) => {
                    errors.push(format!("Failed to read {:?}: {}", path, e));
                }
            }
        }
    }).expect("Failed to walk directory");

    println!("Tested {} scripts", count);

    if !errors.is_empty() {
        for err in &errors {
            println!("{}", err);
        }
        panic!("Found {} errors", errors.len());
    }
}

fn visit_dirs(dir: &Path, cb: &mut dyn FnMut(&fs::DirEntry)) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&entry);
            }
        }
    }
    Ok(())
}

fn print_errors(node: tree_sitter::Node, source: &str, path: &Path) {
    if node.is_error() || node.is_missing() {
        let start = node.start_position();
        let end = node.end_position();
        println!(
            "Error in {:?} at {}:{}-{}:{} - Kind: {}",
            path,
            start.row + 1,
            start.column + 1,
            end.row + 1,
            end.column + 1,
            node.kind()
        );
    }
    
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        print_errors(child, source, path);
    }
}
