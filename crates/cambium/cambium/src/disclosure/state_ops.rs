/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::*;

impl DisclosureState {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            expanded: false,
            disabled: false,
        }
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn toggle(&mut self) {
        if !self.disabled {
            self.expanded = !self.expanded;
        }
    }
}

impl AccordionConfig {
    pub fn with_heading_level(mut self, level: u8) -> Self {
        self.heading_level = level.clamp(1, 6);
        self
    }

    pub fn with_panel_regions(mut self, panel_regions: bool) -> Self {
        self.panel_regions = panel_regions;
        self
    }
}

impl<Id> AccordionState<Id> {
    pub fn new() -> Self {
        Self {
            id: "cambium-accordion".into(),
            label: "Sections".into(),
            expanded: Vec::new(),
            mode: AccordionMode::Multiple,
            collapsible: true,
        }
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn single(mut self, collapsible: bool) -> Self {
        self.mode = AccordionMode::Single;
        self.collapsible = collapsible;
        self.expanded.truncate(1);
        self
    }

    pub fn with_expanded(mut self, expanded: impl IntoIterator<Item = Id>) -> Self
    where
        Id: Eq,
    {
        self.expanded.clear();
        for id in expanded {
            if !self.expanded.contains(&id) {
                self.expanded.push(id);
            }
            if self.mode == AccordionMode::Single {
                break;
            }
        }
        self
    }
}

impl<Id> AccordionState<Id>
where
    Id: Clone + Eq,
{
    pub fn is_expanded(&self, id: &Id) -> bool {
        self.expanded.contains(id)
    }

    pub fn toggle(&mut self, id: Id) {
        if let Some(position) = self.expanded.iter().position(|open| open == &id) {
            if self.collapsible {
                self.expanded.remove(position);
            }
            return;
        }
        if self.mode == AccordionMode::Single {
            self.expanded.clear();
        }
        self.expanded.push(id);
    }
}

impl<Id> AccordionItem<Id> {
    pub fn new(id: Id, label: impl Into<String>, body: impl Into<String>) -> Self
    where
        Id: ToString,
    {
        let dom_id = id.to_string();
        Self {
            id,
            dom_id,
            label: label.into(),
            body: body.into(),
        }
    }

    pub fn with_dom_id(mut self, dom_id: impl Into<String>) -> Self {
        self.dom_id = dom_id.into();
        self
    }
}

impl<Id> TreeState<Id>
where
    Id: Clone + Eq,
{
    pub fn is_expanded(&self, id: &Id) -> bool {
        self.expanded.contains(id)
    }

    pub fn toggle(&mut self, id: Id) {
        if let Some(position) = self.expanded.iter().position(|open| open == &id) {
            self.expanded.remove(position);
        } else {
            self.expanded.push(id);
        }
    }

    pub(super) fn move_active(&mut self, id: Id) {
        self.active = Some(id.clone());
        if self.selection_mode == TreeSelectionMode::FollowsFocus {
            self.selected = Some(id);
        }
        self.focus_active = true;
    }

    pub(super) fn activate(&mut self, id: Id) {
        self.active = Some(id.clone());
        self.selected = Some(id);
        self.focus_active = true;
    }

    pub(super) fn reconcile(&mut self, items: &[TreeItem<Id>]) {
        let mut all = Vec::new();
        collect_all(items, &mut all);
        for (index, id) in all.iter().enumerate() {
            assert!(
                !all[..index].contains(id),
                "duplicate identity in TreeItem hierarchy"
            );
        }
        self.expanded
            .retain(|expanded| all.iter().any(|id| id == expanded));
        let visible = visible_nodes(items, &self.expanded);
        if self
            .active
            .as_ref()
            .is_none_or(|active| !visible.iter().any(|node| &node.id == active))
        {
            self.active = self
                .selected
                .as_ref()
                .filter(|selected| visible.iter().any(|node| &node.id == *selected))
                .cloned()
                .or_else(|| visible.first().map(|node| node.id.clone()));
            self.focus_active = false;
        }
        if self
            .selected
            .as_ref()
            .is_none_or(|selected| !all.iter().any(|id| id == selected))
        {
            self.selected = self.active.clone();
        }
    }
}

impl<Id> TreeState<Id> {
    pub fn new() -> Self {
        Self {
            id: "cambium-tree".into(),
            label: "Tree".into(),
            expanded: Vec::new(),
            selection_mode: TreeSelectionMode::FollowsFocus,
            active: None,
            selected: None,
            focus_active: false,
        }
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn with_expanded(mut self, expanded: impl IntoIterator<Item = Id>) -> Self
    where
        Id: Eq,
    {
        self.expanded.clear();
        for id in expanded {
            if !self.expanded.contains(&id) {
                self.expanded.push(id);
            }
        }
        self
    }

    pub fn with_selection_mode(mut self, selection_mode: TreeSelectionMode) -> Self {
        self.selection_mode = selection_mode;
        self
    }

    pub fn with_selected(mut self, selected: Id) -> Self
    where
        Id: Clone,
    {
        self.active = Some(selected.clone());
        self.selected = Some(selected);
        self
    }

    pub fn active(&self) -> Option<&Id> {
        self.active.as_ref()
    }

    pub fn selected(&self) -> Option<&Id> {
        self.selected.as_ref()
    }
}

impl<Id> TreeItem<Id> {
    pub fn new(id: Id, label: impl Into<String>) -> Self
    where
        Id: ToString,
    {
        let dom_id = id.to_string();
        Self {
            id,
            dom_id,
            label: label.into(),
            children: Vec::new(),
        }
    }

    pub fn with_dom_id(mut self, dom_id: impl Into<String>) -> Self {
        self.dom_id = dom_id.into();
        self
    }

    pub fn with_children(mut self, children: impl IntoIterator<Item = TreeItem<Id>>) -> Self {
        self.children = children.into_iter().collect();
        self
    }
}
