// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Atomic JSON/byte writes and tolerant reads shared by the store modules,
//! and by castellan's wallet custody, which writes into the same layout.

use std::path::Path;
use std::{fs, io};

use serde::{Deserialize, Serialize};

/// Read JSON at `path`, or `None` when the file is absent.
pub fn load_json_optional<T>(path: &Path) -> io::Result<Option<T>>
where
    T: for<'de> Deserialize<'de>,
{
    match fs::read_to_string(path) {
        Ok(text) => {
            let value = serde_json::from_str(&text)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            Ok(Some(value))
        },
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}

/// Write `value` as pretty JSON at `path` atomically.
pub fn save_json_atomic<T>(path: &Path, value: &T) -> io::Result<()>
where
    T: Serialize,
{
    let json = json_pretty_bytes(value)?;
    save_bytes_atomic(path, &json)
}

pub(super) fn json_pretty_bytes<T>(value: &T) -> io::Result<Vec<u8>>
where
    T: Serialize,
{
    serde_json::to_vec_pretty(value).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// Write `bytes` at `path` atomically, creating its directory.
pub fn save_bytes_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    crate::atomic_file::write_bytes_with_backup(path, bytes)
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn save_bytes_atomic_replaces_wallet_bytes_and_cleans_backup() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "mere-wallet-store-io-{}-{nonce}",
            std::process::id()
        ));
        let target = directory.join("wallet.bin");

        save_bytes_atomic(&target, b"first wallet payload").expect("first save");
        save_bytes_atomic(&target, b"replacement wallet payload").expect("replacement save");

        assert_eq!(
            fs::read(&target).expect("read wallet"),
            b"replacement wallet payload"
        );
        assert!(!target.with_extension("bin.previous").exists());
        assert_eq!(
            fs::read_dir(&directory)
                .expect("list wallet directory")
                .count(),
            1,
            "the successful replacement leaves only the wallet file"
        );
        fs::remove_dir_all(directory).expect("remove wallet directory");
    }
}
