// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The reopen tests' on-disk form: a session's `graph.json` snapshot, written
//! whole and read back through the checked loader. Test support only; the
//! kernel has no persistence owner of its own (Q017, 2026-10-26).

use kernel::graph::Graph;
use kernel::persistence::GraphSnapshot;
use std::path::Path;
use std::{fs, io};

const GRAPH_FILE: &str = "graph.json";

pub fn save_graph(directory: &Path, graph: &Graph) -> io::Result<()> {
    let json = serde_json::to_string_pretty(&graph.to_snapshot())
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let tmp = directory.join(format!("{GRAPH_FILE}.tmp"));
    fs::write(&tmp, json)?;
    fs::rename(&tmp, directory.join(GRAPH_FILE))
}

pub fn load_graph(directory: &Path) -> io::Result<Option<Graph>> {
    let path = directory.join(GRAPH_FILE);
    if !path.exists() {
        return Ok(None);
    }
    let snapshot: GraphSnapshot = serde_json::from_str(&fs::read_to_string(&path)?)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    Graph::try_from_snapshot(&snapshot)
        .map(Some)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}
