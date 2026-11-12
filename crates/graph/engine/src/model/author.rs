// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Who asserted: the author a journal entry, a predicate declaration or a
//! classification origin records. Moved from the kernel's journal (engine
//! split P5 L3); the kernel re-exports it at `kernel::graph::Author`.

use rkyv::{Archive, Deserialize, Serialize};

/// The id the trusted UI's person records under when the host has no persona
/// to name: see [`Author::user`].
pub const USER_AUTHOR: &str = "user";

/// What kind of author made a change: a person, or a rule, script or engine
/// acting at some version (the ambiance design's moves between keeping levels
/// record the same four).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Archive,
    Serialize,
    Deserialize,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum AuthorKind {
    Person,
    Rule,
    Script,
    Engine,
}

/// Who or what made a journal entry, and through which application.
///
/// Every field is always written, since a positional codec cannot read back a
/// field written conditionally.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Archive,
    Serialize,
    Deserialize,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Author {
    pub kind: AuthorKind,
    /// A persona or subject id for a person; the rule's, script's or engine's
    /// name otherwise.
    pub id: String,
    /// The rule's, script's or engine's version. A person has none.
    pub version: Option<String>,
    /// The application the change came through. A resident fills this in
    /// from the admitted route rather than taking the client's word for it.
    pub via: Option<String>,
}

impl Author {
    /// Stable assertion attribution; version and recording application stay metadata.
    pub fn asserter_iri(&self) -> String {
        if self.kind == AuthorKind::Person
            && !self.id.chars().any(|c| c.is_whitespace() || c.is_control())
            && url::Url::parse(&self.id).is_ok()
        {
            return self.id.clone();
        }
        let kind = match self.kind {
            AuthorKind::Person => "person",
            AuthorKind::Rule => "rule",
            AuthorKind::Script => "script",
            AuthorKind::Engine => "engine",
        };
        let encoded: String = url::form_urlencoded::byte_serialize(self.id.as_bytes()).collect();
        format!("https://mere.computer/ns/agent#{kind}/{encoded}")
    }

    /// A person, by persona or subject id.
    pub fn person(id: impl Into<String>) -> Self {
        Self::new(AuthorKind::Person, id, None)
    }

    /// The person at the trusted UI, when the host has no persona to name.
    pub fn user() -> Self {
        Self::person(USER_AUTHOR)
    }

    /// A rule at a version.
    pub fn rule(id: impl Into<String>, version: impl Into<String>) -> Self {
        Self::new(AuthorKind::Rule, id, Some(version.into()))
    }

    /// A script at a version.
    pub fn script(id: impl Into<String>, version: impl Into<String>) -> Self {
        Self::new(AuthorKind::Script, id, Some(version.into()))
    }

    /// An engine at a version.
    pub fn engine(id: impl Into<String>, version: impl Into<String>) -> Self {
        Self::new(AuthorKind::Engine, id, Some(version.into()))
    }

    pub fn new(kind: AuthorKind, id: impl Into<String>, version: Option<String>) -> Self {
        Self {
            kind,
            id: id.into(),
            version,
            via: None,
        }
    }

    /// The same author, arriving through `application`.
    pub fn via(mut self, application: impl Into<String>) -> Self {
        self.via = Some(application.into());
        self
    }
}

impl std::fmt::Display for Author {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = match self.kind {
            AuthorKind::Person => "person",
            AuthorKind::Rule => "rule",
            AuthorKind::Script => "script",
            AuthorKind::Engine => "engine",
        };
        write!(f, "{kind} {}", self.id)?;
        if let Some(version) = &self.version {
            write!(f, " {version}")?;
        }
        if let Some(via) = &self.via {
            write!(f, " via {via}")?;
        }
        Ok(())
    }
}
