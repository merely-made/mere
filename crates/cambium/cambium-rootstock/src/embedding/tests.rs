// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
use super::*;
use std::{cell::Cell, rc::Rc};
struct Producer {
    retired: Rc<Cell<u32>>,
    suspended: Rc<Cell<u32>>,
}
impl TextureProducer for Producer {
    fn render(&mut self, _: &crate::ProducerContext<'_>) -> Option<crate::ProducedTexture> {
        None
    }
    fn portable_scene(
        &mut self,
        frame: &ProducerFrameInfo,
    ) -> Result<ProducedScene, crate::PortableProducerError> {
        let mut scene = Scene::new(frame.logical_size.0 as u32, frame.logical_size.1 as u32);
        scene.push_rect(1., 2., 5., 7., [1., 0., 0., 1.]);
        Ok(ProducedScene {
            scene,
            source_identity: 18,
            source_revision: 4,
            alpha: netrender::SourceAlpha::Straight,
            encoding: crate::SourceEncoding::Srgb,
        })
    }
    fn suspend(&mut self) {
        self.suspended.set(self.suspended.get() + 1);
    }
    fn retire(&mut self) {
        self.retired.set(self.retired.get() + 1);
    }
}
fn view(id: u64) -> (View<Producer>, Rc<Cell<u32>>, Rc<Cell<u32>>) {
    let retired = Rc::new(Cell::new(0));
    let suspended = Rc::new(Cell::new(0));
    (
        View::new(
            id,
            Producer {
                retired: retired.clone(),
                suspended: suspended.clone(),
            },
            Placement {
                bounds: [10., 20., 100., 80.],
                clip: [10., 20., 50., 80.],
                scale: 2.,
            },
        )
        .unwrap(),
        retired,
        suspended,
    )
}
#[test]
fn independent_focus_capture_clip_and_coordinates() {
    let (mut a, _, _) = view(1);
    let (mut b, _, _) = view(2);
    assert!(
        a.input(SessionInput::Text("unfocused".into()), |_| ())
            .is_none()
    );
    a.input(SessionInput::Focus(true), |_| ());
    assert!(
        a.input(SessionInput::Text("focused".into()), |_| ())
            .is_some()
    );
    assert!(
        b.input(SessionInput::Text("independent".into()), |_| ())
            .is_none()
    );
    let event = || SessionInput::PointerMoved {
        x: 80.,
        y: 30.,
        modifiers: Default::default(),
    };
    assert!(a.input(event(), |_| ()).is_none());
    a.set_pointer_capture(true);
    let local = a.input(event(), |v| v).unwrap();
    assert!(matches!(
        local,
        SessionInput::PointerMoved { x: 70., y: 10., .. }
    ));
    a.input(SessionInput::Cancel, |_| ());
    assert!(a.input(event(), |_| ()).is_none());
    a.set_pointer_capture(true);
    a.input(SessionInput::Focus(false), |_| ());
    assert!(a.input(event(), |_| ()).is_none());
    b.set_visible(false);
    assert!(b.input(SessionInput::Focus(true), |_| ()).is_none());
}
#[test]
fn source_facts_survive_scale_resize_suspend_and_dispose() {
    let (mut a, retired, suspended) = view(1);
    let first = a.frame(None).unwrap();
    assert!(!a.needs_frame());
    let physical = first.physical_scene();
    assert_eq!(
        [physical.viewport_width, physical.viewport_height],
        [200, 160]
    );
    assert_eq!(physical.transforms[0].m[0], 1.);
    assert_eq!(physical.transforms[1].m[0], 2.);
    a.set_visible(false);
    assert_eq!(suspended.get(), 1);
    assert!(a.frame(None).is_err());
    a.set_visible(true);
    assert!(a.needs_frame());
    a.resize(Placement {
        scale: 1.5,
        ..a.placement()
    })
    .unwrap();
    let resized = a.frame(None).unwrap();
    assert_eq!(
        resized.produced.source_identity,
        first.produced.source_identity
    );
    assert_eq!(
        resized.produced.source_revision,
        first.produced.source_revision
    );
    a.dispose();
    assert_eq!(retired.get(), 1);
    assert!(a.frame(None).is_err());
    drop(a);
    assert_eq!(retired.get(), 1);
}
#[test]
fn invalid_geometry_is_refused_without_rebinding() {
    let (mut a, _, _) = view(1);
    let before = a.placement();
    assert!(
        a.resize(Placement {
            scale: f32::NAN,
            ..before
        })
        .is_err()
    );
    assert_eq!(a.placement(), before);
    assert!(
        Placement {
            bounds: [0., 0., -2., 10.],
            ..before
        }
        .validate()
        .is_err()
    );
}
#[test]
fn caller_clock_wakes_are_bounded_by_visibility_and_disposal() {
    let (mut a, _, _) = view(1);
    a.frame(None).unwrap();
    a.request_wake(std::time::Duration::from_millis(70));
    a.request_wake(std::time::Duration::from_millis(50));
    assert!(!a.wake(std::time::Duration::from_millis(40)));
    a.set_visible(false);
    assert_eq!(a.next_wake(), None);
    a.set_visible(true);
    assert!(a.wake(std::time::Duration::from_millis(50)));
    assert!(a.needs_frame());
    assert_eq!(a.next_wake(), None);
    a.dispose();
    a.request_wake(std::time::Duration::from_millis(80));
    assert_eq!(a.next_wake(), None);
}
#[cfg(all(feature = "embedding-egui", not(target_arch = "wasm32")))]
#[test]
fn egui_ime_character_ranges_are_lowered_to_unicode_byte_ranges() {
    let mut input = super::egui::Input::default();
    let units = ScrollUnits {
        line: [30., 40.],
        page: [100., 80.],
    };
    let events = input
        .event(
            &super::egui::toolkit::Event::Ime(super::egui::toolkit::ImeEvent::Preedit {
                text: "café 🦉".into(),
                active_range_chars: Some(3..6),
            }),
            units,
        )
        .unwrap();
    assert!(matches!(
        &events[0],
        OwnerInput::Session(SessionInput::Ime(
            document_session_api::SessionIme::Preedit {
                selection: Some((3, 10)),
                ..
            }
        ))
    ));
    assert!(
        input
            .event(
                &super::egui::toolkit::Event::Ime(super::egui::toolkit::ImeEvent::Preedit {
                    text: "é".into(),
                    active_range_chars: Some(0..3)
                }),
                units
            )
            .is_err()
    );
}
#[cfg(all(feature = "embedding-egui", not(target_arch = "wasm32")))]
#[test]
fn egui_keyboard_text_and_paste_keep_distinct_owners() {
    use super::egui::toolkit::Event;
    let mut input = super::egui::Input::default();
    let units = ScrollUnits {
        line: [30., 40.],
        page: [100., 80.],
    };
    assert_eq!(
        input.event(&Event::Text("é".into()), units).unwrap(),
        vec![OwnerInput::Session(SessionInput::KeyboardText("é".into()))]
    );
    assert_eq!(
        input.event(&Event::Paste("a\nb".into()), units).unwrap(),
        vec![OwnerInput::Session(SessionInput::Text("a\nb".into()))]
    );
    assert!(
        input
            .event(&Event::Text("\n".into()), units)
            .unwrap()
            .is_empty()
    );
}
#[cfg(all(feature = "embedding-iced", not(target_arch = "wasm32")))]
#[test]
fn iced_ime_bytes_and_caller_wheel_units_survive_translation() {
    use super::iced::backend::core::{Event, Point, input_method, mouse};
    let mut input = super::iced::Input::default();
    let units = ScrollUnits {
        line: [30., 40.],
        page: [100., 80.],
    };
    let events = input
        .event(
            &Event::InputMethod(input_method::Event::Preedit("café 🦉".into(), Some(3..10))),
            units,
        )
        .unwrap();
    assert!(matches!(
        &events[0],
        OwnerInput::Session(SessionInput::Ime(
            document_session_api::SessionIme::Preedit {
                selection: Some((3, 10)),
                ..
            }
        ))
    ));
    input
        .event(
            &Event::Mouse(mouse::Event::CursorMoved {
                position: Point::new(24., 32.),
            }),
            units,
        )
        .unwrap();
    let events = input
        .event(
            &Event::Mouse(mouse::Event::WheelScrolled {
                delta: mouse::ScrollDelta::Lines { x: 2., y: -1. },
            }),
            units,
        )
        .unwrap();
    assert_eq!(
        events,
        vec![OwnerInput::Wheel {
            position: [24., 32.],
            delta: [-60., 40.]
        }]
    );
}
#[cfg(all(feature = "embedding-iced", not(target_arch = "wasm32")))]
#[test]
fn iced_printable_key_pairs_text_without_enter_control_text() {
    use super::iced::backend::core::{Event, keyboard};
    use document_session_api::SessionKey;
    let mut input = super::iced::Input::default();
    let units = ScrollUnits {
        line: [30., 40.],
        page: [100., 80.],
    };
    let key = keyboard::Key::Character("a".into());
    let events = input
        .event(
            &Event::Keyboard(keyboard::Event::KeyPressed {
                key: key.clone(),
                modified_key: key,
                physical_key: keyboard::key::Physical::Code(keyboard::key::Code::KeyA),
                location: keyboard::Location::Standard,
                modifiers: keyboard::Modifiers::default(),
                text: Some("a".into()),
                repeat: true,
            }),
            units,
        )
        .unwrap();
    assert!(matches!(&events[..],
        [OwnerInput::Session(SessionInput::Key { key: SessionKey::Character(text), repeat: true, .. }),
         OwnerInput::Session(SessionInput::KeyboardText(produced))] if text == "a" && produced == "a"));
    let key = keyboard::Key::Named(keyboard::key::Named::Enter);
    let events = input
        .event(
            &Event::Keyboard(keyboard::Event::KeyPressed {
                key: key.clone(),
                modified_key: key,
                physical_key: keyboard::key::Physical::Code(keyboard::key::Code::Enter),
                location: keyboard::Location::Standard,
                modifiers: keyboard::Modifiers::default(),
                text: Some("\r".into()),
                repeat: false,
            }),
            units,
        )
        .unwrap();
    assert!(matches!(
        &events[..],
        [OwnerInput::Session(SessionInput::Key {
            key: SessionKey::Enter,
            ..
        })]
    ));
}
#[cfg(all(feature = "embedding-iced", not(target_arch = "wasm32")))]
#[test]
fn iced_clipboard_requests_pair_with_their_answers() {
    use super::iced::backend::core::{Clipboard, Event, clipboard, keyboard};
    use backstep::{ClipboardError, ClipboardEvent, ClipboardRequest};
    use cambium::ClipboardCommand;
    use std::sync::Arc;
    let mut input = super::iced::Input::default();
    let units = ScrollUnits {
        line: [1., 1.],
        page: [1., 1.],
    };
    let key = keyboard::Key::Character("v".into());
    let paste = Event::Keyboard(keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: keyboard::key::Physical::Code(keyboard::key::Code::KeyV),
        location: keyboard::Location::Standard,
        modifiers: keyboard::Modifiers::CTRL,
        text: None,
        repeat: false,
    });
    assert_eq!(
        super::iced::Input::clipboard_command(&paste),
        Some(ClipboardCommand::Paste)
    );
    let read = |text: &str| {
        Event::Clipboard(clipboard::Event::Read(Ok(Arc::new(
            clipboard::Content::Text(text.into()),
        ))))
    };
    // An answer to another widget's request is not this owner's.
    assert_eq!(input.event(&read("theirs"), units), Ok(vec![]));
    // Paste: a read request, answered by pasted text.
    let request = input.clipboard_request(ClipboardCommand::Paste, None);
    assert_eq!(request, Ok(ClipboardRequest::Read));
    let mut requests = Clipboard::new();
    super::iced::Input::carry(request.unwrap(), &mut requests);
    assert_eq!(requests.reads, vec![clipboard::Kind::Text]);
    assert_eq!(
        input.event(&read("pasted"), units),
        Ok(vec![OwnerInput::Session(SessionInput::Text("pasted".into()))])
    );
    // Copy without a selection the host can supply is refused, not requested.
    assert_eq!(
        input.clipboard_request(ClipboardCommand::Copy, None),
        Err(ClipboardError::Unavailable)
    );
    let request = input.clipboard_request(ClipboardCommand::Cut, Some("cut"));
    assert_eq!(request, Ok(ClipboardRequest::Write("cut".into())));
    super::iced::Input::carry(request.unwrap(), &mut requests);
    assert_eq!(requests.write, Some(clipboard::Content::Text("cut".into())));
    let written = Event::Clipboard(clipboard::Event::Written(Err(
        clipboard::Error::ClipboardOccupied,
    )));
    assert_eq!(
        input.event(&written, units),
        Ok(vec![OwnerInput::Clipboard(ClipboardEvent::Written(Err(
            ClipboardError::Unavailable
        )))])
    );
    // A failed read reaches the owner as a failure; non-text reads as empty.
    input.clipboard_request(ClipboardCommand::Paste, None).unwrap();
    let files = Event::Clipboard(clipboard::Event::Read(Ok(Arc::new(
        clipboard::Content::Files(vec![]),
    ))));
    assert_eq!(
        input.event(&files, units),
        Ok(vec![OwnerInput::Clipboard(ClipboardEvent::Read(Err(
            ClipboardError::Empty
        )))])
    );
}
