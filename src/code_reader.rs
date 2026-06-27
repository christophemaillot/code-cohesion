use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::Serialize;
use walkdir::WalkDir;

const MAX_READ_BYTES: usize = 32 * 1024;

#[derive(Debug, Clone, Serialize)]
pub struct ListedFile {
    pub path: String,
    pub bytes: u64,
}

#[derive(Debug, Clone)]
pub struct CodeReader {
    root: PathBuf,
}

impl CodeReader {
    pub fn new(root: impl AsRef<Path>) -> Result<Self> {
        let root = root
            .as_ref()
            .canonicalize()
            .with_context(|| format!("cannot resolve {}", root.as_ref().display()))?;
        Ok(Self { root })
    }

    pub fn list_files(&self, max_files: usize) -> Vec<ListedFile> {
        WalkDir::new(&self.root)
            .into_iter()
            .filter_entry(|entry| !is_ignored(entry.path()))
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .filter_map(|entry| {
                let meta = entry.metadata().ok()?;
                let rel = entry.path().strip_prefix(&self.root).ok()?;
                Some(ListedFile {
                    path: normalize_path(rel),
                    bytes: meta.len(),
                })
            })
            .take(max_files)
            .collect()
    }

    pub fn read_file(&self, relative_path: &str) -> Result<String> {
        let path = self.root.join(relative_path);
        let path = path
            .canonicalize()
            .with_context(|| format!("cannot resolve {relative_path}"))?;

        if !path.starts_with(&self.root) {
            bail!("refusing to read outside scan root");
        }

        let bytes = fs::read(&path).with_context(|| format!("cannot read {}", path.display()))?;
        let truncated = bytes.len() > MAX_READ_BYTES;
        let bytes = if truncated {
            &bytes[..MAX_READ_BYTES]
        } else {
            &bytes
        };
        let mut content = String::from_utf8_lossy(bytes).to_string();
        if truncated {
            content.push_str("\n\n[truncated]\n");
        }
        Ok(content)
    }
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

pub fn normalize_path(path: &Path) -> String {
    path.components()
        .map(|part| part.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}
