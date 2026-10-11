// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The device fabric's public stored pieces: opaque signed grants. This
//! host's delegated-device identity and the retained remote-auth wrapping keys
//! are secrets, in castellan (dramatis repo plan, DR-B, ruling D8).

use std::path::Path;
use std::{fs, io};

use super::DeviceId;
use super::io::save_bytes_atomic;
use super::paths::device_grant_path;

/// Load the opaque grant payload for one device, or `None` when absent.
pub fn load_device_grant(data_root: &Path, device_id: DeviceId) -> io::Result<Option<Vec<u8>>> {
    let path = device_grant_path(data_root, device_id);
    match fs::read(&path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}

/// Save the opaque grant payload for one device atomically.
pub fn save_device_grant(data_root: &Path, device_id: DeviceId, bytes: &[u8]) -> io::Result<()> {
    let path = device_grant_path(data_root, device_id);
    save_bytes_atomic(&path, bytes)
}

#[cfg(test)]
mod tests {
    use super::super::test_support::*;
    use super::*;

    #[test]
    fn opaque_device_grant_round_trips() {
        let root = temp_data_root("grant");
        let bytes = vec![0xa1, 0x62, 0x6f, 0x6b];
        save_device_grant(&root, fixture_device(), &bytes).unwrap();
        let restored = load_device_grant(&root, fixture_device()).unwrap().unwrap();
        assert_eq!(restored, bytes);
        let _ = fs::remove_dir_all(&root);
    }
}
