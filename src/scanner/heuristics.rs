use std::path::Path;

use anyhow::Result;

use crate::code_reader::normalize_path;
use crate::language::SupportedLanguage;

use super::FileFinding;
use super::ast;
use super::extractors::extract_imports;
use super::role_mapper::infer_roles;
use super::scorer::score_file;

pub(crate) fn analyze_file(root: &Path, path: &Path, content: &str) -> Result<FileFinding> {
    let rel = path.strip_prefix(root)?;
    let normalized = normalize_path(rel);
    let language = SupportedLanguage::from_path(path).expect("scanner filters supported languages");
    let lines = content.lines().count();
    let imports = extract_imports(content);
    let symbols = ast::extract_symbols(language, content);
    let likely_roles = infer_roles(&normalized, content, &imports);
    let (suspicion, reasons) = score_file(lines, &imports, &symbols, &likely_roles);

    Ok(FileFinding {
        path: normalized,
        language,
        lines,
        imports,
        symbols,
        likely_roles,
        suspicion,
        reasons,
    })
}
