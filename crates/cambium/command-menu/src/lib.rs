/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! One home for commands (Scenograph editor plan, track C2, SE45 to SE48).
//!
//! A host registers its commands in a [`CommandSet`]; the person's choices
//! ride beside them as [`CommandChoices`]: commands added to and removed from
//! the defaults, and the recently used ones. [`CommandSet::menu`] composes
//! what a surface shows, in one order every surface shares: the commands for
//! where it was opened lead (Turnstone's rule, harvested at SE32), then the
//! kept ones, then the recent ones. A query searches every command instead.
//! [`MenuSession`] is a palette's state while it is open, and [`catalogue`]
//! holds the ids hosts share. Storage and drawing
//! are the host's; Cambium draws a command as a `CommandItem`.

pub mod catalogue;
mod session;

use serde::{Deserialize, Serialize};

pub use session::MenuSession;

/// A command a host offers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Command {
    pub id: String,
    pub label: String,
    /// The surface it belongs to when the menu opens over one (`"node"`,
    /// `"canvas"`), or a family for search (`"projection"`).
    pub category: String,
    pub shortcut: Option<String>,
    /// Why it cannot run right now; shown rather than hidden.
    pub disabled_reason: Option<String>,
}

impl Command {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        category: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            category: category.into(),
            shortcut: None,
            disabled_reason: None,
        }
    }

    pub fn with_shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    fn matches(&self, query: &str) -> bool {
        [&self.label, &self.category, &self.id]
            .iter()
            .any(|text| text.to_lowercase().contains(query))
    }
}

/// The person's choices: commands added to the defaults, defaults removed,
/// and the recently used commands, most recent first. Ids are the host's.
/// Hosts store it as a view, not graph truth (SE31); each list defaults to
/// empty, so what pandect's `CommandMenuView` or Turnstone's `CommandMenuV1`
/// wrote reads back unchanged.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandChoices {
    #[serde(default)]
    pub added: Vec<String>,
    #[serde(default)]
    pub removed: Vec<String>,
    #[serde(default)]
    pub recent: Vec<String>,
}

/// The commands a host registers and the defaults it starts a person with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandSet {
    commands: Vec<Command>,
    defaults: Vec<String>,
    recent_cap: usize,
}

