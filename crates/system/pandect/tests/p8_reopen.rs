// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! P8's first probe on a minimal surface with no kernel (engine split D245):
//! a headless assertion edit survives reopen at the same pin, and the
//! surface's deltas and the engine store's share one journal, one cursor and
//! one digest.

mod support;

use std::collections::BTreeSet;
use std::path::Path;
use std::time::UNIX_EPOCH;

use mere::dataset::View;
use mere::model::{Author, JournalDelta};
use muniment::DirectoryBackend;
use muniment::journal::DigestChain;
use pandect::{GraphSession, MereSessions, PinnedView, SessionId};
use support::{Surface, SurfaceDelta};
use uuid::Uuid;

type Sessions = MereSessions<DirectoryBackend, Surface>;

fn has_predicate(pin: &PinnedView<Surface>, predicate: &str) -> bool {
    pin.dataset()
        .quads(View::Live)
        .any(|q| q.predicate.as_str() == predicate)
}

fn rows(pin: &PinnedView<Surface>, view: View) -> BTreeSet<String> {
    pin.dataset().quads(view).map(|q| q.to_string()).collect()
}

async fn reopen(dir: &Path, id: SessionId) -> GraphSession<DirectoryBackend, Surface> {
    Sessions::new_with(DirectoryBackend::open(dir).unwrap())
        .open_with(id)
        .await
        .unwrap()
}

fn live_digest(session: &GraphSession<DirectoryBackend, Surface>) -> String {
    let pin = session.pin(session.journal().live_cursor()).unwrap();
    pin.digest().to_owned()
}

/// Copy the session directory and rewrite one journal line in the copy.
fn tamper(dir: &Path, line: usize, from: &str, to: &str) -> tempfile::TempDir {
    let copy = tempfile::tempdir().unwrap();
    copy_dir(dir, copy.path());
    let journal = walk(copy.path())
        .into_iter()
        .find(|path| path.ends_with("journal.jsonl"))
        .expect("a journal file");
    let text = std::fs::read_to_string(&journal).unwrap();
    let mut lines: Vec<String> = text.lines().map(str::to_owned).collect();
    assert!(lines[line].contains(from), "line {line} holds {from}");
    lines[line] = lines[line].replacen(from, to, 1);
    std::fs::write(&journal, lines.join("\n") + "\n").unwrap();
    copy
}

fn walk(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            out.extend(walk(&path));
        } else {
            out.push(path);
        }
    }
    out
}

fn copy_dir(from: &Path, to: &Path) {
    for entry in std::fs::read_dir(from).unwrap() {
        let path = entry.unwrap().path();
        let target = to.join(path.file_name().unwrap());
        if path.is_dir() {
            std::fs::create_dir_all(&target).unwrap();
            copy_dir(&path, &target);
        } else {
            std::fs::copy(&path, &target).unwrap();
        }
    }
}

#[test]
fn assertion_edit_survives_reopen_at_the_same_pin() {
    pollster::block_on(async {
        let dir = tempfile::tempdir().unwrap();
        let sessions = Sessions::new_with(DirectoryBackend::open(dir.path()).unwrap());
        let author = Author::person("p8-probe");
        let id = sessions
            .mint(author.clone(), None)
            .await
            .unwrap()
            .session_id;
        let mut session = sessions.open_with(id).await.unwrap();
        let (a, b) = (Uuid::from_u128(0x801), Uuid::from_u128(0x802));
        let (old_id, _) = session
            .edit_now(author.clone(), |graph| {
                graph.show(a, "https://p8.test/a");
                graph.show(b, "https://p8.test/b");
                graph.assert(a, b, "https://p8.test/withdrawn")
            })
            .unwrap();
        let old_quad = "https://p8.test/withdrawn";
        let before = session.pin(session.journal().live_cursor()).unwrap();
        assert!(has_predicate(&before, old_quad));
        session
            .edit_now(author, |graph| {
                assert!(graph.withdraw(a, b, &old_id, 30));
                graph.assert(a, b, "https://p8.test/retained");
            })
            .unwrap();
        session.flush(UNIX_EPOCH).await.unwrap();
        let pin = session.pin(session.journal().live_cursor()).unwrap();
        let identity = (pin.session(), pin.cursor(), pin.digest().to_owned());
        let live = rows(&pin, View::Live);
        let history = rows(&pin, View::History);
        assert!(!has_predicate(&pin, old_quad));
        assert!(has_predicate(&pin, "https://p8.test/retained"));
        assert_ne!(
            rows(&before, View::Live),
            live,
            "negative control: omitted edit"
        );
        assert_ne!(
            before.digest(),
            pin.digest(),
            "negative control: wrong cursor digest"
        );
        drop(session);
        drop(sessions);
        let reopened = reopen(dir.path(), id).await;
        let again = reopened.pin(reopened.journal().live_cursor()).unwrap();
        assert_eq!(
            (again.session(), again.cursor(), again.digest()),
            (identity.0, identity.1, identity.2.as_str())
        );
        assert_eq!(rows(&again, View::Live), live);
        assert_eq!(rows(&again, View::History), history);
        assert!(!has_predicate(&again, old_quad));
    });
}

