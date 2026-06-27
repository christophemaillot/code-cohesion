use std::collections::BTreeSet;

use super::Role;
use super::extractors::strip_string_literals;

pub(crate) fn infer_roles(path: &str, content: &str, imports: &[String]) -> Vec<Role> {
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
