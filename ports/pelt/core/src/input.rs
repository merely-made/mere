// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Shared retained-content input. Positions are logical content coordinates;
//! only the producer boundary converts them to physical pixels.
use inker::{DragEvent, PhysicalPosition, PointerEvent, SessionInput, SessionModifiers};

#[derive(Clone, Debug)]
pub enum PeltInput {
    Session(SessionInput),
    /// Lossless native keyboard envelope for surface producers.
    Keyboard(inker::KeyboardEvent),
    /// Engine-owned scrolling after the host has handled document keys.
    ScrollKey(inker::SessionScrollKey),
    Pointer(PointerEvent),
    Drag(DragEvent),
    Wheel {
        position: PhysicalPosition,
        delta: (f32, f32),
        modifiers: SessionModifiers,
    },
}

impl From<SessionInput> for PeltInput {
    fn from(input: SessionInput) -> Self {
        Self::Session(input)
    }
}

impl PeltInput {
    pub fn position(&self) -> Option<(f32, f32)> {
        match self {
            Self::Pointer(event) => Some((event.position.x, event.position.y)),
            Self::Drag(event) => Some((event.position.x, event.position.y)),
            Self::Wheel { position, .. } => Some((position.x, position.y)),
            Self::Session(
                SessionInput::PointerMoved { x, y, .. } | SessionInput::PointerButton { x, y, .. },
            ) => Some((*x, *y)),
            _ => None,
        }
    }

    pub fn translate(&mut self, dx: f32, dy: f32) {
        match self {
            Self::Pointer(event) => {
                event.position.x += dx;
                event.position.y += dy;
            },
            Self::Drag(event) => {
                event.position.x += dx;
                event.position.y += dy;
            },
            Self::Wheel { position, .. } => {
                position.x += dx;
                position.y += dy;
            },
            Self::Session(
                SessionInput::PointerMoved { x, y, .. } | SessionInput::PointerButton { x, y, .. },
            ) => {
                *x += dx;
                *y += dy;
            },
            _ => {},
        }
    }

    /// Express a host input measured in physical pixels in layout pixels:
    /// positions, wheel deltas and pointer contact sizes divided by `scale`,
    /// the layout scale the placements were registered at. Placements carry
    /// logical rects, so a host that receives physical pointers converts once
    /// here rather than at each producer.
    pub fn to_layout(&mut self, scale: f32) {
        if !(scale.is_finite() && scale > 0.0) {
            return;
        }
        match self {
            Self::Pointer(event) => {
                event.position.x /= scale;
                event.position.y /= scale;
                event.width /= scale;
                event.height /= scale;
            },
            Self::Drag(event) => {
                event.position.x /= scale;
                event.position.y /= scale;
            },
            Self::Wheel {
                position, delta, ..
            } => {
                position.x /= scale;
                position.y /= scale;
                delta.0 /= scale;
                delta.1 /= scale;
            },
            Self::Session(
                SessionInput::PointerMoved { x, y, .. } | SessionInput::PointerButton { x, y, .. },
            ) => {
                *x /= scale;
                *y /= scale;
            },
            _ => {},
        }
    }

    pub(crate) fn pointer(&self) -> Option<(i32, inker::PointerPhase, Option<inker::MouseButton>)> {
        use inker::{MouseButton, PointerPhase, SessionButtonState, SessionPointerButton};
        match self {
            Self::Pointer(event) => Some((event.pointer_id, event.phase, event.button)),
            Self::Session(SessionInput::PointerMoved { .. }) => Some((0, PointerPhase::Move, None)),
            Self::Session(SessionInput::PointerButton { button, state, .. }) => Some((
                0,
                if *state == SessionButtonState::Pressed {
                    PointerPhase::Down
                } else {
                    PointerPhase::Up
                },
                Some(match button {
                    SessionPointerButton::Primary => MouseButton::Left,
                    SessionPointerButton::Secondary => MouseButton::Right,
                    SessionPointerButton::Auxiliary => MouseButton::Middle,
                }),
            )),
            Self::Session(SessionInput::Cancel) => Some((0, PointerPhase::Cancel, None)),
            _ => None,
        }
    }

    pub(crate) fn cancel(&self) -> Self {
        match self {
            Self::Pointer(event) => {
                let mut event = event.clone();
                event.phase = inker::PointerPhase::Cancel;
                event.buttons = inker::PointerButtons::NONE;
                event.button = None;
                Self::Pointer(event)
            },
            _ => Self::Session(SessionInput::Cancel),
        }
    }
}

