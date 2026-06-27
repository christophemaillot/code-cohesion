use serde::Serialize;

use crate::language::SupportedLanguage;

#[derive(Debug, Clone, Serialize)]
pub struct ScanReport {
    pub root: String,
    pub files_scanned: usize,
    pub findings: Vec<FileFinding>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileFinding {
    pub path: String,
    pub language: SupportedLanguage,
    pub lines: usize,
    pub imports: Vec<String>,
    pub symbols: Vec<String>,
    pub likely_roles: Vec<Role>,
    pub suspicion: Suspicion,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Ui,
    State,
    ApiClient,
    Route,
    Domain,
    Persistence,
    Validation,
    Parsing,
    SideEffects,
    Tests,
    Configuration,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Suspicion {
    Low,
    Medium,
    High,
}
