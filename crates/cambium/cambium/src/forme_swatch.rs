// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A small projection of an existing arrangement. Hosts resolve stable tile
//! identities and retained passages; this component owns no layout or reading.
use crate::{OptionalAction, ShellControl, ShellSlot, el, shell_control};
use std::rc::Rc;
use workbench::{SplitAxis, Tile, TileTree};

#[derive(Clone, Debug, PartialEq)]
pub struct FormeSwatchTile<Id> {
    pub id: Id,
    pub title: String,
    pub passage: Option<String>,
    pub focused: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub struct FormeSwatchCell<Id> {
    /// Normalized left, top, right, bottom in this projection's aperture.
    pub bounds: [f32; 4],
    pub tabs: Vec<FormeSwatchTile<Id>>,
    pub active: usize,
}
#[derive(Clone, Debug, PartialEq)]
pub struct FormeSwatch<Id> {
    pub label: String,
    pub cells: Vec<FormeSwatchCell<Id>>,
}

impl<Id> FormeSwatch<Id> {
    pub fn from_tree(
        label: String,
        tree: &TileTree,
        mut resolve: impl FnMut(&Tile) -> Option<FormeSwatchTile<Id>>,
    ) -> Self {
        fn walk<Id>(
            tree: &TileTree,
            bounds: [f32; 4],
            resolve: &mut impl FnMut(&Tile) -> Option<FormeSwatchTile<Id>>,
            cells: &mut Vec<FormeSwatchCell<Id>>,
        ) {
            match tree {
                TileTree::Stack(stack) => {
                    let active_id = stack.tabs.get(stack.active).map(|t| t.id);
                    let mut active = 0;
                    let mut tabs = Vec::new();
                    for tile in &stack.tabs {
                        if let Some(tab) = resolve(tile) {
                            if Some(tile.id) == active_id {
                                active = tabs.len();
                            }
                            tabs.push(tab);
                        }
                    }
                    if !tabs.is_empty() {
                        cells.push(FormeSwatchCell {
                            bounds,
                            tabs,
                            active,
                        });
                    }
                },
                TileTree::Split { axis, children } => {
                    let weights: Vec<_> = children
                        .iter()
                        .map(|c| {
                            if c.fraction.is_finite() {
                                c.fraction.max(0.0)
                            } else {
                                0.0
                            }
                        })
                        .collect();
                    let sum: f32 = weights.iter().sum();
                    let mut start = 0.0;
                    for (child, weight) in children.iter().zip(weights) {
                        let fraction = if sum > 0.0 {
                            weight / sum
                        } else {
                            1.0 / children.len() as f32
                        };
                        let end = (start + fraction).min(1.0);
                        let [x, y, r, b] = bounds;
                        let region = match axis {
                            SplitAxis::Row => [x + (r - x) * start, y, x + (r - x) * end, b],
                            SplitAxis::Column => [x, y + (b - y) * start, r, y + (b - y) * end],
                        };
                        if fraction > 0.0 {
                            walk(&child.tree, region, resolve, cells);
                        }
                        start = end;
                    }
                },
            }
        }
        let mut cells = Vec::new();
        walk(tree, [0.0, 0.0, 1.0, 1.0], &mut resolve, &mut cells);
        Self { label, cells }
    }
}

/// Every tab remains reachable through a semantic button. A miniature does not
/// start guest sessions, resize the real tiles, or expose arrangement gestures.
pub fn forme_swatch<
    State: 'static,
    Action: 'static,
    Id: Clone + 'static,
    OA: OptionalAction<Action> + 'static,
>(
    swatch: &FormeSwatch<Id>,
    enter: impl Fn(&mut State, Id) -> OA + 'static,
) -> ShellSlot<State, Action> {
    let enter = Rc::new(enter);
    let cells: Vec<ShellSlot<State, Action>> = swatch
        .cells
        .iter()
        .map(|cell| {
            let tabs: Vec<ShellSlot<State, Action>> = cell
                .tabs
                .iter()
                .enumerate()
                .map(|(index, tile)| {
                    let enter = enter.clone();
                    let id = tile.id.clone();
                    shell_control(
                        tile.title.clone(),
                        format!("Enter tile: {}", tile.title),
                        ShellControl {
                            pressed: Some(index == cell.active),
                            ..Default::default()
                        },
                        move |state, _| enter(state, id.clone()),
                    )
                })
                .collect();
            let active = &cell.tabs[cell.active.min(cell.tabs.len() - 1)];
            let passage = active
                .passage
                .as_ref()
                .map(|s| s.split_whitespace().take(24).collect::<Vec<_>>().join(" "));
            let [x, y, r, b] = cell.bounds;
            Box::new(
                el(
                    "div",
                    (
                        el("div", tabs).attr("class", "forme-swatch-tabs"),
                        el("p", passage.unwrap_or_else(|| "No retained passage".into()))
                            .attr("class", "forme-swatch-passage"),
                        el("span", if active.focused { "Focused" } else { "" })
                            .attr("class", "forme-swatch-focus"),
                    ),
                )
                .attr(
                    "class",
                    if active.focused {
                        "forme-swatch-cell focused"
                    } else {
                        "forme-swatch-cell"
                    },
                )
                .attr(
                    "style",
                    format!(
                        "position:absolute;left:{}%;top:{}%;width:{}%;height:{}%;",
                        x * 100.0,
                        y * 100.0,
                        (r - x) * 100.0,
                        (b - y) * 100.0
                    ),
                ),
            ) as ShellSlot<State, Action>
        })
        .collect();
    Box::new(
        el("div", cells)
            .attr("class", "forme-swatch")
            .attr("role", "group")
            .attr("aria-label", swatch.label.clone()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use workbench::{ContentSource, TabStack, TileBranch, TileId};
    fn stack(ids: &[u64], active: usize) -> TileTree {
        TileTree::Stack(TabStack {
            active,
            tabs: ids
                .iter()
                .map(|id| Tile {
                    id: TileId(*id),
                    title: format!("Tile {id}"),
                    content: ContentSource::Open {
                        kind: "test".into(),
                        id: id.to_string(),
                    },
                    accent: None,
                })
                .collect(),
        })
    }
    #[test]
    fn nested_geometry_and_inactive_tabs_survive_the_projection() {
        let tree = TileTree::Split {
            axis: SplitAxis::Row,
            children: vec![
                TileBranch {
                    fraction: 0.6,
                    tree: stack(&[1, 2], 1),
                },
                TileBranch {
                    fraction: 0.4,
                    tree: TileTree::Split {
                        axis: SplitAxis::Column,
                        children: vec![
                            TileBranch {
                                fraction: 0.25,
                                tree: stack(&[3], 0),
                            },
                            TileBranch {
                                fraction: 0.75,
                                tree: stack(&[4], 0),
                            },
                        ],
                    },
                },
            ],
        };
        let before = tree.clone();
        let swatch = FormeSwatch::from_tree("Compare".into(), &tree, |t| {
            Some(FormeSwatchTile {
                id: t.id,
                title: t.title.clone(),
                passage: None,
                focused: t.id == TileId(2),
            })
        });
        assert_eq!(tree, before);
        assert_eq!(swatch.cells.len(), 3);
        assert_eq!(swatch.cells[0].bounds, [0.0, 0.0, 0.6, 1.0]);
        assert_eq!(swatch.cells[1].bounds, [0.6, 0.0, 1.0, 0.25]);
        assert_eq!(swatch.cells[2].bounds, [0.6, 0.25, 1.0, 1.0]);
        assert_eq!(swatch.cells[0].active, 1);
        assert_eq!(
            swatch.cells[0]
                .tabs
                .iter()
                .map(|t| t.id)
                .collect::<Vec<_>>(),
            vec![TileId(1), TileId(2)]
        );
    }
    #[test]
    fn missing_active_target_does_not_retarget_another_stacks_identity() {
        let swatch = FormeSwatch::from_tree("Compare".into(), &stack(&[1, 2, 3], 1), |t| {
            (t.id != TileId(2)).then(|| FormeSwatchTile {
                id: t.id,
                title: t.title.clone(),
                passage: None,
                focused: false,
            })
        });
        assert_eq!(
            swatch.cells[0]
                .tabs
                .iter()
                .map(|t| t.id)
                .collect::<Vec<_>>(),
            vec![TileId(1), TileId(3)]
        );
        assert_eq!(swatch.cells[0].active, 0);
    }
}
