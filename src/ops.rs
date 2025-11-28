use regex::Regex;
use tree_sitter::Node;

/// Search for rule names (returns rule names only, no structure)
pub fn search_rules(
    src: &str,
    pattern: &str,
    use_regex: bool,
    search_target: SearchTarget,
) -> Result<Vec<String>, String> {
    let tree = parse(src)?;
    if tree.root_node().has_error() {
        eprintln!("note: CST includes errors; result may be incomplete");
    }

    let rule_names = match search_target {
        SearchTarget::Lhs => {
            if use_regex {
                let regex =
                    Regex::new(pattern).map_err(|e| format!("invalid regular expression: {e}"))?;
                find_rule_names_by_regex(tree.root_node(), src, &regex)
            } else {
                find_rule_names_by_substring(tree.root_node(), src, pattern)
            }
        }
        SearchTarget::Rhs => {
            if use_regex {
                let regex =
                    Regex::new(pattern).map_err(|e| format!("invalid regular expression: {e}"))?;
                find_rules_by_rhs_regex(tree.root_node(), src, &regex)
            } else {
                find_rules_by_rhs_substring(tree.root_node(), src, pattern)
            }
        }
        SearchTarget::Both => {
            if use_regex {
                let regex =
                    Regex::new(pattern).map_err(|e| format!("invalid regular expression: {e}"))?;
                find_rules_by_both_regex(tree.root_node(), src, &regex)
            } else {
                find_rules_by_both_substring(tree.root_node(), src, pattern)
            }
        }
    };

    if rule_names.is_empty() {
        return Err(format!(
            "grammar rule matching `{pattern}` not found in CST"
        ));
    }

    Ok(rule_names)
}

#[derive(Debug, Clone, Copy, Default)]
pub enum SearchTarget {
    Lhs,
    Rhs,
    #[default]
    Both,
}

/// Extract rule structure (exact match only)
pub fn extract(src: &str, rule_name: &str, include_prec: bool) -> Result<Vec<Vec<String>>, String> {
    let tree = parse(src)?;
    if tree.root_node().has_error() {
        eprintln!("note: CST includes errors; result may be incomplete");
    }

    let Some(rule_node) = find_rule_by_exact_match(tree.root_node(), src, rule_name) else {
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

fn find_rule_by_exact_match<'a>(root: Node<'a>, src: &str, expected: &str) -> Option<Node<'a>> {
    if root.kind() == "grammar_rule"
        && let Some(name) = rule_name(root, src)
        && name == expected
    {
        return Some(root);
    }

    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        if let Some(found) = find_rule_by_exact_match(child, src, expected) {
            return Some(found);
        }
    }

    None
}

fn find_rule_names_by_substring<'a>(root: Node<'a>, src: &str, pattern: &str) -> Vec<String> {
    let mut results = Vec::new();

    if root.kind() == "grammar_rule"
        && let Some(name) = rule_name(root, src)
        && name.contains(pattern)
    {
        results.push(name);
    }

    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        results.extend(find_rule_names_by_substring(child, src, pattern));
    }

    results
}

fn find_rule_names_by_regex<'a>(root: Node<'a>, src: &str, regex: &Regex) -> Vec<String> {
    let mut results = Vec::new();

    if root.kind() == "grammar_rule"
        && let Some(name) = rule_name(root, src)
        && regex.is_match(&name)
    {
        results.push(name);
    }

    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        results.extend(find_rule_names_by_regex(child, src, regex));
    }

    results
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

fn extract_rhs_text(rule_node: Node, src: &str) -> String {
    let mut rhs_parts = Vec::new();

    let mut cursor = rule_node.walk();
    for child in rule_node.children(&mut cursor) {
        if child.is_error() {
            continue;
        }

        match child.kind() {
            "grammar_rule_declaration" | ":" | ";" | "comment" => continue,
            "precedence_directive" => continue,
            "|" => {
                rhs_parts.push(" ");
            }
            // code in actions is explicitly dropped
            "action" | "code_block" | "error_sentinel" => {}
            _ => {
                if let Ok(text) = child.utf8_text(src.as_bytes()) {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        rhs_parts.push(trimmed);
                    }
                }
            }
        }
    }

    rhs_parts.join(" ")
}

fn find_rules_by_rhs_substring<'a>(root: Node<'a>, src: &str, pattern: &str) -> Vec<String> {
    let mut results = Vec::new();

    if root.kind() == "grammar_rule" {
        let rhs_text = extract_rhs_text(root, src);
        if rhs_text.contains(pattern)
            && let Some(name) = rule_name(root, src)
        {
            results.push(name);
        }
    }

    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        results.extend(find_rules_by_rhs_substring(child, src, pattern));
    }

    results
}

fn find_rules_by_rhs_regex<'a>(root: Node<'a>, src: &str, regex: &Regex) -> Vec<String> {
    let mut results = Vec::new();

    if root.kind() == "grammar_rule" {
        let rhs_text = extract_rhs_text(root, src);
        if regex.is_match(&rhs_text)
            && let Some(name) = rule_name(root, src)
        {
            results.push(name);
        }
    }

    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        results.extend(find_rules_by_rhs_regex(child, src, regex));
    }

    results
}

fn find_rules_by_both_substring<'a>(root: Node<'a>, src: &str, pattern: &str) -> Vec<String> {
    let mut results = Vec::new();

    if root.kind() == "grammar_rule" {
        let mut matches = false;

        // Check LHS
        if let Some(name) = rule_name(root, src) {
            if name.contains(pattern) {
                matches = true;
            }

            // Check RHS
            if !matches {
                let rhs_text = extract_rhs_text(root, src);
                if rhs_text.contains(pattern) {
                    matches = true;
                }
            }

            if matches {
                results.push(name);
            }
        }
    }

    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        results.extend(find_rules_by_both_substring(child, src, pattern));
    }

    results
}

fn find_rules_by_both_regex<'a>(root: Node<'a>, src: &str, regex: &Regex) -> Vec<String> {
    let mut results = Vec::new();

    if root.kind() == "grammar_rule" {
        let mut matches = false;

        // Check LHS
        if let Some(name) = rule_name(root, src) {
            if regex.is_match(&name) {
                matches = true;
            }

            // Check RHS
            if !matches {
                let rhs_text = extract_rhs_text(root, src);
                if regex.is_match(&rhs_text) {
                    matches = true;
                }
            }

            if matches {
                results.push(name);
            }
        }
    }

    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        results.extend(find_rules_by_both_regex(child, src, regex));
    }

    results
}
