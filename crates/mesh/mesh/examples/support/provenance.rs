// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Resolve exact external package identities from the compiled workspace lock.

use serde::Deserialize;
use serde_json::{Value, json};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, Error>;

#[derive(Deserialize)]
struct Lockfile {
    package: Vec<LockPackage>,
}

#[derive(Deserialize)]
struct LockPackage {
    name: String,
    version: String,
    source: Option<String>,
    #[serde(default)]
    dependencies: Vec<String>,
}

/// Return exact selected package identities from the compiled workspace lock.
/// Each requested name must be a direct dependency of `mere-mesh` and resolve
/// to exactly one git-sourced package entry.
pub fn selected_git_packages(lock: &[u8], names: &[&str]) -> Result<Vec<Value>> {
    let lock = std::str::from_utf8(lock)?;
    let lock: Lockfile = toml::from_str(lock)?;

    let roots: Vec<_> = lock
        .package
        .iter()
        .filter(|package| package.name == "mere-mesh")
        .collect();
    if roots.len() != 1 {
        return Err(format!(
            "Cargo.lock must contain exactly one mere-mesh package; found {}",
            roots.len()
        )
        .into());
    }

    let mut selected = Vec::with_capacity(names.len());
    for name in names {
        if !roots[0]
            .dependencies
            .iter()
            .any(|dependency| dependency.split_whitespace().next() == Some(name))
        {
            return Err(format!(
                "Cargo.lock package {name} is not a selected direct mere-mesh dependency"
            )
            .into());
        }

        let matches: Vec<_> = lock
            .package
            .iter()
            .filter(|package| package.name.as_str() == *name)
            .collect();
        if matches.len() != 1 {
            return Err(format!(
                "Cargo.lock must contain exactly one selected package named {name}; found {}",
                matches.len()
            )
            .into());
        }
        let package = matches[0];
        let source = package
            .source
            .as_deref()
            .ok_or_else(|| format!("Cargo.lock package {name} has no source"))?;
        if !source.starts_with("git+") {
            return Err(format!("Cargo.lock package {name} is not git sourced: {source}").into());
        }
        let revision = source
            .rsplit_once('#')
            .map(|(_, revision)| revision)
            .filter(|revision| {
                revision.len() == 40 && revision.bytes().all(|byte| byte.is_ascii_hexdigit())
            })
            .ok_or_else(|| format!("Cargo.lock package {name} has no resolved git revision"))?;

        selected.push(json!({
            "name": package.name.as_str(),
            "version": package.version.as_str(),
            "source": source,
            "resolved_revision": revision,
        }));
    }
    Ok(selected)
}

#[cfg(test)]
mod tests {
    use super::selected_git_packages;

    const DEPENDENCY: &str = "mere-transport";
    const VALID_LOCK: &str = r#"
version = 4

[[package]]
name = "mere-mesh"
version = "0.0.1"
dependencies = ["mere-transport"]

[[package]]
name = "mere-transport"
version = "0.2.0"
source = "git+https://example.test/murm?branch=main#0123456789abcdef0123456789abcdef01234567"
"#;

    #[test]
    fn returns_actual_git_package_revision() {
        let selected = selected_git_packages(VALID_LOCK.as_bytes(), &[DEPENDENCY]).unwrap();
        assert_eq!(selected[0]["name"], DEPENDENCY);
        assert_eq!(selected[0]["version"], "0.2.0");
        assert_eq!(
            selected[0]["resolved_revision"],
            "0123456789abcdef0123456789abcdef01234567"
        );
    }

    #[test]
    fn rejects_missing_package() {
        let lock = VALID_LOCK.replace("name = \"mere-transport\"", "name = \"other\"");
        assert!(selected_git_packages(lock.as_bytes(), &[DEPENDENCY]).is_err());
    }

    #[test]
    fn rejects_ambiguous_package_name() {
        let duplicate = r#"
[[package]]
name = "mere-transport"
version = "0.2.1"
source = "git+https://example.test/murm?branch=main#1123456789abcdef0123456789abcdef01234567"
"#;
        let lock = format!("{VALID_LOCK}\n{duplicate}");
        assert!(selected_git_packages(lock.as_bytes(), &[DEPENDENCY]).is_err());
    }

    #[test]
    fn rejects_non_git_package() {
        let lock = VALID_LOCK.replace(
            "git+https://example.test/murm?branch=main#0123456789abcdef0123456789abcdef01234567",
            "registry+https://example.test/index",
        );
        assert!(selected_git_packages(lock.as_bytes(), &[DEPENDENCY]).is_err());
    }

    #[test]
    fn rejects_package_not_selected_by_root() {
        let lock = VALID_LOCK.replace("dependencies = [\"mere-transport\"]", "dependencies = []");
        assert!(selected_git_packages(lock.as_bytes(), &[DEPENDENCY]).is_err());
    }

    #[test]
    fn rejects_unresolved_or_malformed_git_revision() {
        for fragment in ["", "main", "0123456789abcdef0123456789abcdef0123456g"] {
            let lock = VALID_LOCK.replace("0123456789abcdef0123456789abcdef01234567", fragment);
            assert!(selected_git_packages(lock.as_bytes(), &[DEPENDENCY]).is_err());
        }
    }

    #[test]
    fn accepts_compiled_workspace_lockfile() {
        let lock = include_bytes!("../../../../../Cargo.lock");
        let selected = selected_git_packages(
            lock,
            &[
                "mere-p2panda-net",
                "mere-transport",
                "muniment",
                "stickleback",
            ],
        )
        .unwrap();
        assert_eq!(selected.len(), 4);
    }
}
