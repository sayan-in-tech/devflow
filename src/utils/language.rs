use std::{fs, path::Path};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Python,
    Node,
    Go,
    Rust,
    Unknown,
}

pub fn detect_project_language(root: &Path) -> Language {
    if root.join("pyproject.toml").exists() || root.join("requirements.txt").exists() {
        return Language::Python;
    }
    if root.join("package.json").exists() {
        return Language::Node;
    }
    if root.join("go.mod").exists() {
        return Language::Go;
    }
    if root.join("Cargo.toml").exists() {
        return Language::Rust;
    }
    Language::Unknown
}

pub fn expected_toolchain_hint(root: &Path) -> Option<String> {
    for file in [".nvmrc", "rust-toolchain", "go.mod", "pyproject.toml"] {
        let path = root.join(file);
        if path.exists() {
            if let Ok(c) = fs::read_to_string(path) {
                return Some(c.lines().next().unwrap_or_default().trim().to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn detects_rust() {
        let dir = tempdir().expect("tempdir");
        std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname='a'\n").expect("write");
        assert_eq!(detect_project_language(dir.path()), Language::Rust);
    }
}
