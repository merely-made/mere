// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
use super::{OwnerInput, ScrollUnits};
use backstep::{Clipboard, ClipboardError, ClipboardEvent, ClipboardRequest};
use cambium::ClipboardCommand;
use document_session_api::{
    SessionButtonState, SessionIme, SessionInput, SessionKey, SessionModifiers,
    SessionPointerButton,
};
/// egui's clipboard as backstep's call-shaped [`Clipboard`]. A write queues
/// egui's `CopyText` output command, which the integration carries to the
/// platform at the end of the frame; egui reports no failure, so a queued write
/// is `Ok`. Use it inside a frame (the output resets when one begins). Reads are
/// refused: egui reads the platform clipboard itself and delivers
/// `Event::Paste`, which [`Input::event`] passes on as session text.
pub struct EguiClipboard<'a>(pub &'a egui::Context);
impl Clipboard for EguiClipboard<'_> {
    fn read_text(&mut self) -> Result<String, ClipboardError> {
        Err(ClipboardError::Refused)
    }
    fn write_text(&mut self, text: &str) -> Result<(), ClipboardError> {
        self.0.copy_text(text.to_owned());
        Ok(())
    }
}
#[derive(Default)]
pub struct Input {
    cursor: Option<[f32; 2]>,
    modifiers: SessionModifiers,
}
fn modifiers(v: egui::Modifiers) -> SessionModifiers {
    SessionModifiers {
        shift: v.shift,
        control: v.ctrl,
        alt: v.alt,
        meta: v.mac_cmd,
    }
}
fn key(v: egui::Key) -> SessionKey {
    use egui::Key as K;
    match v {
        K::Enter => SessionKey::Enter,
        K::Tab => SessionKey::Tab,
        K::Backspace => SessionKey::Backspace,
        K::Delete => SessionKey::Delete,
        K::Escape => SessionKey::Escape,
        K::Space => SessionKey::Space,
        K::ArrowLeft => SessionKey::ArrowLeft,
        K::ArrowRight => SessionKey::ArrowRight,
        K::ArrowUp => SessionKey::ArrowUp,
        K::ArrowDown => SessionKey::ArrowDown,
        K::Home => SessionKey::Home,
        K::End => SessionKey::End,
        K::PageUp => SessionKey::PageUp,
        K::PageDown => SessionKey::PageDown,
        _ => {
            let name = v.name().to_lowercase();
            if name.chars().count() == 1 {
                SessionKey::Character(name)
            } else {
                SessionKey::Unidentified
            }
        },
    }
}
/// An unmodified Delete press and release: Genet's sessions delete a focused
/// control's non-collapsed range on it, and do nothing for a document range.
fn delete_selection() -> [OwnerInput; 2] {
    [SessionButtonState::Pressed, SessionButtonState::Released].map(|state| {
        OwnerInput::Session(SessionInput::Key {
            key: SessionKey::Delete,
            state,
            modifiers: SessionModifiers::default(),
            repeat: false,
        })
    })
}
impl Input {
    /// egui's Copy or Cut event as a clipboard command; `None` otherwise.
    /// Paste is not one: its text already arrives as `Event::Paste`.
    pub fn clipboard_command(event: &egui::Event) -> Option<ClipboardCommand> {
        match event {
            egui::Event::Copy => Some(ClipboardCommand::Copy),
            egui::Event::Cut => Some(ClipboardCommand::Cut),
            _ => None,
        }
    }
    /// Carry a clipboard `command` through `clipboard` ([`EguiClipboard`], or
    /// the host's own) and answer at once, as iced's pair answers later.
    /// Copy and Cut write `selection`, the session's `selected_text()`: the
    /// owner holds the `DocumentSession` and this input does not, so the owner
    /// passes it, and `None` refuses with `Unavailable` as iced does. A Cut
    /// whose write succeeded also yields an unmodified Delete, which deletes a
    /// focused control's range; a document range is copied and stays. Paste
    /// reads and yields session text. Other answers come back as
    /// [`OwnerInput::Clipboard`].
    pub fn clipboard(
        &self,
        command: ClipboardCommand,
        selection: Option<&str>,
        clipboard: &mut dyn Clipboard,
    ) -> Result<Vec<OwnerInput>, ClipboardError> {
        let request = match command {
            ClipboardCommand::Paste => ClipboardRequest::Read,
            ClipboardCommand::Copy | ClipboardCommand::Cut => {
                ClipboardRequest::Write(selection.ok_or(ClipboardError::Unavailable)?.to_owned())
            },
        };
        Ok(match request.serve(clipboard) {
            ClipboardEvent::Read(Ok(text)) => vec![OwnerInput::Session(SessionInput::Text(text))],
            ClipboardEvent::Written(Ok(())) if command == ClipboardCommand::Cut => {
                let mut events = vec![OwnerInput::Clipboard(ClipboardEvent::Written(Ok(())))];
                events.extend(delete_selection());
                events
            },
            other => vec![OwnerInput::Clipboard(other)],
        })
    }
    /// The caller separately grants per-appearance Focus from its egui Response.
    /// Window activation cannot grant keyboard authority to every appearance.
    #[allow(deprecated)]
    pub fn event(
        &mut self,
        event: &egui::Event,
        scroll: ScrollUnits,
    ) -> Result<Vec<OwnerInput>, &'static str> {
        let session = match event {
            egui::Event::ModifiersChanged(m) => {
                self.modifiers = modifiers(*m);
                return Ok(vec![]);
            },
            egui::Event::PointerMoved(p) => {
                self.cursor = Some([p.x, p.y]);
                SessionInput::PointerMoved {
                    x: p.x,
                    y: p.y,
                    modifiers: self.modifiers,
                }
            },
            egui::Event::PointerButton {
                pos,
                button,
                pressed,
                modifiers: m,
            } => {
                self.cursor = Some([pos.x, pos.y]);
                self.modifiers = modifiers(*m);
                let button = match button {
                    egui::PointerButton::Primary => SessionPointerButton::Primary,
                    egui::PointerButton::Secondary => SessionPointerButton::Secondary,
                    egui::PointerButton::Middle => SessionPointerButton::Auxiliary,
                    _ => return Err("document input has no extra mouse-button envelope"),
                };
                SessionInput::PointerButton {
                    x: pos.x,
                    y: pos.y,
                    button,
                    state: if *pressed {
                        SessionButtonState::Pressed
                    } else {
                        SessionButtonState::Released
                    },
                    modifiers: self.modifiers,
                }
            },
            egui::Event::PointerGone => {
                self.cursor = None;
                SessionInput::Cancel
            },
            egui::Event::Key {
                key: k,
                pressed,
                repeat,
                modifiers: m,
                ..
            } => {
                self.modifiers = modifiers(*m);
                SessionInput::Key {
                    key: key(*k),
                    state: if *pressed {
                        SessionButtonState::Pressed
                    } else {
                        SessionButtonState::Released
                    },
                    modifiers: self.modifiers,
                    repeat: *repeat,
                }
            },
            egui::Event::Text(text) if matches!(text.as_str(), "\r" | "\n" | "\r\n" | "\t") => {
                return Ok(vec![]);
            },
            egui::Event::Text(text) => SessionInput::KeyboardText(text.clone()),
            egui::Event::Paste(text) => SessionInput::Text(text.clone()),
            egui::Event::Ime(ime) => SessionInput::Ime(match ime {
                egui::ImeEvent::Enabled => SessionIme::Enabled,
                egui::ImeEvent::Disabled => SessionIme::Disabled,
                egui::ImeEvent::Commit(text) => SessionIme::Commit(text.clone()),
                egui::ImeEvent::Preedit {
                    text,
                    active_range_chars,
                } => {
                    let selection = if let Some(range) = active_range_chars {
                        let byte = |n| {
                            text.char_indices()
                                .map(|(b, _)| b)
                                .chain([text.len()])
                                .nth(n)
                        };
                        Some((
                            byte(range.start).ok_or("IME character range outside preedit")?,
                            byte(range.end).ok_or("IME character range outside preedit")?,
                        ))
                    } else {
                        None
                    };
                    SessionIme::Preedit {
                        text: text.clone(),
                        selection,
                    }
                },
                egui::ImeEvent::DeleteSurrounding { .. } => {
                    return Err("document input cannot express IME delete-surrounding");
                },
            }),
            egui::Event::WindowFocused(false) => SessionInput::Focus(false),
            egui::Event::MouseWheel { unit, delta, .. } => {
                let units = scroll.validate()?;
                let factor = match unit {
                    egui::MouseWheelUnit::Point => [1., 1.],
                    egui::MouseWheelUnit::Line => units.line,
                    egui::MouseWheelUnit::Page => units.page,
                };
                return Ok(self
                    .cursor
                    .map(|position| {
                        vec![OwnerInput::Wheel {
                            position,
                            delta: [-delta.x * factor[0], -delta.y * factor[1]],
                        }]
                    })
                    .unwrap_or_default());
            },
            // Copy and Cut need the session's selection; see `clipboard`.
            _ => return Ok(vec![]),
        };
        Ok(vec![OwnerInput::Session(session)])
    }
}