impl Default for CommandSet {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandSet {
    /// An empty set keeping the eight most recent commands.
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            defaults: Vec::new(),
            recent_cap: 8,
        }
    }

    /// Register `command`, replacing one with the same id.
    pub fn register(&mut self, command: Command) {
        match self
            .commands
            .iter_mut()
            .find(|known| known.id == command.id)
        {
            Some(known) => *known = command,
            None => self.commands.push(command),
        }
    }

    /// The commands a person starts with, in order.
    pub fn with_defaults<I, S>(mut self, ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.defaults = ids.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_recent_cap(mut self, cap: usize) -> Self {
        self.recent_cap = cap;
        self
    }

    /// Mark `id` unavailable for `reason`, or available again with `None`.
    pub fn set_disabled(&mut self, id: &str, reason: Option<String>) {
        if let Some(command) = self.commands.iter_mut().find(|command| command.id == id) {
            command.disabled_reason = reason;
        }
    }

    pub fn get(&self, id: &str) -> Option<&Command> {
        self.commands.iter().find(|command| command.id == id)
    }

    pub fn commands(&self) -> &[Command] {
        &self.commands
    }

    /// The commands the person keeps: the defaults less those removed, then
    /// those added, each once.
    pub fn kept(&self, choices: &CommandChoices) -> Vec<&Command> {
        let mut ids: Vec<&String> = self
            .defaults
            .iter()
            .filter(|id| !choices.removed.contains(id))
            .collect();
        ids.extend(choices.added.iter());
        let mut seen = Vec::new();
        ids.into_iter()
            .filter(|id| {
                let fresh = !seen.contains(id);
                seen.push(*id);
                fresh
            })
            .filter_map(|id| self.get(id))
            .collect()
    }

    /// What a surface opened over `context` shows. Empty `query`: that
    /// surface's commands, then the kept ones, then the recent ones, each
    /// command once. A query searches every command by label, category and
    /// id, keeping that order for those it finds.
    pub fn menu(
        &self,
        choices: &CommandChoices,
        context: Option<&str>,
        query: &str,
    ) -> Vec<&Command> {
        let query = query.trim().to_lowercase();
        let contextual = self
            .commands
            .iter()
            .filter(|command| context.is_some_and(|context| command.category == context));
        let recent = choices.recent.iter().filter_map(|id| self.get(id));
        let ordered: Vec<&Command> = if query.is_empty() {
            contextual.chain(self.kept(choices)).chain(recent).collect()
        } else {
            contextual
                .chain(self.kept(choices))
                .chain(recent)
                .chain(self.commands.iter())
                .filter(|command| command.matches(&query))
                .collect()
        };
        let mut seen: Vec<&str> = Vec::new();
        ordered
            .into_iter()
            .filter(|command| {
                let fresh = !seen.contains(&command.id.as_str());
                seen.push(&command.id);
                fresh
            })
            .collect()
    }

    /// Record that `id` ran: it moves to the front of the recent ones.
    pub fn record_use(&self, choices: &mut CommandChoices, id: &str) {
        if self.get(id).is_none() {
            return;
        }
        choices.recent.retain(|recent| recent != id);
        choices.recent.insert(0, id.to_string());
        choices.recent.truncate(self.recent_cap);
    }

    /// Keep `id` among the person's commands.
    pub fn add(&self, choices: &mut CommandChoices, id: &str) {
        if self.get(id).is_none() {
            return;
        }
        choices.removed.retain(|removed| removed != id);
        if !self.defaults.iter().any(|default| default == id)
            && !choices.added.iter().any(|added| added == id)
        {
            choices.added.push(id.to_string());
        }
    }

    /// Stop keeping `id`: an added command is dropped, a default is removed.
    pub fn remove(&self, choices: &mut CommandChoices, id: &str) {
        choices.added.retain(|added| added != id);
        if self.defaults.iter().any(|default| default == id)
            && !choices.removed.iter().any(|removed| removed == id)
        {
            choices.removed.push(id.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set() -> CommandSet {
        let mut set = CommandSet::new()
            .with_defaults(["zoom-in", "zoom-out", "save-projection"])
            .with_recent_cap(3);
        for (id, label, category) in [
            ("open-detail", "Edit selected object", "node"),
            ("open-as-node", "Open as new node", "node"),
            ("zoom-in", "Zoom in", "canvas"),
            ("zoom-out", "Zoom out", "canvas"),
            ("save-projection", "Save projection", "projection"),
            ("add-address", "Add address", "canvas"),
        ] {
            set.register(Command::new(id, label, category));
        }
        set
    }

    fn ids<'a>(items: &[&'a Command]) -> Vec<&'a str> {
        items.iter().map(|item| item.id.as_str()).collect()
    }

    #[test]
    fn the_surfaces_commands_lead_then_kept_then_recent() {
        let set = set();
        let mut choices = CommandChoices::default();
        set.record_use(&mut choices, "add-address");
        set.record_use(&mut choices, "zoom-in");
        assert_eq!(
            ids(&set.menu(&choices, Some("node"), "")),
            [
                "open-detail",
                "open-as-node",
                "zoom-in",
                "zoom-out",
                "save-projection",
                "add-address"
            ],
            "each command once: zoom-in is kept, so recency does not repeat it"
        );
        assert_eq!(
            ids(&set.menu(&choices, None, "")),
            ["zoom-in", "zoom-out", "save-projection", "add-address"]
        );
    }

    #[test]
    fn a_query_searches_every_command() {
        let set = set();
        let choices = CommandChoices::default();
        assert_eq!(
            ids(&set.menu(&choices, None, "ADD")),
            ["add-address"],
            "not kept, still found"
        );
        assert_eq!(
            ids(&set.menu(&choices, Some("node"), "o")),
            [
                "open-detail",
                "open-as-node",
                "zoom-in",
                "zoom-out",
                "save-projection"
            ]
        );
        assert_eq!(
            ids(&set.menu(&choices, None, "node")),
            ["open-detail", "open-as-node"],
            "by category"
        );
        assert!(set.menu(&choices, None, "nothing like it").is_empty());
    }

    #[test]
    fn adds_and_removes_change_what_is_kept() {
        let set = set();
        let mut choices = CommandChoices::default();
        set.remove(&mut choices, "zoom-out");
        set.add(&mut choices, "add-address");
        set.add(&mut choices, "add-address");
        set.add(&mut choices, "no-such-command");
        assert_eq!(
            ids(&set.menu(&choices, None, "")),
            ["zoom-in", "save-projection", "add-address"]
        );
        set.remove(&mut choices, "add-address");
        set.add(&mut choices, "zoom-out");
        assert_eq!(
            choices,
            CommandChoices::default(),
            "undoing both leaves nothing recorded"
        );
    }

    #[test]
    fn recent_ones_are_capped_and_most_recent_first() {
        let set = set();
        let mut choices = CommandChoices::default();
        for id in [
            "open-detail",
            "add-address",
            "open-as-node",
            "open-detail",
            "zoom-out",
        ] {
            set.record_use(&mut choices, id);
        }
        set.record_use(&mut choices, "unregistered");
        assert_eq!(choices.recent, ["zoom-out", "open-detail", "open-as-node"]);
    }

    #[test]
    fn a_disabled_command_shows_its_reason() {
        let mut set = set();
        set.set_disabled("save-projection", Some("nothing to save".into()));
        let menu = set.menu(&CommandChoices::default(), None, "save");
        assert!(menu[0].disabled_reason.is_some());
        assert_eq!(menu[0].disabled_reason.as_deref(), Some("nothing to save"));
    }

    #[test]
    fn choices_read_what_the_older_stores_wrote() {
        // pandect's `CommandMenuView` and Turnstone's `CommandMenuV1` wrote
        // these field names, each list optional.
        let full: CommandChoices =
            serde_json::from_str(r#"{"added":["a"],"removed":["b"],"recent":["c","a"]}"#)
                .expect("reads");
        assert_eq!(full.added, ["a"]);
        assert_eq!(full.removed, ["b"]);
        assert_eq!(full.recent, ["c", "a"]);
        let sparse: CommandChoices = serde_json::from_str(r#"{"recent":["c"]}"#).expect("reads");
        assert_eq!(sparse.recent, ["c"]);
        assert!(sparse.added.is_empty() && sparse.removed.is_empty());
        let written = serde_json::to_string(&full).expect("writes");
        assert_eq!(
            written,
            r#"{"added":["a"],"removed":["b"],"recent":["c","a"]}"#
        );
    }
}
