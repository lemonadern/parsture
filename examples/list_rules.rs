use std::{fs::read_to_string, path::PathBuf};

use clap::Parser;
use tree_sitter::Node;

#[derive(Parser)]
#[command(about = "List grammar rules and CST errors", version)]
struct Args {
    /// Path to the yacc/bison grammar file
    path: PathBuf,
}

fn main() {
    let args = Args::parse();
    let src = read_to_string(&args.path).expect("failed to read file");

    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_bison::LANGUAGE.into())
        .expect("failed to load bison grammar");

    let tree = parser.parse(&src, None).expect("parse failed");

    let mut rules = Vec::new();
    let mut errors = Vec::new();
    collect_rules(tree.root_node(), &src, &mut rules, &mut errors);

    let root = tree.root_node();
    if root.has_error() {
        println!(
            "rules: {}, errors shown: {} ({} total), span: {:?}-{:?}",
            rules.len(),
            errors.len().min(10),
            errors.len(),
            root.start_position(),
            root.end_position()
        );
        for err in errors.iter().take(5) {
            println!("ERROR\t{}", err + 1);
        }
        for err in errors.iter().rev().take(5).rev() {
            println!("ERROR\t{}", err + 1);
        }
    } else {
        println!(
            "rules: {}, errors: 0, span: {:?}-{:?}",
            rules.len(),
            root.start_position(),
            root.end_position()
        );
    }
    for (name, row) in rules {
        println!("{name}\t{}", row + 1);
    }
}

fn collect_rules(node: Node, src: &str, acc: &mut Vec<(String, usize)>, errors: &mut Vec<usize>) {
    if node.is_error() {
        errors.push(node.start_position().row);
    }
    if node.kind() == "grammar_rule" {
        if let Some(name) = rule_name(node, src) {
            acc.push((name, node.start_position().row));
        }
    }

    for idx in 0..node.child_count() {
        if let Some(child) = node.child(idx) {
            collect_rules(child, src, acc, errors);
        }
    }
}

fn rule_name(node: Node, src: &str) -> Option<String> {
    (0..node.child_count()).find_map(|idx| {
        node.child(idx).and_then(|child| {
            if child.kind() == "grammar_rule_declaration" {
                child.utf8_text(src.as_bytes()).ok().map(|t| t.to_string())
            } else {
                None
            }
        })
    })
}
