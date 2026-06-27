use std::collections::BTreeSet;
use std::path::Path;

use anyhow::Result;

use crate::code_reader::normalize_path;

use super::{FileFinding, Role, Suspicion};

pub(crate) fn analyze_file(root: &Path, path: &Path, content: &str) -> Result<FileFinding> {
    let rel = path.strip_prefix(root)?;
    let normalized = normalize_path(rel);
    let lines = content.lines().count();
    let imports = extract_imports(content);
    let symbols = extract_symbols(content);
    let likely_roles = infer_roles(&normalized, content, &imports);
    let (suspicion, reasons) = suspicion_for(lines, &imports, &symbols, &likely_roles);

    Ok(FileFinding {
        path: normalized,
        lines,
        imports,
        symbols,
        likely_roles,
        suspicion,
        reasons,
    })
}

fn extract_imports(content: &str) -> Vec<String> {
    content
        .lines()
        .map(str::trim)
        .filter(|line| {
            line.starts_with("use ")
                || line.starts_with("mod ")
                || line.starts_with("import ")
                || line.starts_with("from ")
                || line.starts_with("require(")
        })
        .take(80)
        .map(|line| line.trim_end_matches(';').to_string())
        .collect()
}

fn extract_symbols(content: &str) -> Vec<String> {
    content
        .lines()
        .map(str::trim)
        .filter_map(|line| {
            for prefix in [
                "pub fn ",
                "fn ",
                "pub struct ",
                "struct ",
                "pub enum ",
                "enum ",
                "class ",
                "export function ",
                "function ",
                "export const ",
                "const ",
            ] {
                if let Some(rest) = line.strip_prefix(prefix) {
                    let name = rest
                        .split(|ch: char| !ch.is_alphanumeric() && ch != '_')
                        .next()
                        .unwrap_or_default();
                    if !name.is_empty() {
                        return Some(format!("{} {}", prefix.trim_end(), name));
                    }
                }
            }
            None
        })
        .take(120)
        .collect()
}

fn infer_roles(path: &str, content: &str, imports: &[String]) -> Vec<Role> {
    let path = path.to_lowercase();
    let imports = imports.join("\n").to_lowercase();
    let content = strip_string_literals(content).to_lowercase();
    let haystack = format!("{path}\n{imports}\n{content}");
    let mut roles = BTreeSet::new();

    if path.ends_with(".tsx")
        || path.ends_with(".jsx")
        || haystack.contains("from 'react'")
        || haystack.contains("from \"react\"")
        || haystack.contains("jsx")
    {
        roles.insert(Role::Ui);
    }

    add_if(
        &mut roles,
        Role::State,
        &haystack,
        &[
            "use_state",
            "usestate",
            "store",
            "reducer",
            "zustand",
            "redux",
        ],
    );
    add_if(
        &mut roles,
        Role::ApiClient,
        &haystack,
        &["fetch(", "reqwest", "axios", "graphql"],
    );

    if path.contains("/routes/")
        || path.contains("/controllers/")
        || haystack.contains("endpoint")
        || haystack.contains("handler")
    {
        roles.insert(Role::Route);
    }

    if path.contains("/domain/") || path.contains("service") || haystack.contains("business") {
        roles.insert(Role::Domain);
    }

    add_if(
        &mut roles,
        Role::Persistence,
        &haystack,
        &[
            "sqlx",
            "database",
            "repository",
            "sqlite",
            "postgres",
            "prisma",
            "diesel",
        ],
    );
    add_if(
        &mut roles,
        Role::Validation,
        &haystack,
        &["validate", "validator", "schema", "zod"],
    );

    if path.contains("parser") || haystack.contains("regex") || haystack.contains("nom::") {
        roles.insert(Role::Parsing);
    }

    add_if(
        &mut roles,
        Role::SideEffects,
        &haystack,
        &[
            "email", "mailer", "stripe", "std::fs", "fs::", "spawn", "command",
        ],
    );

    if path.contains("test")
        || path.contains("spec")
        || haystack.contains("#[test]")
        || haystack.contains("mod tests")
    {
        roles.insert(Role::Tests);
    }

    add_if(
        &mut roles,
        Role::Configuration,
        &haystack,
        &["config", "settings", "std::env", " env = "],
    );

    roles.into_iter().collect()
}

