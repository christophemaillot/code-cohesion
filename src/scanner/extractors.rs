pub(crate) fn extract_imports(content: &str) -> Vec<String> {
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

pub(crate) fn strip_string_literals(content: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignores_role_words_inside_string_literals() {
        let stripped =
            strip_string_literals(r#"let config = ["react", "database", "endpoint", "validate"];"#);

        assert!(!stripped.contains("react"));
        assert!(!stripped.contains("database"));
        assert!(!stripped.contains("endpoint"));
        assert!(!stripped.contains("validate"));
    }
}
