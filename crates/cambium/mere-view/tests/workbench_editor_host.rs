// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
#![cfg(feature = "workbench-editor")]
//! A second, windowless host embeds the Forme controls over its own owner session.
//! Persistence acknowledgements stay with this host; there is no Graphshell state.
use cambium::{el, lens};
use cambium_genet_winit_host::Harness;
use mere_view::{
    MereViewElement,
    workbench_editor::{self, Effect, Snapshot, WorkbenchEditor},
};
use platen::workspace::{FormeSession, FormeWorkspace, NoPreferences, SaveKind};
use taproot::Selector;
use uuid::Uuid;

struct Host {
    editor: WorkbenchEditor,
    session: FormeSession<NoPreferences>,
    graph_id: Uuid,
    stored_bytes: Option<Vec<u8>>,
    prepared_graph_ids: Vec<Uuid>,
    writes: usize,
    fail_next_write: bool,
    notice: String,
}

impl Host {
    fn new() -> Self {
        let graph_id = Uuid::from_u128(0x4f524d45);
        Self {
            editor: WorkbenchEditor::default(),
            session: FormeSession::new(FormeWorkspace::new(graph_id)),
            graph_id,
            stored_bytes: None,
            prepared_graph_ids: Vec::new(),
            writes: 0,
            fail_next_write: false,
            notice: String::new(),
        }
    }

    fn sync_editor(&mut self) {
        self.editor.set_snapshot(Snapshot {
            editing: self.session.editing(),
            dirty: self.session.dirty(),
            busy: self.session.busy(),
            can_undo: self.session.can_undo(),
            can_redo: self.session.can_redo(),
            visible: self.session.view().visible,
            notice: self.notice.clone(),
        });
    }

    /// Route the component's real effect through the host-owned session and
    /// acknowledge a local byte-store write. A failed write leaves the prior
    /// bytes untouched and the Forme draft available for another Apply.
    fn accept_control(&mut self) {
        let effect = self
            .editor
            .take_effect()
            .expect("control emitted an effect");
        match effect {
            Effect::Unlock => {
                self.session.unlock().unwrap();
                self.notice = "Layout unlocked".into();
            },
            Effect::Apply => {
                let proposal = self
                    .session
                    .prepare_save(SaveKind::Apply, 10)
                    .expect("host prepares the accepted draft");
                let proposal_graph = proposal.graph_id();
                let bytes = proposal.bytes().expect("prepared owner record serializes");
                self.prepared_graph_ids.push(proposal_graph);
                let result = if std::mem::take(&mut self.fail_next_write) {
                    Err("fixture byte-store failure".to_string())
                } else {
                    self.stored_bytes = Some(bytes);
                    self.writes += 1;
                    Ok(())
                };
                match self.session.finish_save(proposal, result) {
                    Ok(()) => self.notice = "Layout saved".into(),
                    Err(error) => self.notice = format!("Layout save unconfirmed: {error}"),
                }
            },
            Effect::Undo => {
                let proposal = self
                    .session
                    .prepare_save(SaveKind::Undo, 11)
                    .expect("host prepares an available saved undo");
                let bytes = proposal.bytes().expect("undo record serializes");
                self.prepared_graph_ids.push(proposal.graph_id());
                self.stored_bytes = Some(bytes);
                self.writes += 1;
                self.session
                    .finish_save(proposal, Ok(()))
                    .expect("owner acknowledges undo");
                self.notice = "Layout undo saved".into();
            },
            other => panic!("unexpected workbench action: {other:?}"),
        }
        self.sync_editor();
    }
}

fn root(_: &Host) -> MereViewElement<Host, ()> {
    Box::new(el(
        "main",
        lens(
            |editor: &mut WorkbenchEditor| workbench_editor::view(editor),
            |host: &mut Host| &mut host.editor,
        ),
    ))
}

#[test]
fn second_host_routes_unlock_apply_failure_retry_and_undo_through_owner_session() {
    let mut host = Harness::new(
        "main {display:flex;flex-direction:column;padding:10px} button {display:block;padding:6px}",
        Host::new(),
        root as fn(&Host) -> MereViewElement<Host, ()>,
    );
    host.layout_at(800.0, 600.0);
    let original_bounds = host.state().session.committed().bounds;

    assert!(host.click_on(&Selector::role("button").containing("Unlock forme")));
    host.update(|host| {
        host.accept_control();
        host.session
            .edit(None, 1, |workspace| {
                workspace.bounds[0] -= 24.0;
                Ok(())
            })
            .unwrap();
        host.notice = "Layout changed by the host's divider gesture".into();
        host.sync_editor();
    });
    assert!(host.state().session.dirty());
    let edited_bounds = host.state().session.view().bounds;

    assert!(host.click_on(&Selector::role("button").containing("Lock and apply")));
    host.update(|host| {
        host.fail_next_write = true;
        host.accept_control();
    });
    assert!(host.state().session.editing());
    assert!(host.state().session.dirty());
    assert_eq!(host.state().session.committed().bounds, original_bounds);
    assert_eq!(host.state().stored_bytes, None);
    assert_eq!(host.state().prepared_graph_ids, vec![host.state().graph_id]);
    assert!(host.state().notice.contains("unconfirmed"));

    assert!(host.click_on(&Selector::role("button").containing("Lock and apply")));
    host.update(Host::accept_control);
    assert_eq!(host.state().writes, 1);
    assert_eq!(
        host.state().prepared_graph_ids,
        vec![host.state().graph_id, host.state().graph_id]
    );
    assert_eq!(host.state().session.committed().bounds, edited_bounds);
    assert!(!host.state().session.editing());
    assert!(host.state().session.can_undo());
    assert!(host.click_on(&Selector::role("button").containing("Undo saved forme change")));
    host.update(Host::accept_control);
    assert_eq!(host.state().session.committed().bounds, original_bounds);
    assert!(host.state().session.can_redo());
}
