// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Host-independent theme editing. A draft never changes the registry or the
//! host's active appearance until explicitly committed. Hosts own disk writes,
//! preview rendering and applying a theme to their own surfaces.

use crate::Theme;
use crate::library::{validate_definition, validate_user_theme};
use crate::theme::registry::{Harmony, Mode, ThemeRegistry, ThemeSource, ThemeTokenSet};
use crate::theme::seed::derive_from_def_for_mode;
use tinct::Seeds;

/// Authoring actions deliberately cannot change identity or provenance.
#[derive(Clone, Debug)]
pub enum Edit {
    Name(String),
    Seeds(Seeds),
    Harmony(Harmony),
    HighContrast(bool),
    /// Set the authored canonical default as one undoable edit. Preview mode
    /// remains local to the host; custom modes cannot be encoded by these flags.
    DefaultMode(Mode),
    /// Empty rules remove the override and restore derived appearance.
    ModeSheet {
        mode: Mode,
        rules: Vec<String>,
    },
}

/// Preview evidence. An authored sheet must be rendered by the host; it is
/// never reported as if the canonical token derivation incorporated its CSS.
#[derive(Clone, Debug, PartialEq)]
pub enum Preview {
    Tokens(ThemeTokenSet),
    Stylesheet(Vec<String>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorkshopError {
    UnknownTheme,
    BuiltInRequiresFork,
    InvalidIdentity,
    IdentityInUse,
    SourceChanged,
    CustomModeNeedsCalculator,
    InvalidTheme(String),
}

impl std::fmt::Display for WorkshopError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownTheme => f.write_str("The theme was not found."),
            Self::BuiltInRequiresFork => f.write_str("Fork a built-in theme before editing it."),
            Self::InvalidIdentity => f.write_str("A theme needs a nonempty id and name."),
            Self::IdentityInUse => f.write_str("Another theme already uses this id."),
            Self::SourceChanged => {
                f.write_str("The saved theme changed or was removed while editing.")
            },
            Self::CustomModeNeedsCalculator => {
                f.write_str("This custom mode needs a calculator or stylesheet.")
            },
            Self::InvalidTheme(reason) => write!(f, "The theme could not be saved: {reason}"),
        }
    }
}

impl std::error::Error for WorkshopError {}

/// A single authored theme transaction, with a save point and edit history.
#[derive(Clone, Debug)]
pub struct ThemeDraft {
    draft: Theme,
    baseline: Theme,
    /// None means this is a new fork, not yet registered.
    registered: Option<Theme>,
    undo: Vec<Theme>,
    redo: Vec<Theme>,
}

impl ThemeDraft {
    pub fn open(registry: &ThemeRegistry, id: &str) -> Result<Self, WorkshopError> {
        let theme = registry.theme_def(id).ok_or(WorkshopError::UnknownTheme)?;
        if theme.source != ThemeSource::User {
            return Err(WorkshopError::BuiltInRequiresFork);
        }
        Ok(Self::new(theme.clone(), Some(theme.clone())))
    }

    /// Fork without inserting anything into the registry. Dropping or
    /// discarding this draft therefore leaves the source untouched.
    pub fn fork(
        registry: &ThemeRegistry,
        source_id: &str,
        id: &str,
        name: &str,
    ) -> Result<Self, WorkshopError> {
        let theme = registry
            .theme_def(source_id)
            .ok_or(WorkshopError::UnknownTheme)?
            .clone();
        Self::fork_theme(registry, &theme, id, name)
    }

    /// Create a new, unregistered draft from a user definition. A colliding
    /// built-in identity or built-in provenance must be forked explicitly.
    /// Commit still checks the destination registry for any user-ID collision.
    pub fn from_theme(mut theme: Theme) -> Result<Self, WorkshopError> {
        theme.id = theme.id.trim().to_ascii_lowercase();
        if theme.id.is_empty() || theme.name.trim().is_empty() {
            return Err(WorkshopError::InvalidIdentity);
        }
        if theme.source != ThemeSource::User {
            return Err(WorkshopError::BuiltInRequiresFork);
        }
        if ThemeRegistry::default().theme_def(&theme.id).is_some() {
            return Err(WorkshopError::IdentityInUse);
        }
        validate_user_theme(&theme).map_err(WorkshopError::InvalidTheme)?;
        Ok(Self::new(theme, None))
    }

