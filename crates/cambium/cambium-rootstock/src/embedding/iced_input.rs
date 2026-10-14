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
use iced_wgpu::core::{Event, input_method, keyboard, mouse};
#[derive(Default)]
pub struct Input {
    cursor: Option<[f32; 2]>,
    modifiers: SessionModifiers,
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
                let mut events = vec![OwnerInput::Session(SessionInput::Key {
                    key: key(k),
                    state: SessionButtonState::Pressed,
                    modifiers: self.modifiers,
                    repeat: *repeat,
                })];
                if let Some(text) = text {
                    if !text.is_empty()
                        && (!self.modifiers.control || self.modifiers.alt)
                        && !self.modifiers.meta
                    {
                        events.push(OwnerInput::Session(SessionInput::Text(text.to_string())));
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
            Event::Clipboard(_) => return Err("clipboard requests remain host operations"),
            _ => return Ok(vec![]),
        };
        Ok(vec![OwnerInput::Session(session)])
    }
}
