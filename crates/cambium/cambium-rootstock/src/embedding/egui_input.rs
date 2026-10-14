// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
use super::{OwnerInput, ScrollUnits};
use document_session_api::{
    SessionButtonState, SessionIme, SessionInput, SessionKey, SessionModifiers,
    SessionPointerButton,
};
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
impl Input {
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
            egui::Event::Text(text) | egui::Event::Paste(text) => SessionInput::Text(text.clone()),
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
            egui::Event::Copy | egui::Event::Cut => {
                return Err("clipboard copy/cut remains a host operation");
            },
            _ => return Ok(vec![]),
        };
        Ok(vec![OwnerInput::Session(session)])
    }
}
