use std::path::Path;

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SupportedLanguage {
    Rust,
    TypeScript,
    Tsx,
    JavaScript,
    Jsx,
    Python,
}

impl SupportedLanguage {
    pub fn from_path(path: &Path) -> Option<Self> {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("rs") => Some(Self::Rust),
            Some("ts") => Some(Self::TypeScript),
            Some("tsx") => Some(Self::Tsx),
            Some("js") => Some(Self::JavaScript),
            Some("jsx") => Some(Self::Jsx),
            Some("py") => Some(Self::Python),
            _ => None,
        }
    }
}
