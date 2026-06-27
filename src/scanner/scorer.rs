use super::{Role, Suspicion};

pub(crate) fn score_file(
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
    fn high_suspicion_for_many_roles() {
        let roles = vec![Role::Ui, Role::State, Role::ApiClient, Role::Persistence];
        let (suspicion, reasons) = score_file(120, &[], &[], &roles);

        assert_eq!(suspicion, Suspicion::High);
        assert!(!reasons.is_empty());
    }
}
