// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
#![cfg(feature = "presentation-editor")]
//! An independent host embeds rule controls, routes their effects to Scenograph,
//! and supplies durable acknowledgements without a Graphshell or browser owner.
use cambium::{el, lens};
use cambium_genet_winit_host::Harness;
use mere_view::{
    MereViewElement,
    presentation_editor::{self, Effect, PresentationEditor, Snapshot},
};
use scenograph::presentation::{
    PrimitiveKind,
    editor::{RuleWorkspace, SaveKind, SavedState},
};
use taproot::Selector;

struct Host {
    editor: PresentationEditor,
    workspace: RuleWorkspace,
    stored: SavedState,
    fail_next: bool,
    conflict: bool,
    notice: String,
}
impl Host {
    fn new() -> Self {
        let stored = SavedState::empty();
        let mut host = Self {
            editor: PresentationEditor::default(),
            workspace: RuleWorkspace::new(stored.clone()).unwrap(),
            stored,
            fail_next: false,
            conflict: false,
            notice: String::new(),
        };
        host.sync();
        host
    }
    fn sync(&mut self) {
        self.editor.set_snapshot(Snapshot {
            available: true,
            editing: self.workspace.editing(),
            dirty: self.workspace.dirty(),
            busy: self.workspace.busy(),
            can_undo: self.workspace.can_undo(),
            can_redo: self.workspace.can_redo(),
            can_reload: self.conflict,
            notice: self.notice.clone(),
        });
    }
    fn accept(&mut self) {
        let effect = self
            .editor
            .take_effect()
            .expect("permitted control emitted an effect");
        match effect {
            Effect::Begin => self.workspace.begin().unwrap(),
            Effect::AddNode | Effect::AddRelation | Effect::AddField => {
                let kind = match effect {
                    Effect::AddNode => PrimitiveKind::Node,
                    Effect::AddRelation => PrimitiveKind::Relation,
                    _ => PrimitiveKind::Field,
                };
                self.workspace
                    .edit(None, 1, |draft| {
                        draft.add(kind);
                    })
                    .unwrap();
            },
            Effect::Apply | Effect::Undo | Effect::Redo => {
                if effect != Effect::Apply && self.workspace.editing() {
                    self.workspace.undo_draft(effect == Effect::Redo).unwrap();
                } else {
                    let kind = match effect {
                        Effect::Apply => SaveKind::Apply,
                        Effect::Undo => SaveKind::Undo,
                        _ => SaveKind::Redo,
                    };
                    let proposal = self.workspace.prepare(kind).unwrap();
                    let result = if std::mem::take(&mut self.fail_next) {
                        Err("fixture storage refusal".to_owned())
                    } else {
                        self.stored = proposal.state;
                        Ok(())
                    };
                    self.notice = match self.workspace.finish(proposal.id, result) {
                        Ok(true) => "Saved".into(),
                        other => format!("Save unconfirmed: {other:?}"),
                    };
                }
            },
            Effect::Discard => self.workspace.discard().unwrap(),
            Effect::Reload => {
                self.workspace.reconcile(self.stored.clone()).unwrap();
                self.conflict = false;
            },
        }
        self.sync();
    }
}

fn root(_: &Host) -> MereViewElement<Host, ()> {
    Box::new(el(
        "main",
        lens(
            |editor: &mut PresentationEditor| presentation_editor::view(editor),
            |host: &mut Host| &mut host.editor,
        ),
    ))
}

#[test]
fn second_host_preserves_private_draft_on_failure_then_applies_and_undoes() {
    let mut host = Harness::new(
        "main {display:flex;flex-direction:column;padding:10px} button {display:block;padding:6px}",
        Host::new(),
        root as fn(&Host) -> MereViewElement<Host, ()>,
    );
    host.layout_at(800.0, 600.0);
    assert!(host.click_on(&Selector::role("button").containing("Begin rule draft")));
    host.update(Host::accept);
    assert!(host.click_on(&Selector::role("button").containing("Add node rule")));
    host.update(Host::accept);
    assert!(host.state().workspace.dirty());
    assert!(host.click_on(&Selector::role("button").containing("Apply rule draft")));
    host.update(|host| {
        host.fail_next = true;
        host.accept();
    });
    assert!(host.state().workspace.dirty());
    assert!(host.state().workspace.committed().rules.is_empty());
    assert!(host.state().stored.rules.rules.is_empty());
    assert!(host.state().notice.contains("unconfirmed"));
    assert!(host.click_on(&Selector::role("button").containing("Apply rule draft")));
    host.update(Host::accept);
    assert_eq!(host.state().stored.rules.rules.len(), 1);
    assert_eq!(
        host.state().workspace.committed(),
        &host.state().stored.rules
    );
    assert!(!host.state().workspace.editing());
    assert!(host.click_on(&Selector::role("button").containing("Undo saved rules")));
    host.update(Host::accept);
    assert!(host.state().stored.rules.rules.is_empty());
    assert!(host.state().workspace.can_redo());
}

#[test]
fn second_host_reconciles_conflict_without_losing_private_draft() {
    let mut host = Harness::new(
        "main {display:flex;flex-direction:column;padding:10px} button {display:block;padding:6px}",
        Host::new(),
        root as fn(&Host) -> MereViewElement<Host, ()>,
    );
    host.layout_at(800.0, 600.0);
    assert!(host.click_on(&Selector::role("button").containing("Begin rule draft")));
    host.update(Host::accept);
    assert!(host.click_on(&Selector::role("button").containing("Add field rule")));
    host.update(Host::accept);
    let private = host.state().workspace.draft().unwrap().clone();
    host.update(|host| {
        let mut other = RuleWorkspace::new(host.stored.clone()).unwrap();
        other.begin().unwrap();
        other
            .edit(None, 2, |draft| {
                draft.add(PrimitiveKind::Node);
            })
            .unwrap();
        host.stored = other.prepare(SaveKind::Apply).unwrap().state;
        host.conflict = true;
        host.sync();
    });
    // Conflict refuses writes until the independent host refreshes its baseline.
    let _ = host.click_on(&Selector::role("button").containing("Apply rule draft"));
    assert!(!host.state().editor.has_effects());
    assert!(host.click_on(&Selector::role("button").containing("Reconcile saved rules")));
    host.update(Host::accept);
    assert_eq!(host.state().workspace.draft(), Some(&private));
    assert_eq!(
        host.state().workspace.committed(),
        &host.state().stored.rules
    );
    assert!(host.click_on(&Selector::role("button").containing("Apply rule draft")));
    host.update(Host::accept);
    assert_eq!(
        host.state().stored.rules.rules[0].target.kind,
        PrimitiveKind::Field
    );
    assert_eq!(host.state().stored.generation, 2);
    // The reconciled remote version remains available through saved undo.
    assert!(host.click_on(&Selector::role("button").containing("Undo saved rules")));
    host.update(Host::accept);
    assert_eq!(
        host.state().stored.rules.rules[0].target.kind,
        PrimitiveKind::Node
    );
}