pub(crate) fn button_bit(button: Option<inker::MouseButton>) -> u16 {
    match button {
        Some(inker::MouseButton::Left) => 1,
        Some(inker::MouseButton::Right) => 2,
        Some(inker::MouseButton::Middle) => 4,
        Some(inker::MouseButton::Back) => 8,
        Some(inker::MouseButton::Forward) => 16,
        None => 0,
    }
}

pub(crate) fn modifiers(value: SessionModifiers) -> inker::KeyboardModifiers {
    inker::KeyboardModifiers {
        shift: value.shift,
        ctrl: value.control,
        alt: value.alt,
        meta: value.meta,
    }
}

/// Construct a mouse pointer without pretending optional hardware sensors exist.
pub fn mouse_pointer(
    pointer_id: i32,
    phase: inker::PointerPhase,
    position: PhysicalPosition,
    button: Option<inker::MouseButton>,
    buttons: inker::PointerButtons,
    modifiers: SessionModifiers,
) -> PointerEvent {
    PointerEvent {
        pointer_id,
        pointer_type: inker::PointerType::Mouse,
        is_primary: true,
        phase,
        position,
        button,
        buttons,
        width: 1.0,
        height: 1.0,
        pressure: None,
        tangential_pressure: None,
        tilt_x: None,
        tilt_y: None,
        twist: None,
        altitude_angle: None,
        azimuth_angle: None,
        modifiers: self::modifiers(modifiers),
    }
}

/// Lower the representable logical part of a native keyboard event. Hosts with
/// logical keys or a composition lifecycle should use SessionInput directly.
pub(crate) fn document_keyboard(event: inker::KeyboardEvent) -> Result<SessionInput, String> {
    use inker::{SessionButtonState, SessionIme, SessionKey};
    if event.key_code == 0 && event.pressed {
        if let Some(text) = event.text.as_ref() {
            return Ok(SessionInput::Ime(SessionIme::Commit(text.clone())));
        }
    }
    let key = match event.key_code {
        8 => SessionKey::Backspace,
        9 => SessionKey::Tab,
        13 => SessionKey::Enter,
        27 => SessionKey::Escape,
        32 => SessionKey::Space,
        33 => SessionKey::PageUp,
        34 => SessionKey::PageDown,
        35 => SessionKey::End,
        36 => SessionKey::Home,
        37 => SessionKey::ArrowLeft,
        38 => SessionKey::ArrowUp,
        39 => SessionKey::ArrowRight,
        40 => SessionKey::ArrowDown,
        46 => SessionKey::Delete,
        _ if event.text.is_some() => SessionKey::Character(event.text.unwrap()),
        48..=57 | 65..=90 => {
            SessionKey::Character(char::from_u32(event.key_code).unwrap().to_string())
        },
        _ => {
            return Err(format!(
                "keyboard code {} cannot be represented by document input; use SessionInput::Key or SessionInput::Ime",
                event.key_code
            ));
        },
    };
    Ok(SessionInput::Key {
        key,
        state: if event.pressed {
            SessionButtonState::Pressed
        } else {
            SessionButtonState::Released
        },
        modifiers: SessionModifiers {
            shift: event.modifiers.shift,
            control: event.modifiers.ctrl,
            alt: event.modifiers.alt,
            meta: event.modifiers.meta,
        },
        repeat: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physical_pointers_and_wheels_become_layout_pixels() {
        let mut pointer = PeltInput::Pointer(mouse_pointer(
            1,
            inker::PointerPhase::Move,
            PhysicalPosition { x: 200.0, y: 100.0 },
            None,
            inker::PointerButtons::NONE,
            SessionModifiers::default(),
        ));
        pointer.to_layout(2.0);
        assert_eq!(pointer.position(), Some((100.0, 50.0)));
        let mut wheel = PeltInput::Wheel {
            position: PhysicalPosition { x: 40.0, y: 20.0 },
            delta: (8.0, -16.0),
            modifiers: SessionModifiers::default(),
        };
        wheel.to_layout(2.0);
        let PeltInput::Wheel {
            position, delta, ..
        } = wheel
        else {
            unreachable!()
        };
        assert_eq!((position.x, position.y, delta), (20.0, 10.0, (4.0, -8.0)));
        let mut key = PeltInput::Session(SessionInput::Cancel);
        key.to_layout(2.0);
        assert!(matches!(key, PeltInput::Session(SessionInput::Cancel)));
        let mut unchanged = PeltInput::Session(SessionInput::PointerMoved {
            x: 6.0,
            y: 8.0,
            modifiers: SessionModifiers::default(),
        });
        unchanged.to_layout(0.0);
        assert_eq!(unchanged.position(), Some((6.0, 8.0)));
    }
}