    /// Copy any validated definition into a fresh user identity, preserving
    /// seeds, harmony, contrast and mode sheets without inserting the source.
    pub fn fork_theme(
        registry: &ThemeRegistry,
        source: &Theme,
        id: &str,
        name: &str,
    ) -> Result<Self, WorkshopError> {
        let id = id.trim().to_ascii_lowercase();
        if id.is_empty() || name.trim().is_empty() {
            return Err(WorkshopError::InvalidIdentity);
        }
        if registry.theme_def(&id).is_some() {
            return Err(WorkshopError::IdentityInUse);
        }
        validate_definition(source).map_err(WorkshopError::InvalidTheme)?;
        let mut theme = source.clone();
        theme.id = id;
        theme.name = name.to_owned();
        theme.source = ThemeSource::User;
        Self::from_theme(theme)
    }

    fn new(theme: Theme, registered: Option<Theme>) -> Self {
        Self {
            baseline: theme.clone(),
            draft: theme,
            registered,
            undo: vec![],
            redo: vec![],
        }
    }

    pub fn theme(&self) -> &Theme {
        &self.draft
    }

    /// A new fork needs saving even before its first edit.
    pub fn is_dirty(&self) -> bool {
        self.registered.is_none() || self.draft != self.baseline
    }

    pub fn edit(&mut self, edit: Edit) -> bool {
        let mut next = self.draft.clone();
        match edit {
            Edit::Name(name) => next.name = name,
            Edit::Seeds(seeds) => next.seeds = seeds,
            Edit::Harmony(harmony) => next.harmony = harmony,
            Edit::HighContrast(high_contrast) => next.high_contrast = high_contrast,
            Edit::DefaultMode(mode) => {
                if matches!(mode, Mode::Custom(_)) {
                    return false;
                }
                next.seeds.dark = mode.dark();
                next.high_contrast = mode.high_contrast();
            },
            Edit::ModeSheet { mode, rules } => {
                if rules.is_empty() {
                    next.mode_sheets.remove(&mode.as_key());
                } else {
                    next.mode_sheets.insert(mode.as_key(), rules);
                }
            },
        }
        if next == self.draft {
            return false;
        }
        self.undo.push(std::mem::replace(&mut self.draft, next));
        self.redo.clear();
        true
    }

    pub fn undo(&mut self) -> bool {
        let Some(previous) = self.undo.pop() else {
            return false;
        };
        self.redo.push(std::mem::replace(&mut self.draft, previous));
        true
    }

    pub fn redo(&mut self) -> bool {
        let Some(next) = self.redo.pop() else {
            return false;
        };
        self.undo.push(std::mem::replace(&mut self.draft, next));
        true
    }

    /// Restore the last save point (or the initial fork). Drop a new fork
    /// entirely to abandon creating it.
    pub fn discard(&mut self) {
        self.draft = self.baseline.clone();
        self.undo.clear();
        self.redo.clear();
    }

    pub fn preview(&self, mode: &Mode) -> Result<Preview, WorkshopError> {
        if let Some(rules) = self.draft.mode_sheet(mode) {
            return Ok(Preview::Stylesheet(rules.clone()));
        }
        if matches!(mode, Mode::Custom(_)) {
            return Err(WorkshopError::CustomModeNeedsCalculator);
        }
        Ok(Preview::Tokens(derive_from_def_for_mode(&self.draft, mode)))
    }

    /// Commit to the in-memory registry without activating or persisting it.
    /// Compare against the opening/save-point definition to reject lost edits.
    pub fn commit(&mut self, registry: &mut ThemeRegistry) -> Result<(), WorkshopError> {
        if self.draft.name.trim().is_empty() {
            return Err(WorkshopError::InvalidIdentity);
        }
        let current = registry.theme_def(&self.draft.id);
        match &self.registered {
            Some(expected) if current != Some(expected) => {
                return Err(WorkshopError::SourceChanged);
            },
            None if current.is_some() => return Err(WorkshopError::IdentityInUse),
            _ => {},
        }
        validate_user_theme(&self.draft).map_err(WorkshopError::InvalidTheme)?;
        registry
            .add_user_theme(self.draft.clone())
            .map_err(WorkshopError::InvalidTheme)?;
        self.baseline = self.draft.clone();
        self.registered = Some(self.draft.clone());
        self.undo.clear();
        self.redo.clear();
        Ok(())
    }
}
