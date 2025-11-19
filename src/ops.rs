use tree_sitter::Node;

pub fn extract(src: &str, rule_name: &str, include_prec: bool) -> Result<Vec<Vec<String>>, String> {
    let tree = parse(src)?;
    if tree.root_node().has_error() {
        eprintln!("note: CST includes errors; result may be incomplete");
    }

    let Some(rule_node) = find_rule(tree.root_node(), src, rule_name) else {
        return Err(format!("grammar rule `{rule_name}` not found in CST"));
    };

    Ok(extract_rhs_variants(rule_node, src, include_prec))
}

fn parse(src: &str) -> Result<tree_sitter::Tree, String> {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_bison::LANGUAGE.into())
        .map_err(|e| format!("failed to load bison grammar: {e}"))?;

    parser
        .parse(src, None)
        .ok_or_else(|| "tree-sitter failed to parse the file".to_string())
}

fn find_rule<'a>(root: Node<'a>, src: &str, expected: &str) -> Option<Node<'a>> {
    if root.kind() == "grammar_rule"
        && let Some(name) = rule_name(root, src)
        && name == expected
    {
        return Some(root);
    }

    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        if let Some(found) = find_rule(child, src, expected) {
            return Some(found);
        }
    }

    None
}

fn rule_name(node: Node, src: &str) -> Option<String> {
    let mut cursor = node.walk();
    node.children(&mut cursor).find_map(|child| {
        if child.kind() == "grammar_rule_declaration" {
            child
                .utf8_text(src.as_bytes())
                .ok()
                .map(|text| text.to_string())
        } else {
            None
        }
    })
}

fn extract_rhs_variants(rule_node: Node, src: &str, include_prec: bool) -> Vec<Vec<String>> {
    let mut rhs_variants = Vec::new();
    let mut current = Vec::new();

    let mut cursor = rule_node.walk();
    for child in rule_node.children(&mut cursor) {
        if child.is_error() {
            continue;
        }

        match child.kind() {
            "grammar_rule_declaration" | ":" | ";" | "comment" => continue,
            "precedence_directive" if !include_prec => continue,
            "|" => {
                rhs_variants.push(std::mem::take(&mut current));
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

    rhs_variants.push(current);
    rhs_variants
}