#[test]
fn surface_and_resource_deltas_share_one_journal_cursor_and_digest() {
    pollster::block_on(async {
        let dir = tempfile::tempdir().unwrap();
        let sessions = Sessions::new_with(DirectoryBackend::open(dir.path()).unwrap());
        let author = Author::person("p8-probe");
        let id = sessions
            .mint(author.clone(), None)
            .await
            .unwrap()
            .session_id;
        let mut session = sessions.open_with(id).await.unwrap();
        let (a, b) = (Uuid::from_u128(0x901), Uuid::from_u128(0x902));
        let (_, applied) = session
            .edit_now(author.clone(), |graph| {
                graph.show(a, "https://p8.test/c");
                graph.show(b, "https://p8.test/d");
                graph.assert(a, b, "https://p8.test/linked")
            })
            .unwrap();
        session.flush(UNIX_EPOCH).await.unwrap();

        // One journal: the surface's bindings and the store's writes,
        // interleaved in the order they were made, under one cursor.
        let kinds: Vec<&str> = session
            .journal()
            .entries()
            .iter()
            .map(|entry| match &entry.delta {
                JournalDelta::Resource(_) => "resource",
                JournalDelta::Surface(SurfaceDelta::ShowResource { .. }) => "show",
                JournalDelta::Surface(SurfaceDelta::HideSurface { .. }) => "hide",
            })
            .collect();
        assert_eq!(kinds, ["resource", "show", "resource", "show", "resource"]);
        assert_eq!((applied.first.0, applied.end.0), (0, 5));
        assert_eq!(session.journal().live_cursor().0, 5);

        // One digest: the pin's names every entry of both kinds.
        let chain = DigestChain::of(session.journal().log()).unwrap();
        let digest = live_digest(&session);
        assert_eq!(chain.at(5).map(|hash| hash.to_hex()), Some(digest.clone()));
        // The directory has one writer: let go of it before reopening.
        drop(session);
        drop(sessions);
        let reopened = reopen(dir.path(), id).await;
        assert_eq!(reopened.journal().live_cursor().0, 5);
        assert_eq!(live_digest(&reopened), digest);
        assert_eq!(
            reopened.graph().shown(a),
            Some(mere::model::ResourceNode::for_term("https://p8.test/c").id())
        );
        drop(reopened);

        // Controls: a tampered surface entry and a tampered resource entry
        // each change the digest the reopened session names.
        let surface = tamper(
            dir.path(),
            1,
            &a.to_string(),
            &Uuid::from_u128(0x9ff).to_string(),
        );
        assert_ne!(live_digest(&reopen(surface.path(), id).await), digest);
        let resource = tamper(
            dir.path(),
            4,
            "https://p8.test/linked",
            "https://p8.test/other",
        );
        let forged = reopen(resource.path(), id).await;
        assert_ne!(live_digest(&forged), digest);
        let pin = forged.pin(forged.journal().live_cursor()).unwrap();
        assert!(has_predicate(&pin, "https://p8.test/other"));
        assert!(!has_predicate(&pin, "https://p8.test/linked"));
    });
}
