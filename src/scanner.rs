mod heuristics;
mod types;

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use walkdir::WalkDir;

use heuristics::analyze_file;
pub use types::{FileFinding, Role, ScanReport, Suspicion};

pub fn scan(root: impl AsRef<Path>) -> Result<ScanReport> {
    let root = root
        .as_ref()
        .canonicalize()
        .with_context(|| format!("cannot resolve {}", root.as_ref().display()))?;

    let mut findings = Vec::new();

    for entry in WalkDir::new(&root)
        .into_iter()
        .filter_entry(|entry| !is_ignored(entry.path()))
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file() && is_source_file(entry.path()))
    {
        let path = entry.path().to_path_buf();
        let content =
            fs::read_to_string(&path).with_context(|| format!("cannot read {}", path.display()))?;
        findings.push(analyze_file(&root, &path, &content)?);
    }

    findings.sort_by(|a, b| {
        b.suspicion
            .cmp(&a.suspicion)
            .then_with(|| b.reasons.len().cmp(&a.reasons.len()))
            .then_with(|| b.lines.cmp(&a.lines))
    });

    Ok(ScanReport {
        root: root.display().to_string(),
        files_scanned: findings.len(),
        findings,
    })
}

fn is_source_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| {
            matches!(
                ext,
                "rs" | "ts" | "tsx" | "js" | "jsx" | "py" | "go" | "java" | "kt" | "swift" | "rb"
            )
        })
}

fn is_ignored(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            matches!(
                name,
                ".git" | "target" | "node_modules" | "dist" | "build" | ".next" | ".turbo"
            )
        })
}
