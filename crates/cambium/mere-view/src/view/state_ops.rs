/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::*;

impl MereViewState {
    pub fn apply(&mut self, event: MereViewEvent) {
        match event {
            MereViewEvent::Hover(key) => {
                // Hovering an archive node peeks its popover; leaving it, or
                // hovering anything else, closes a peek (never a pinned one).
                match key.as_deref().and_then(archive_of_node) {
                    Some(archive) => self.apply(MereViewEvent::ArchivePopover(
                        archive.to_string(),
                        DetailPopoverEvent::Hover(true),
                    )),
                    None => {
                        if let Some((open, _)) = self.archive_popover.clone() {
                            self.apply(MereViewEvent::ArchivePopover(
                                open,
                                DetailPopoverEvent::Hover(false),
                            ));
                        }
                    },
                }
                self.hovered = key;
            },
            MereViewEvent::Focus(key) => {
                if key.is_some() {
                    self.focus_return = None;
                }
                self.focused = key;
            },
            MereViewEvent::Expand(card) => {
                self.focus_return = match (self.expanded, card) {
                    (Some(closed), None) => Some(card_node_key(closed)),
                    _ => None,
                };
                self.expanded = card;
            },
            MereViewEvent::ArchivePopover(key, event) => {
                let mut popover = match self.archive_popover.take() {
                    Some((open, popover)) if open == key => popover,
                    // A pinned popover stays put while another node is hovered.
                    Some((open, popover))
                        if popover.mode == DetailPopoverMode::Pinned
                            && matches!(event, DetailPopoverEvent::Hover(_)) =>
                    {
                        self.archive_popover = Some((open, popover));
                        return;
                    },
                    _ => DetailPopoverState::default(),
                };
                popover.apply(event);
                if popover.return_focus {
                    popover.return_focus = false;
                    self.focus_return = Some(archive_node_key(&key));
                } else if popover.mode == DetailPopoverMode::Pinned {
                    self.focus_return = None;
                }
                self.archive_popover =
                    (popover.mode != DetailPopoverMode::Hidden).then_some((key, popover));
            },
            MereViewEvent::MergeDraft { key, seed, command } => {
                self.merge_drafts
                    .entry(key)
                    .or_insert_with(|| TextInput::new(seed))
                    .apply(command);
            },
            MereViewEvent::SelectArchive(key) => {
                if self.selected_archives.contains(&key) {
                    self.selected_archives.retain(|selected| selected != &key);
                } else {
                    self.selected_archives.push(key);
                }
            },
        }
    }

    /// Apply `event`, then lay the view out again for `model` in a tile of
    /// `width` by `height`: the one call a host makes per event, since
    /// opening or closing a tier card changes the area the graph is laid out
    /// in. Laying out again is free when nothing it reads changed.
    pub fn apply_and_lay_out(
        &mut self,
        event: MereViewEvent,
        model: &MereViewModel,
        width: u32,
        height: u32,
    ) {
        self.apply(event);
        self.lay_out(model, width, height);
    }

    /// Lay the model's graph out for a tile of `width` by `height`, unless its
    /// nodes, its relations, the layout and the size are the ones last laid
    /// out. Call it before rendering whenever any of them may have changed.
    pub fn lay_out(&mut self, model: &MereViewModel, width: u32, height: u32) {
        // A refresh removes vanished archive selections before a later Compose.
        self.selected_archives.retain(|key| {
            model
                .archive
                .as_ref()
                .is_some_and(|archive| archive.entries.iter().any(|entry| &entry.key == key))
        });
        // A card or popover whose session, shelf or archive is gone closes.
        if let Some(card) = self.expanded
            && !card_listed(model, card)
        {
            self.expanded = None;
        }
        if let Some((key, _)) = &self.archive_popover
            && !model
                .archive
                .as_ref()
                .is_some_and(|archive| archive.entries.iter().any(|entry| &entry.key == key))
        {
            self.archive_popover = None;
        }
        // A draft whose conflict is gone, or whose merge is, goes with it.
        self.merge_drafts.retain(|key, _| {
            model
                .merge
                .as_ref()
                .is_some_and(|merge| merge.conflicts.iter().any(|conflict| &conflict.key == key))
        });
        if model.hide_graph {
            // The host's editing surface owns the geometry in panel mode.
            return;
        }
        let size = catalog_area(model, self.expanded, width, height);
        let nodes: Vec<String> = model.graph.nodes.iter().map(|n| n.key.clone()).collect();
        let relations: Vec<(String, String)> = model
            .graph
            .relations
            .iter()
            .map(|r| (r.from.clone(), r.to.clone()))
            .collect();
        let laid = &self.laid;
        if laid.layout == model.layout
            && laid.size == size
            && laid.nodes == nodes
            && laid.relations == relations
        {
            return;
        }
        let positions = lay_out(&model.graph, &model.layout, size.0, size.1);
        self.laid = Laid {
            layout: model.layout.clone(),
            size,
            nodes,
            relations,
            positions,
        };
    }

    /// Where the last layout put the node with `key`, normalized to `0..=1`
    /// within the graph's own area (below the tier, when there is one).
    pub fn position(&self, key: &str) -> Option<(f32, f32)> {
        self.laid.positions.get(key).copied()
    }
}
