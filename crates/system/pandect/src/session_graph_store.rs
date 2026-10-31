// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Where a session's `graph.json` lives. [`GraphSession`](crate::GraphSession)
//! writes and loads it; this module holds only the file name and the
//! conventional path, which sidecar modules share.
//!
//! Native-only, like the filesystem layout it names.

use std::path::{Path, PathBuf};

use incipit::SessionId;

use crate::engine_profile_store::SESSIONS_DIR;

/// The graph file name under a session directory.
pub const GRAPH_FILE: &str = "graph.json";

/// The conventional per-session graph path
/// `<data_root>/sessions/<session_id>/graph.json`, matching the `ManifestStore`
/// and engine-profile layout.
pub fn session_graph_path(data_root: &Path, session_id: SessionId) -> PathBuf {
    data_root
        .join(SESSIONS_DIR)
        .join(session_id.as_uuid().to_string())
        .join(GRAPH_FILE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn session_graph_path_follows_the_sessions_convention() {
        let id = SessionId(Uuid::from_u128(0x4242));
        let path = session_graph_path(Path::new("/data"), id);
        let expected = Path::new("/data")
            .join("sessions")
            .join(id.as_uuid().to_string())
            .join("graph.json");
        assert_eq!(path, expected);
    }
}
