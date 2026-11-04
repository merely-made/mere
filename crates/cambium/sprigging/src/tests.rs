// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use std::cell::Cell;
use std::rc::Rc;

/// A leaf whose dirtiness is externally observable, so the retention gate can
/// be driven from the test.
struct FlagLeaf {
    dirty: Rc<Cell<bool>>,
}

impl Leaf for FlagLeaf {
    fn measure(&mut self, _known: SizeHint, _available: SizeHint) -> Size {
        Size {
            width: 8.0,
            height: 8.0,
        }
    }
    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        cx.fill_rect(0.0, 0.0, 8.0, 8.0, ColorF::WHITE);
        self.dirty.set(false);
    }
    fn paint_dirty(&self) -> bool {
        self.dirty.get()
    }
}

#[test]
fn render_into_respects_the_paint_dirty_gate() {
    let dirty = Rc::new(Cell::new(true));
    let mut reg: LeafRegistry<u64> = LeafRegistry::new();
    reg.insert(
        3,
        Box::new(FlagLeaf {
            dirty: dirty.clone(),
        }),
    );
    let mut out = RenderedLeaves::new();
    let sized = |_k: u64| {
        Some(Size {
            width: 8.0,
            height: 8.0,
        })
    };

    // Fresh + dirty -> painted.
    assert_eq!(
        reg.render_into(sized, &mut out),
        1,
        "fresh dirty leaf paints"
    );
    assert!(out.get(3).is_some());

    // Clean + cached -> skipped (the gate).
    assert_eq!(
        reg.render_into(sized, &mut out),
        0,
        "clean, cached leaf is not repainted"
    );

    // Size change without paint_dirty (container relayout) -> repainted.
    let bigger = |_k: u64| {
        Some(Size {
            width: 16.0,
            height: 16.0,
        })
    };
    assert_eq!(
        reg.render_into(bigger, &mut out),
        1,
        "a resized leaf repaints even when it is not paint_dirty"
    );
    assert_eq!(
        reg.render_into(bigger, &mut out),
        0,
        "and is stable once cached at the new size"
    );

    // Re-dirtied -> repainted.
    dirty.set(true);
    assert_eq!(
        reg.render_into(sized, &mut out),
        1,
        "a re-dirtied leaf repaints"
    );

    // No size (not laid out this frame) -> skipped even when dirty.
    dirty.set(true);
    let no_size = |_k: u64| None;
    assert_eq!(
        reg.render_into(no_size, &mut out),
        0,
        "a leaf with no layout size is skipped"
    );

    // retain_keys drops stale buffers.
    out.retain_keys(|k| k != 3);
    assert!(out.get(3).is_none());

    reg.insert(
        4,
        Box::new(FlagLeaf {
            dirty: dirty.clone(),
        }),
    );
    reg.retain(|key| *key == 4);
    assert!(!reg.contains(&3));
    assert!(reg.contains(&4));
}

#[test]
fn replacement_repaints_even_when_clean_and_cannot_inherit_a_scene() {
    let size = Size {
        width: 8.0,
        height: 8.0,
    };
    let sized = |_| Some(size);
    let mut reg = LeafRegistry::new();
    let mut scene = SceneSlot::new(size);
    scene.set_scene(vello::Scene::new());
    reg.insert(3, Box::new(scene));
    let mut out = RenderedLeaves::new();
    assert_eq!(reg.render_into(sized, &mut out), 1);
    let scene_epoch = out.scenes().next().unwrap().2;
    let old_life = reg.lifetime(&3).unwrap();

    for dirty in [false, true] {
        reg.insert(
            3,
            Box::new(FlagLeaf {
                dirty: Rc::new(Cell::new(dirty)),
            }),
        );
        assert_ne!(reg.lifetime(&3).as_ref(), Some(&old_life));
        assert_eq!(reg.render_into(sized, &mut out), 1);
        assert_eq!(out.scenes().count(), 0, "no former contributor scene");
        assert!(matches!(out.get(3).unwrap(), [PaintCmd::DrawRect(_)]));
        assert!(out.path_a_entries().next().unwrap().1 > scene_epoch);
        assert_eq!(reg.render_into(sized, &mut out), 0);
    }
    let current = reg.lifetime(&3).unwrap();
    reg.get_mut_as::<FlagLeaf>(&3).unwrap().dirty.set(true);
    assert_eq!(reg.lifetime(&3).as_ref(), Some(&current));
    assert_eq!(reg.render_into(sized, &mut out), 1);

    // A detached old registry must not authorize a new insertion.
    let old = std::mem::take(&mut reg);
    reg.insert(
        3,
        Box::new(FlagLeaf {
            dirty: Rc::new(Cell::new(false)),
        }),
    );
    assert_ne!(reg.lifetime(&3), old.lifetime(&3));
    assert_eq!(reg.render_into(|_| None, &mut out), 0);
    assert!(
        out.get(3).is_none(),
        "replacement without layout drops stale paint"
    );
    assert_eq!(reg.render_into(sized, &mut out), 1);
    let epoch = out.path_a_entries().next().unwrap().1;
    out = RenderedLeaves::new();
    assert_eq!(reg.render_into(sized, &mut out), 1);
    assert!(
        out.path_a_entries().next().unwrap().1 > epoch,
        "cache reset cannot alias a host fragment epoch"
    );
    reg.remove(&3);
    assert_eq!(reg.render_into(sized, &mut out), 0);
    assert!(out.is_empty());
}

#[test]
fn swatch_measures_paints_and_clears_dirty() {
    let mut reg: LeafRegistry<u64> = LeafRegistry::new();
    reg.insert(
        1,
        Box::new(Swatch::new(
            ColorF::WHITE,
            Size {
                width: 10.0,
                height: 4.0,
            },
        )),
    );

    let leaf = reg.get_mut(&1).expect("leaf present");
    assert!(leaf.paint_dirty(), "a fresh leaf starts paint-dirty");

    let size = leaf.measure(SizeHint::default(), SizeHint::default());
    assert_eq!(
        size,
        Size {
            width: 10.0,
            height: 4.0
        }
    );

    let mut cmds = Vec::new();
    let mut cx = PaintCx::new(&mut cmds, size);
    leaf.paint(&mut cx);

    assert_eq!(cmds.len(), 1, "swatch emits exactly one rect");
    assert!(matches!(cmds[0], PaintCmd::DrawRect(_)));
    assert!(!leaf.paint_dirty(), "paint clears the paint-dirty gate");
}
