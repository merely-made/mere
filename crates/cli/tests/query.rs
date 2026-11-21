// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::fs;
use std::path::Path;
use std::process::{Child, Command, Output};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{Value, json};

const MERE: &str = env!("CARGO_BIN_EXE_mere");
const FIXTURES: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../graph/engine/examples/fixtures"
);
const ROWS: &[u8] =
    b"[[\"Ada\",\"Library catalog\"],[\"Ada\",\"Publisher\"],[\"Grace\",\"Publisher\"]]\n";

struct Server(Child);
impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn run(dir: &Path, args: &[&str]) -> Output {
    Command::new(MERE)
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn local_query_preserves_unbound_and_escaped_cells() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("data.ttl"),
        "<urn:book> <urn:title> \"Ada | Grace\\nVol. 1\" .",
    )
    .unwrap();
    fs::write(
        dir.path().join("read.rq"),
        "SELECT ?title ?missing WHERE { <urn:book> <urn:title> ?title }",
    )
    .unwrap();
    let result = run(
        dir.path(),
        &["query", "--graph", "data.ttl", "--query", "read.rq"],
    );
    assert!(result.status.success(), "{:?}", result.stderr);
    assert_eq!(
        serde_json::from_slice::<Value>(&result.stdout).unwrap(),
        json!([["Ada | Grace\nVol. 1", null]])
    );
}

#[test]
fn iroh_invitation_limits_access_across_repeated_queries() {
    let dir = tempfile::tempdir().unwrap();
    let file = |name| dir.path().join(name);
    let graph = Path::new(FIXTURES).join("paper-authors.trig");
    fs::copy(
        Path::new(FIXTURES).join("paper-authors.rq"),
        file("allowed.rq"),
    )
    .unwrap();
    fs::write(file("other.rq"), "SELECT * WHERE { ?s ?p ?o }").unwrap();
    let mut server = Server(
        Command::new(MERE)
            .current_dir(dir.path())
            .args(["serve", "--graph"])
            .arg(graph)
            .args([
                "--via",
                "iroh",
                "--allow-query",
                "allowed.rq",
                "--invite",
                "paper.peer",
            ])
            .spawn()
            .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(25);
    while !file("paper.peer").exists() {
        assert!(server.0.try_wait().unwrap().is_none(), "owner exited early");
        assert!(Instant::now() < deadline, "owner failed to advertise");
        thread::sleep(Duration::from_millis(50));
    }
    let query = |invite, query| run(dir.path(), &["query", "--peer", invite, "--query", query]);
    let expect_rows = || {
        let result = query("paper.peer", "allowed.rq");
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(result.stdout, ROWS);
    };
    expect_rows();
    let other = query("paper.peer", "other.rq");
    assert!(!other.status.success() && other.stdout.is_empty());
    assert!(String::from_utf8_lossy(&other.stderr).contains("not enabled"));
    let mut edited: Value = serde_json::from_slice(&fs::read(file("paper.peer")).unwrap()).unwrap();
    edited["token"][0] = json!(edited["token"][0].as_u64().unwrap() ^ 1);
    fs::write(file("edited.peer"), serde_json::to_vec(&edited).unwrap()).unwrap();
    let invalid = query("edited.peer", "allowed.rq");
    assert!(!invalid.status.success() && invalid.stdout.is_empty());
    expect_rows();
}
