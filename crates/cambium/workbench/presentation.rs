/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Controlled, transient workspace presentation, separate from the saved tree.
//! The host decides which stacks give way, their minimum widths, and overlay
//! geometry. No viewport threshold or document measure belongs in this contract.

use crate::{TabStack, TileId, TilePath};

/// Host-provided drawer geometry in the workspace root's coordinate space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DrawerGeometry {
    pub trigger: (f32, f32, f32, f32),
    pub panel_size: (f32, f32),
    pub bounds: (f32, f32, f32, f32),
}

/// A folded stack's rail and its controlled open state.
#[derive(Clone, Debug, PartialEq)]
pub struct CollapsedStack {
    pub label: String,
    pub rail_width: f32,
    pub open: bool,
    /// `Some` opens above the workspace; `None` opens in the original split.
    pub drawer: Option<DrawerGeometry>,
}

/// Presentation for the stack containing a stable, host-assigned tile anchor.
#[derive(Clone, Debug, PartialEq)]
pub struct StackPresentation {
    pub anchor: TileId,
    pub min_width: f32,
    pub collapsed: Option<CollapsedStack>,
}

/// Temporary split shares. Removing this entry restores the canonical shares.
#[derive(Clone, Debug, PartialEq)]
pub struct SplitPresentation {
    pub path: TilePath,
    pub fractions: Vec<f32>,
}

/// A rail/drawer intent; it never mutates tabs, active selection, or split ratios.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresentationEvent {
    Opened(TileId),
    Closed(TileId),
}

/// Non-destructive, host-controlled presentation of a canonical tile tree.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WorkbenchPresentation {
    pub stacks: Vec<StackPresentation>,
    pub splits: Vec<SplitPresentation>,
    /// Optional source stack to focus after a close. Hosts may additionally
    /// request focus on a particular editor inside that stack.
    pub return_focus: Option<TileId>,
    /// An edge request, cleared when another rail opens.
    pub focus_return_requested: bool,
}

impl WorkbenchPresentation {
    pub fn stack(&self, stack: &TabStack) -> Option<&StackPresentation> {
        self.stacks
            .iter()
            .find(|entry| stack.tabs.iter().any(|tile| tile.id == entry.anchor))
    }

    /// Invalid paths, lengths, or fractions fall back to the canonical shares.
    pub fn fractions(&self, path: &[usize], count: usize) -> Option<&[f32]> {
        self.splits
            .iter()
            .find(|entry| {
                entry.path.0 == path
                    && entry.fractions.len() == count
                    && entry
                        .fractions
                        .iter()
                        .all(|value| value.is_finite() && *value >= 0.0)
                    && entry.fractions.iter().any(|value| *value > 0.0)
            })
            .map(|entry| entry.fractions.as_slice())
    }

    /// Apply a controlled rail transition. Unknown or already-set states are
    /// unchanged; the canonical tree is deliberately not an argument.
    pub fn apply(&mut self, event: PresentationEvent) -> bool {
        let (anchor, open) = match event {
            PresentationEvent::Opened(anchor) => (anchor, true),
            PresentationEvent::Closed(anchor) => (anchor, false),
        };
        let Some(collapsed) = self
            .stacks
            .iter_mut()
            .find(|entry| entry.anchor == anchor)
            .and_then(|entry| entry.collapsed.as_mut())
        else {
            return false;
        };
        if collapsed.open == open {
            return false;
        }
        collapsed.open = open;
        self.focus_return_requested = !open;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ContentSource, Tile, TileTree};

    #[test]
    fn folding_and_opening_preserve_the_entire_canonical_tree() {
        let tree = TileTree::Stack(TabStack {
            tabs: vec![Tile {
                id: TileId(7),
                title: "Reading".into(),
                content: ContentSource::Open {
                    kind: "test".into(),
                    id: "7".into(),
                },
                accent: None,
            }],
            active: 0,
        });
        let snapshot = tree.clone();
        let mut presentation = WorkbenchPresentation {
            stacks: vec![StackPresentation {
                anchor: TileId(7),
                min_width: 280.0,
                collapsed: Some(CollapsedStack {
                    label: "Reading".into(),
                    rail_width: 28.0,
                    open: false,
                    drawer: None,
                }),
            }],
            ..Default::default()
        };
        assert!(presentation.apply(PresentationEvent::Opened(TileId(7))));
        assert!(!presentation.focus_return_requested);
        assert!(presentation.apply(PresentationEvent::Closed(TileId(7))));
        assert!(presentation.focus_return_requested);
        assert!(!presentation.apply(PresentationEvent::Opened(TileId(9))));
        assert_eq!(tree, snapshot);
        assert_eq!(
            presentation
                .stack(match &tree {
                    TileTree::Stack(s) => s,
                    _ => unreachable!(),
                })
                .unwrap()
                .min_width,
            280.0
        );
    }

    #[test]
    fn transient_fractions_validate_and_leave_canonical_shares_to_the_host() {
        let mut presentation = WorkbenchPresentation::default();
        presentation.splits.push(SplitPresentation {
            path: TilePath(vec![]),
            fractions: vec![0.2, 0.8],
        });
        assert_eq!(presentation.fractions(&[], 2), Some(&[0.2, 0.8][..]));
        assert_eq!(presentation.fractions(&[0], 2), None);
        assert_eq!(presentation.fractions(&[], 3), None);
        presentation.splits[0].fractions[0] = f32::NAN;
        assert_eq!(presentation.fractions(&[], 2), None);
    }
}
