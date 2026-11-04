// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

/// A logical key, after the platform applied layout and modifiers.
///
/// Mirrors [`cambium::Key`]'s cases and adds [`Unidentified`](Key::Unidentified),
/// which Cambium's vocabulary has no room for because Cambium sits downstream
/// of this decision: by the time a key reaches a runner it has either resolved
/// to text or to a named key.
///
/// The host cannot assume that. Windows delivers injected text as `VK_PACKET`,
/// which surfaces as an unidentified key carrying text. On-screen keyboards,
/// keyboard remappers, and several assistive input tools all type that way, so
/// dropping the case would mean people using them cannot type at all.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Key {
    /// A key that produced text: the resolved character string.
    Character(String),
    /// A named, non-text key.
    Named(NamedKey),
    /// A key the platform could not name. It may still carry text; see
    /// [`KeyPress::injected_text`].
    Unidentified,
    /// A dead key: an accent awaiting composition. Never types on its own, even
    /// if the platform reported text alongside it. Kept distinct from
    /// [`Unidentified`](Key::Unidentified) so the injected-text path cannot
    /// swallow it; both winit and the DOM report this case.
    Dead,
}

/// The named keys host routing reads.
///
/// Deliberately the same set as [`cambium::NamedKey`], so lowering is total and
/// obvious. An event source that sees a named key outside this set lowers it to
/// [`Other`](NamedKey::Other) rather than inventing a case here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NamedKey {
    Backspace,
    Enter,
    Tab,
    Escape,
    Space,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    Delete,
    Home,
    End,
    F10,
    Alt,
    PageUp,
    PageDown,
    /// A named key this vocabulary does not special-case.
    Other,
}

/// Modifier state at the time of a press.
///
/// `meta` is the platform command key: Command on macOS, Super or Windows
/// elsewhere. The field names match [`cambium::Modifiers`] so the lowering is a
/// field copy rather than a mapping anyone has to check.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

/// A key press as the host routes it.
///
/// Constructible anywhere, which is the point: a keyboard-order receipt that
/// cannot run in `cargo test` is a receipt nobody collects, and a host whose
/// keyboard path names its event source cannot gain a second one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyPress {
    /// The logical key, after the layout and modifiers the platform applied.
    pub key: Key,
    /// The text this press produces, when the platform reports any.
    ///
    /// Carried separately because `key` alone is not always enough; see
    /// [`Key::Unidentified`].
    pub text: Option<String>,
    /// Modifiers held at the time of the press.
    pub modifiers: Modifiers,
    /// Whether this is an auto-repeat rather than a fresh press.
    pub repeat: bool,
}

/// Which way an arrow key pointed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// A laid-out rect as this module reasons about it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Box2 {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Modifiers {
    /// No modifiers held.
    pub const NONE: Self = Self {
        shift: false,
        ctrl: false,
        alt: false,
        meta: false,
    };

    /// Whether this press is a command chord rather than typing.
    ///
    /// Control or the platform command key. Not Alt: on several layouts AltGr
    /// is how ordinary characters are typed, and treating it as a chord would
    /// stop those layouts from producing text.
    pub fn is_command_chord(self) -> bool {
        self.ctrl || self.meta
    }
}

impl KeyPress {
    /// A press with no modifiers held.
    pub fn new(key: Key) -> Self {
        let text = match &key {
            Key::Character(c) => Some(c.clone()),
            _ => None,
        };
        Self {
            key,
            text,
            modifiers: Modifiers::NONE,
            repeat: false,
        }
    }

    /// A named-key press (Tab, Enter, ArrowLeft, and so on).
    pub fn named(named: NamedKey) -> Self {
        Self::new(Key::Named(named))
    }

    /// A press of a key that produced this text.
    pub fn character(text: impl Into<String>) -> Self {
        Self::new(Key::Character(text.into()))
    }

    /// Hold these modifiers for the press.
    #[must_use]
    pub fn with_modifiers(mut self, modifiers: Modifiers) -> Self {
        self.modifiers = modifiers;
        self
    }

    /// Mark this press as an auto-repeat.
    #[must_use]
    pub fn repeated(mut self) -> Self {
        self.repeat = true;
        self
    }

