/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A palette's state while it is open (SE48): what was typed, the surface it
//! opened over, the highlighted row, whether the search wants focus, and
//! whether it was expanded to every command. Where it sits and how it is
//! drawn are the host's.

use crate::{Command, CommandChoices, CommandSet};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MenuSession {
    pub query: String,
    /// The surface it opened over (`"node"`), or `None` where only the kept
    /// and recent commands show (SE29).
    pub context: Option<String>,
    /// The highlighted row, or `None` before the person moves.
    pub selected: Option<usize>,
    /// The search field should take focus when next drawn.
    pub focus_search: bool,
    /// Expanded to every command ("All commands…"), until it closes.
    pub all: bool,
}

impl MenuSession {
    /// A fresh session over `context`, its search asking for focus.
    pub fn open(context: Option<&str>) -> Self {
        Self {
            context: context.map(str::to_string),
            focus_search: true,
            ..Self::default()
        }
    }

    /// The person typed: the rows change, so the highlight starts over.
    pub fn set_query(&mut self, query: &str) {
        if self.query != query {
            self.query = query.to_string();
            self.selected = None;
        }
    }

    /// Show every command; the highlight starts over.
    pub fn expand_all(&mut self) {
        self.all = true;
        self.selected = None;
    }

    /// Move the highlight by `delta` over `count` rows, wrapping. The first
    /// move down lands on the first row, the first move up on the last.
    pub fn step(&mut self, delta: isize, count: usize) {
        if count == 0 {
            self.selected = None;
            return;
        }
        let count = count as isize;
        self.selected = Some(match self.selected {
            Some(current) => (current as isize + delta).rem_euclid(count) as usize,
            None if delta >= 0 => 0,
            None => (count - 1) as usize,
        });
    }

    /// The rows to show: every command when expanded with no query, else the
    /// set's menu for this context and query.
    pub fn rows<'a>(&self, set: &'a CommandSet, choices: &CommandChoices) -> Vec<&'a Command> {
        if self.all && self.query.trim().is_empty() {
            return set.commands().iter().collect();
        }
        set.menu(choices, self.context.as_deref(), &self.query)
    }

    /// The command Enter runs: the highlighted row, else the first that is
    /// not disabled.
    pub fn chosen<'a>(&self, rows: &[&'a Command]) -> Option<&'a Command> {
        match self.selected {
            Some(index) => rows.get(index).copied(),
            None => rows
                .iter()
                .copied()
                .find(|command| command.disabled_reason.is_none()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set() -> CommandSet {
        let mut set = CommandSet::new().with_defaults(["zoom-in"]);
        for (id, category) in [("open", "node"), ("zoom-in", "canvas"), ("add", "canvas")] {
            set.register(Command::new(id, id, category));
        }
        set
    }

    fn ids<'a>(rows: &[&'a Command]) -> Vec<&'a str> {
        rows.iter().map(|command| command.id.as_str()).collect()
    }

    #[test]
    fn a_fresh_session_asks_for_focus_and_shows_its_context_first() {
        let session = MenuSession::open(Some("node"));
        assert!(session.focus_search);
        assert_eq!(session.selected, None);
        assert_eq!(
            ids(&session.rows(&set(), &CommandChoices::default())),
            ["open", "zoom-in"]
        );
    }

    #[test]
    fn the_highlight_wraps_and_starts_over_when_the_rows_change() {
        let mut session = MenuSession::open(None);
        session.step(-1, 3);
        assert_eq!(
            session.selected,
            Some(2),
            "first move up lands on the last row"
        );
        session.step(1, 3);
        assert_eq!(session.selected, Some(0), "wraps");
        session.step(1, 3);
        session.set_query("add");
        assert_eq!(session.selected, None);
        session.step(1, 0);
        assert_eq!(session.selected, None, "no rows, no highlight");
    }

    #[test]
    fn expanded_shows_every_command_until_a_query_searches() {
        let set = set();
        let choices = CommandChoices::default();
        let mut session = MenuSession::open(None);
        assert_eq!(ids(&session.rows(&set, &choices)), ["zoom-in"]);
        session.expand_all();
        assert_eq!(
            ids(&session.rows(&set, &choices)),
            ["open", "zoom-in", "add"]
        );
        session.set_query("ad");
        assert_eq!(ids(&session.rows(&set, &choices)), ["add"]);
    }

    #[test]
    fn enter_runs_the_highlight_else_the_first_available() {
        let mut set = set();
        set.set_disabled("open", Some("nothing selected".into()));
        let mut session = MenuSession::open(Some("node"));
        let rows = session.rows(&set, &CommandChoices::default());
        assert_eq!(
            session.chosen(&rows).map(|c| c.id.as_str()),
            Some("zoom-in")
        );
        session.step(1, rows.len());
        assert_eq!(session.chosen(&rows).map(|c| c.id.as_str()), Some("open"));
    }
}
