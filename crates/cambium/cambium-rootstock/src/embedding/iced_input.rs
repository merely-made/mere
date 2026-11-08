// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
use super::{OwnerInput, ScrollUnits};
use backstep::{ClipboardError, ClipboardEvent, ClipboardRequest};
use cambium::ClipboardCommand;
use document_session_api::{
    SessionButtonState, SessionIme, SessionInput, SessionKey, SessionModifiers,
    SessionPointerButton,
};
use iced_wgpu::core::{Event, clipboard, input_method, keyboard, mouse};
#[derive(Default)]
pub struct Input {
    cursor: Option<[f32; 2]>,
    modifiers: SessionModifiers,
    /// Clipboard requests this input made that iced has not answered yet.
    reads: usize,
    writes: usize,
}
fn clipboard_error(error: &clipboard::Error) -> ClipboardError {
    use clipboard::Error as E;
    match error {
        E::ContentNotAvailable | E::ConversionFailure => ClipboardError::Empty,
        E::ClipboardUnavailable | E::ClipboardOccupied | E::Unknown { .. } => {
            ClipboardError::Unavailable
        },
    }
}
/// iced's answer as backstep's [`ClipboardEvent`]; non-text content reads as empty.
fn clipboard_event(event: &clipboard::Event) -> ClipboardEvent {
    match event {
        clipboard::Event::Read(result) => ClipboardEvent::Read(match result {
            Ok(content) => match content.as_ref() {
                clipboard::Content::Text(text) => Ok(text.clone()),
                _ => Err(ClipboardError::Empty),
            },
            Err(error) => Err(clipboard_error(error)),
        }),
        clipboard::Event::Written(result) => {
            ClipboardEvent::Written(result.as_ref().map(|_| ()).map_err(clipboard_error))
        },
    }
}
fn modifiers(v: keyboard::Modifiers) -> SessionModifiers {
    SessionModifiers {
        shift: v.shift(),
        control: v.control(),
        alt: v.alt(),
        meta: v.logo(),
    }
}
fn key(v: &keyboard::Key) -> SessionKey {
    use keyboard::{Key as K, key::Named as N};
    match v {
        K::Character(v) => SessionKey::Character(v.to_string()),
        K::Named(named) => match named {
            N::Enter => SessionKey::Enter,
            N::Tab => SessionKey::Tab,
            N::Backspace => SessionKey::Backspace,
            N::Delete => SessionKey::Delete,
            N::Escape => SessionKey::Escape,
            N::Space => SessionKey::Space,
            N::ArrowLeft => SessionKey::ArrowLeft,
            N::ArrowRight => SessionKey::ArrowRight,
            N::ArrowUp => SessionKey::ArrowUp,
            N::ArrowDown => SessionKey::ArrowDown,
            N::Home => SessionKey::Home,
            N::End => SessionKey::End,
            N::PageUp => SessionKey::PageUp,
            N::PageDown => SessionKey::PageDown,
            _ => SessionKey::Unidentified,
        },
        _ => SessionKey::Unidentified,
    }
}
impl Input {
    /// Ctrl/Cmd+C/X/V on `event` as a clipboard command; `None` otherwise.
    pub fn clipboard_command(event: &Event) -> Option<ClipboardCommand> {
        let Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Character(value),
            modifiers: m,
            ..
        }) = event
        else {
            return None;
        };
        if !(m.control() || m.logo()) || m.alt() || m.shift() {
            return None;
        }
        match value.to_ascii_lowercase().as_str() {
            "c" => Some(ClipboardCommand::Copy),
            "x" => Some(ClipboardCommand::Cut),
            "v" => Some(ClipboardCommand::Paste),
            _ => None,
        }
    }
    /// The request a clipboard `command` makes of iced's runtime; hand it over
    /// with [`Self::carry`]. Paste reads, and its answer reaches the owner as
    /// pasted text. Copy and Cut write `selection`, the document's selected
    /// text, which the host supplies: Genet's document-session-api has no
    /// selection query yet, so `None` refuses with `Unavailable`. A Cut's
    /// deletion is the owner's, on `ClipboardEvent::Written(Ok(()))`.
    pub fn clipboard_request(
        &mut self,
        command: ClipboardCommand,
        selection: Option<&str>,
    ) -> Result<ClipboardRequest, ClipboardError> {
        let request = match command {
            ClipboardCommand::Paste => ClipboardRequest::Read,
            ClipboardCommand::Copy | ClipboardCommand::Cut => {
                ClipboardRequest::Write(selection.ok_or(ClipboardError::Unavailable)?.to_owned())
            },
        };
        match request {
            ClipboardRequest::Read => self.reads += 1,
            ClipboardRequest::Write(_) => self.writes += 1,
        }
        Ok(request)
    }
    /// Add `request` to iced's request set (`Shell::clipboard_mut`); iced
    /// answers with an `Event::Clipboard` that [`Self::event`] maps.
    pub fn carry(request: ClipboardRequest, requests: &mut clipboard::Clipboard) {
        match request {
            ClipboardRequest::Read => requests.reads.push(clipboard::Kind::Text),
            ClipboardRequest::Write(text) => requests.write = Some(text.into()),
        }
    }
    /// The answer to a request this input made; `None` for another widget's.
    fn complete(&mut self, event: &clipboard::Event) -> Option<ClipboardEvent> {
        let pending = match event {
            clipboard::Event::Read(_) => &mut self.reads,
            clipboard::Event::Written(_) => &mut self.writes,
        };
        *pending = pending.checked_sub(1)?;
        Some(clipboard_event(event))
    }
    /// Focus(true) belongs to the actual containing widget's focus owner.
    pub fn event(
        &mut self,
        event: &Event,
        scroll: ScrollUnits,
    ) -> Result<Vec<OwnerInput>, &'static str> {
        let session = match event {
            Event::Keyboard(keyboard::Event::ModifiersChanged(m)) => {
                self.modifiers = modifiers(*m);
                return Ok(vec![]);
            },
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: k,
                modifiers: m,
                text,
                repeat,
                ..
            }) => {
                self.modifiers = modifiers(*m);
                let session_key = key(k);
                let printable =
                    matches!(&session_key, SessionKey::Character(_) | SessionKey::Space);
                let mut events = vec![OwnerInput::Session(SessionInput::Key {
                    key: session_key,
                    state: SessionButtonState::Pressed,
                    modifiers: self.modifiers,
                    repeat: *repeat,
                })];
                if let Some(text) = text {
                    if printable
                        && !text.is_empty()
                        && text.chars().all(|c| !c.is_control())
                        && (!self.modifiers.control || self.modifiers.alt)
                        && !self.modifiers.meta
                    {
                        events.push(OwnerInput::Session(SessionInput::KeyboardText(
                            text.to_string(),
                        )));
                    }
                }
                return Ok(events);
            },
            Event::Keyboard(keyboard::Event::KeyReleased {
                key: k,
                modifiers: m,
                ..
            }) => {
                self.modifiers = modifiers(*m);
                SessionInput::Key {
                    key: key(k),
                    state: SessionButtonState::Released,
                    modifiers: self.modifiers,
                    repeat: false,
                }
            },
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                self.cursor = Some([position.x, position.y]);
                SessionInput::PointerMoved {
                    x: position.x,
                    y: position.y,
                    modifiers: self.modifiers,
                }
            },
            Event::Mouse(mouse::Event::CursorLeft) => {
                self.cursor = None;
                SessionInput::Cancel
            },
            Event::Mouse(
                mouse::Event::ButtonPressed(button) | mouse::Event::ButtonReleased(button),
            ) => {
                let Some([x, y]) = self.cursor else {
                    return Ok(vec![]);
                };
                let button = match button {
                    mouse::Button::Left => SessionPointerButton::Primary,
                    mouse::Button::Right => SessionPointerButton::Secondary,
                    mouse::Button::Middle => SessionPointerButton::Auxiliary,
                    _ => return Err("document input has no extra mouse-button envelope"),
                };
                SessionInput::PointerButton {
                    x,
                    y,
                    button,
                    state: if matches!(event, Event::Mouse(mouse::Event::ButtonPressed(_))) {
                        SessionButtonState::Pressed
                    } else {
                        SessionButtonState::Released
                    },
                    modifiers: self.modifiers,
                }
            },
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let scroll = scroll.validate()?;
                let delta = match delta {
                    mouse::ScrollDelta::Lines { x, y } => {
                        [-x * scroll.line[0], -y * scroll.line[1]]
                    },
                    mouse::ScrollDelta::Pixels { x, y } => [-x, -y],
                };
                return Ok(self
                    .cursor
                    .map(|position| vec![OwnerInput::Wheel { position, delta }])
                    .unwrap_or_default());
            },
            Event::InputMethod(event) => SessionInput::Ime(match event {
                input_method::Event::Opened => SessionIme::Enabled,
                input_method::Event::Closed => SessionIme::Disabled,
                input_method::Event::Commit(text) => SessionIme::Commit(text.clone()),
                input_method::Event::Preedit(text, range) => SessionIme::Preedit {
                    text: text.clone(),
                    selection: range.as_ref().map(|r| (r.start, r.end)),
                },
            }),
            Event::Window(iced_wgpu::core::window::Event::Unfocused) => SessionInput::Focus(false),
            Event::Clipboard(event) => {
                return Ok(self
                    .complete(event)
                    .map(|event| match event {
                        ClipboardEvent::Read(Ok(text)) => OwnerInput::Session(SessionInput::Text(text)),
                        other => OwnerInput::Clipboard(other),
                    })
                    .into_iter()
                    .collect());
            },
            _ => return Ok(vec![]),
        };
        Ok(vec![OwnerInput::Session(session)])
    }
}