    /// The character this press should insert when the platform could not name
    /// the key but did report text.
    ///
    /// `None` for a named key, a control character, or a chord: a shortcut must
    /// not become typed text.
    pub fn injected_text(&self) -> Option<&str> {
        if !matches!(self.key, Key::Unidentified) {
            return None;
        }
        if self.modifiers.is_command_chord() {
            return None;
        }
        let text = self.text.as_deref()?;
        if text.is_empty() || text.chars().any(char::is_control) {
            return None;
        }
        Some(text)
    }

    /// Lower into the key event a runner consumes, or `None` for a press the
    /// tree should never see.
    ///
    /// The unidentified-with-text case is the assistive-input path and is the
    /// reason this is not a plain `From`: whether such a press becomes text
    /// depends on the modifiers held, so the decision belongs with the press
    /// rather than with the key.
    ///
    /// A `None` return is the host's "dropped: no runner key and no text" case,
    /// which is worth tracing where it happens.
    pub fn to_runner_key(&self) -> Option<cambium::KeyEvent> {
        let mods = self.modifiers.into();
        let mapped = match &self.key {
            Key::Character(s) => cambium::Key::Character(s.clone()),
            Key::Named(named) => cambium::Key::Named((*named).into()),
            Key::Unidentified => cambium::Key::Character(self.injected_text()?.to_string()),
            Key::Dead => return None,
        };
        Some(cambium::KeyEvent::with_mods(mapped, mods))
    }

    /// The caret movement this press means by default, or `None` for a key the
    /// host has no caret default for.
    ///
    /// `word` selects word-granularity movement, which callers derive from the
    /// modifiers their platform uses for it.
    pub fn caret_movement(&self, word: bool) -> Option<VisualMovement> {
        let Key::Named(named) = &self.key else {
            return None;
        };
        Some(match named {
            NamedKey::ArrowLeft if word => VisualMovement::PreviousWord,
            NamedKey::ArrowLeft => VisualMovement::PreviousCluster,
            NamedKey::ArrowRight if word => VisualMovement::NextWord,
            NamedKey::ArrowRight => VisualMovement::NextCluster,
            NamedKey::ArrowUp => VisualMovement::PreviousLine,
            NamedKey::ArrowDown => VisualMovement::NextLine,
            NamedKey::Home => VisualMovement::LineStart,
            NamedKey::End => VisualMovement::LineEnd,
            _ => return None,
        })
    }

    /// The direction this press points, if it is an arrow key.
    pub fn direction(&self) -> Option<Direction> {
        match &self.key {
            Key::Named(named) => Direction::from_named(*named),
            _ => None,
        }
    }
}

impl Direction {
    /// The arrow key this direction comes from, if any.
    pub fn from_named(named: NamedKey) -> Option<Self> {
        Some(match named {
            NamedKey::ArrowUp => Self::Up,
            NamedKey::ArrowDown => Self::Down,
            NamedKey::ArrowLeft => Self::Left,
            NamedKey::ArrowRight => Self::Right,
            _ => return None,
        })
    }
}

impl Box2 {
    /// The rect's centre point.
    pub fn centre(&self) -> (f32, f32) {
        (self.x + self.w / 2.0, self.y + self.h / 2.0)
    }
}

impl From<NamedKey> for cambium::NamedKey {
    fn from(named: NamedKey) -> Self {
        match named {
            NamedKey::Backspace => Self::Backspace,
            NamedKey::Enter => Self::Enter,
            NamedKey::Tab => Self::Tab,
            NamedKey::Escape => Self::Escape,
            NamedKey::Space => Self::Space,
            NamedKey::ArrowLeft => Self::ArrowLeft,
            NamedKey::ArrowRight => Self::ArrowRight,
            NamedKey::ArrowUp => Self::ArrowUp,
            NamedKey::ArrowDown => Self::ArrowDown,
            NamedKey::Delete => Self::Delete,
            NamedKey::Home => Self::Home,
            NamedKey::End => Self::End,
            NamedKey::F10 => Self::F10,
            NamedKey::Alt => Self::Alt,
            NamedKey::PageUp => Self::PageUp,
            NamedKey::PageDown => Self::PageDown,
            NamedKey::Other => Self::Other,
        }
    }
}

impl From<Modifiers> for cambium::Modifiers {
    fn from(mods: Modifiers) -> Self {
        Self {
            shift: mods.shift,
            ctrl: mods.ctrl,
            alt: mods.alt,
            meta: mods.meta,
        }
    }
}
