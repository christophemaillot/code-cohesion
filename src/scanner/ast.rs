use tree_sitter::{Node, Parser};

use crate::language::SupportedLanguage;

pub(crate) fn extract_symbols(language: SupportedLanguage, content: &str) -> Vec<String> {
    let mut parser = Parser::new();
    if parser
        .set_language(&tree_sitter_language(language))
        .is_err()
    {
        return Vec::new();
    }

    let Some(tree) = parser.parse(content, None) else {
        return Vec::new();
    };

    let mut symbols = Vec::new();
    collect_symbols(language, tree.root_node(), content.as_bytes(), &mut symbols);
    symbols.truncate(120);
    symbols
}

fn tree_sitter_language(language: SupportedLanguage) -> tree_sitter::Language {
    match language {
        SupportedLanguage::Rust => tree_sitter_rust::LANGUAGE.into(),
        SupportedLanguage::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        SupportedLanguage::Tsx => tree_sitter_typescript::LANGUAGE_TSX.into(),
        SupportedLanguage::JavaScript | SupportedLanguage::Jsx => {
            tree_sitter_javascript::LANGUAGE.into()
        }
        SupportedLanguage::Python => tree_sitter_python::LANGUAGE.into(),
        SupportedLanguage::Kotlin => tree_sitter_kotlin_ng::LANGUAGE.into(),
    }
}

fn collect_symbols(
    language: SupportedLanguage,
    node: Node<'_>,
    source: &[u8],
    symbols: &mut Vec<String>,
) {
    if let Some(symbol) = symbol_for_node(language, node, source) {
        symbols.push(symbol);
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_symbols(language, child, source, symbols);
    }
}

fn symbol_for_node(language: SupportedLanguage, node: Node<'_>, source: &[u8]) -> Option<String> {
    match language {
        SupportedLanguage::Rust => rust_symbol(node, source),
        SupportedLanguage::TypeScript
        | SupportedLanguage::Tsx
        | SupportedLanguage::JavaScript
        | SupportedLanguage::Jsx => js_like_symbol(node, source),
        SupportedLanguage::Python => python_symbol(node, source),
        SupportedLanguage::Kotlin => kotlin_symbol(node, source),
    }
}

fn rust_symbol(node: Node<'_>, source: &[u8]) -> Option<String> {
    let label = match node.kind() {
        "function_item" => "fn",
        "struct_item" => "struct",
        "enum_item" => "enum",
        "trait_item" => "trait",
        "impl_item" => "impl",
        "mod_item" => "mod",
        _ => return None,
    };

    node.child_by_field_name("name")
        .and_then(|name| node_text(name, source))
        .map(|name| format!("{label} {name}"))
        .or_else(|| {
            if node.kind() == "impl_item" {
                Some("impl".to_string())
            } else {
                None
            }
        })
}

fn js_like_symbol(node: Node<'_>, source: &[u8]) -> Option<String> {
    match node.kind() {
        "function_declaration" | "method_definition" => node
            .child_by_field_name("name")
            .and_then(|name| node_text(name, source))
            .map(|name| format!("function {name}")),
        "class_declaration" => node
            .child_by_field_name("name")
            .and_then(|name| node_text(name, source))
            .map(|name| format!("class {name}")),
        "lexical_declaration" | "variable_declaration" => variable_symbol(node, source),
        _ => None,
    }
}

fn variable_symbol(node: Node<'_>, source: &[u8]) -> Option<String> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() != "variable_declarator" {
            continue;
        }
        if let Some(name) = child
            .child_by_field_name("name")
            .and_then(|name| node_text(name, source))
        {
            return Some(format!("const {name}"));
        }
    }
    None
}

fn python_symbol(node: Node<'_>, source: &[u8]) -> Option<String> {
    let label = match node.kind() {
        "function_definition" => "def",
        "class_definition" => "class",
        _ => return None,
    };

    node.child_by_field_name("name")
        .and_then(|name| node_text(name, source))
        .map(|name| format!("{label} {name}"))
}

fn kotlin_symbol(node: Node<'_>, source: &[u8]) -> Option<String> {
    let label = match node.kind() {
        "class_declaration" => "class",
        "function_declaration" => "fun",
        "object_declaration" => "object",
        _ => return None,
    };

    node.child_by_field_name("name")
        .and_then(|name| node_text(name, source))
        .map(|name| format!("{label} {name}"))
}

fn node_text(node: Node<'_>, source: &[u8]) -> Option<String> {
    node.utf8_text(source).ok().map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_rust_symbols_from_ast() {
        let symbols = extract_symbols(
            SupportedLanguage::Rust,
            "struct User {}\nimpl User { fn name(&self) {} }\nfn main() {}",
        );

        assert!(symbols.contains(&"struct User".to_string()));
        assert!(symbols.contains(&"fn main".to_string()));
    }

    #[test]
    fn extracts_typescript_symbols_from_ast() {
        let symbols = extract_symbols(
            SupportedLanguage::TypeScript,
            "export class User {}\nexport function loadUser() {}\nconst state = {};",
        );

        assert!(symbols.contains(&"class User".to_string()));
        assert!(symbols.contains(&"function loadUser".to_string()));
        assert!(symbols.contains(&"const state".to_string()));
    }

    #[test]
    fn extracts_kotlin_symbols_from_ast() {
        let symbols = extract_symbols(
            SupportedLanguage::Kotlin,
            "class User\nobject Users\nfun loadUser(): User = User()",
        );

        assert!(symbols.contains(&"class User".to_string()));
        assert!(symbols.contains(&"object Users".to_string()));
        assert!(symbols.contains(&"fun loadUser".to_string()));
    }
}