fn add_if(roles: &mut BTreeSet<Role>, role: Role, haystack: &str, needles: &[&str]) {
    if needles.iter().any(|needle| haystack.contains(needle)) {
        roles.insert(role);
    }
}

fn strip_string_literals(content: &str) -> String {
    let mut output = String::with_capacity(content.len());
    let mut in_string = false;
    let mut quote = '\0';
    let mut escaped = false;

    for ch in content.chars() {
        if in_string {
            if escaped {
                escaped = false;
                output.push(' ');
                continue;
            }
            if ch == '\\' {
                escaped = true;
                output.push(' ');
                continue;
            }
            if ch == quote {
                in_string = false;
            }
            output.push(if ch == '\n' { '\n' } else { ' ' });
            continue;
        }

        if ch == '"' || ch == '\'' || ch == '`' {
            in_string = true;
            quote = ch;
            output.push(' ');
        } else {
            output.push(ch);
        }
    }

    output
}

fn suspicion_for(
    lines: usize,
    imports: &[String],
    symbols: &[String],
    likely_roles: &[Role],
) -> (Suspicion, Vec<String>) {
    let mut reasons = Vec::new();

    if lines >= 800 {
        reasons.push(format!("large file: {lines} lines"));
    } else if lines >= 400 {
        reasons.push(format!("medium-large file: {lines} lines"));
    }

    if imports.len() >= 25 {
        reasons.push(format!("many imports: {}", imports.len()));
    }

    if symbols.len() >= 20 {
        reasons.push(format!("many top-level symbols: {}", symbols.len()));
    }

    if likely_roles.len() >= 4 {
        reasons.push(format!(
            "many likely responsibilities: {}",
            likely_roles.len()
        ));
    } else if likely_roles.len() >= 3 {
        reasons.push(format!(
            "several likely responsibilities: {}",
            likely_roles.len()
        ));
    }

    let suspicion = if lines >= 800 || likely_roles.len() >= 4 || reasons.len() >= 3 {
        Suspicion::High
    } else if lines >= 400 || likely_roles.len() >= 3 || reasons.len() >= 2 {
        Suspicion::Medium
    } else {
        Suspicion::Low
    };

    (suspicion, reasons)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_mixed_roles() {
        let imports = vec![
            "use reqwest::Client".to_string(),
            "use sqlx::PgPool".to_string(),
        ];
        let roles = infer_roles(
            "src/routes/user_service.rs",
            "fn validate_user() {}\nfn send_email() {}\nlet database = postgres;",
            &imports,
        );

        assert!(roles.contains(&Role::ApiClient));
        assert!(roles.contains(&Role::Persistence));
        assert!(roles.contains(&Role::Route));
        assert!(roles.contains(&Role::Validation));
    }

    #[test]
    fn high_suspicion_for_many_roles() {
        let roles = vec![Role::Ui, Role::State, Role::ApiClient, Role::Persistence];
        let (suspicion, reasons) = suspicion_for(120, &[], &[], &roles);

        assert_eq!(suspicion, Suspicion::High);
        assert!(!reasons.is_empty());
    }

    #[test]
    fn ignores_role_words_inside_string_literals() {
        let roles = infer_roles(
            "src/scanner.rs",
            r#"let config = ["react", "database", "endpoint", "validate"];"#,
            &[],
        );

        assert!(!roles.contains(&Role::Ui));
        assert!(!roles.contains(&Role::Route));
        assert!(!roles.contains(&Role::Persistence));
        assert!(!roles.contains(&Role::Validation));
    }
}
