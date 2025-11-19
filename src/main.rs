use std::{env, fs::read_to_string};

use tree_sitter::Node;

fn main() {
    let mut args = env::args().skip(1);
    let Some(src_file) = args.next() else {
        eprintln!("usage: cargo r -- <path-to-yacc> <rule-name>");
        std::process::exit(1);
    };
    let rule_name = args
        .next()
        .unwrap_or_else(|| panic!("rule name is required (e.g. `expression`)"));
    if args.next().is_some() {
        eprintln!("warning: extra arguments were ignored");
    }

    let src = read_to_string(&src_file).expect("failed to read source file");

    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_bison::LANGUAGE.into())
        .expect("failed to load bison grammar");

    let tree = parser
        .parse(&src, None)
        .expect("tree-sitter failed to parse the file");
    if tree.root_node().has_error() {
        eprintln!("note: CST includes errors; result may be incomplete");
    }

    let Some(rule_node) = find_rule(tree.root_node(), &src, &rule_name) else {
        eprintln!("grammar rule `{}` not found", rule_name);
        std::process::exit(1);
    };

    let alternatives = extract_alternatives(rule_node, &src);
    if alternatives.is_empty() {
        println!("(no alternatives found)");
        return;
    }

    for alternative in alternatives {
        if alternative.is_empty() {
            println!("- (empty)");
        } else {
            println!("- {}", alternative.join(" "));
        }
    }
}

fn find_rule<'a>(root: Node<'a>, src: &str, expected: &str) -> Option<Node<'a>> {
    if root.kind() == "grammar_rule" {
        if let Some(name) = rule_name(root, src) {
            if name == expected {
                return Some(root);
            }
        }
    }

    for idx in 0..root.child_count() {
        if let Some(child) = root.child(idx) {
            if let Some(found) = find_rule(child, src, expected) {
                return Some(found);
            }
        }
    }

    None
}

fn rule_name(node: Node, src: &str) -> Option<String> {
    (0..node.child_count()).find_map(|idx| {
        node.child(idx).and_then(|child| {
            if child.kind() == "grammar_rule_declaration" {
                child
                    .utf8_text(src.as_bytes())
                    .ok()
                    .map(|text| text.to_string())
            } else {
                None
            }
        })
    })
}

fn extract_alternatives(rule_node: Node, src: &str) -> Vec<Vec<String>> {
    let mut alternatives = Vec::new();
    let mut current = Vec::new();

    for idx in 0..rule_node.child_count() {
        let Some(child) = rule_node.child(idx) else {
            continue;
        };

        if child.is_error() {
            continue;
        }

        match child.kind() {
            "grammar_rule_declaration" | ":" | ";" => continue,
            "|" => {
                alternatives.push(std::mem::take(&mut current));
            }
            // code in actions is explicitly dropped
            "action" | "code_block" | "error_sentinel" => {}
            _ => {
                if let Ok(text) = child.utf8_text(src.as_bytes()) {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        current.push(trimmed.to_string());
                    }
                }
            }
        }
    }

    alternatives.push(current);
    alternatives
}
