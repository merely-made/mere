// SPDX-License-Identifier: MPL-2.0
//! One same-directory replacement for encrypted authentication files.
//! Syncs the complete file before rename. Cold-reopen safety is qualified;
//! this does not promise survival of directory-metadata loss after power failure.
use personae::IdentityError;
use std::{io::Write, path::Path};

pub(super) fn write_json(path: &Path, value: &impl serde::Serialize) -> Result<(), IdentityError> {
    let bytes = serde_json::to_vec(value)
        .map_err(|_| IdentityError::Backend("encode encrypted authentication file".into()))?;
    let parent = path
        .parent()
        .ok_or_else(|| IdentityError::Backend("authentication file has no directory".into()))?;
    std::fs::create_dir_all(parent).map_err(storage)?;
    let mut nonce = [0; 16];
    getrandom::fill(&mut nonce)
        .map_err(|_| IdentityError::Backend("authentication randomness unavailable".into()))?;
    let tmp = parent.join(format!(
        ".authentication-{}.tmp",
        blake3::hash(&nonce).to_hex()
    ));
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&tmp).map_err(storage)?;
    let result = (|| {
        file.write_all(&bytes).map_err(storage)?;
        file.sync_all().map_err(storage)?;
        drop(file);
        std::fs::rename(&tmp, path).map_err(storage)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(tmp);
    }
    result
}
fn storage(_: std::io::Error) -> IdentityError {
    IdentityError::Backend("encrypted authentication file replacement failed".into())
}
