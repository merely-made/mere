// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A small, keyboard-accessible doorway attached to a projected field. The
//! host supplies screen geometry and resolves the existing scope on activation.
use crate::{OptionalAction, ShellControl, ShellSlot, el, shell_control};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScopeFieldCommand {
    Peek,
    Enter,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScopeField {
    pub label: String,
    /// Left, top, right, bottom in the same layout-pixel space as the aperture.
    pub bounds: [f32; 4],
    pub aperture: [f32; 4],
    pub can_enter: bool,
    pub scale: f32,
}

impl ScopeField {
    /// Keep the doorway attached to the visible part of the field. A tiny or
    /// offscreen field gets no floating substitute elsewhere in the window.
    pub fn placement(&self) -> Option<[f32; 3]> {
        if self
            .bounds
            .iter()
            .chain(&self.aperture)
            .any(|v| !v.is_finite())
            || !self.scale.is_finite()
            || self.scale <= 0.0
        {
            return None;
        }
        let [l, t, r, b] = self.bounds;
        let [vl, vt, vr, vb] = self.aperture;
        let (l, t, r, b) = (l.max(vl), t.max(vt), r.min(vr), b.min(vb));
        if r - l < 112.0 * self.scale || b - t < 36.0 * self.scale {
            return None;
        }
        Some([
            l + 6.0 * self.scale,
            t + 6.0 * self.scale,
            (r - l - 12.0 * self.scale).min(240.0 * self.scale),
        ])
    }
}

pub fn scope_field_controls<
    State: 'static,
    Action: 'static,
    OA: OptionalAction<Action> + 'static,
>(
    field: &ScopeField,
    command: impl Fn(&mut State, ScopeFieldCommand) -> OA + 'static,
) -> Option<ShellSlot<State, Action>> {
    let [x, y, width] = field.placement()?;
    let command = Rc::new(command);
    let peek = command.clone();
    Some(Box::new(el("div", (
        shell_control(field.label.clone(),format!("Preview field: {}",field.label),ShellControl::default(),
            move |state,_|peek(state,ScopeFieldCommand::Peek)),
        shell_control("↘",format!("Enter field: {}",field.label),ShellControl {disabled:!field.can_enter,..Default::default()},
            move |state,_|command(state,ScopeFieldCommand::Enter)),
    ))
    .attr("class","scope-field-controls")
    .attr("role","toolbar")
    .attr("aria-label",format!("Field navigation: {}",field.label))
    .attr("style",format!("position:absolute;left:{x}px;top:{y}px;max-width:{width}px;font-size:{}px;--scope-field-target:{}px;",13.0*field.scale,26.0*field.scale))))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn field(bounds: [f32; 4]) -> ScopeField {
        ScopeField {
            label: "Reading layout".into(),
            bounds,
            aperture: [0., 0., 800., 600.],
            can_enter: true,
            scale: 1.0,
        }
    }
    #[test]
    fn doorway_follows_the_field_and_clips_to_its_aperture() {
        assert_eq!(
            field([100., 80., 600., 450.]).placement(),
            Some([106., 86., 240.])
        );
        assert_eq!(
            field([-100., -80., 200., 450.]).placement(),
            Some([6., 6., 188.])
        );
        assert_eq!(
            field([650., 500., 1000., 900.]).placement(),
            Some([656., 506., 138.])
        );
    }
    #[test]
    fn hidden_tiny_and_invalid_fields_do_not_leave_detached_controls() {
        for bounds in [
            [810., 80., 1000., 450.],
            [10., 10., 100., 100.],
            [10., 10., 500., 40.],
            [0., 0., f32::NAN, 600.],
            [400., 100., 300., 400.],
        ] {
            assert_eq!(field(bounds).placement(), None);
        }
    }

    #[test]
    fn interface_scale_changes_target_size_without_changing_the_field() {
        let mut value = field([100., 80., 600., 450.]);
        value.scale = 2.0;
        assert_eq!(value.placement(), Some([112., 92., 476.]));
        assert_eq!(value.bounds, [100., 80., 600., 450.]);
    }

    #[test]
    fn preview_and_entry_are_separate_named_buttons_and_entry_can_be_disabled() {
        use crate::{DomHandle, GenetAppRunner};
        use genet_scripted_dom::ScriptedDom;
        use std::cell::RefCell;
        fn view(_: &()) -> ShellSlot<(), ()> {
            let mut field = field([100., 80., 600., 450.]);
            field.can_enter = false;
            scope_field_controls(&field, |_: &mut (), _| ()).unwrap()
        }
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let runner = GenetAppRunner::new(dom.clone(), view, ());
        let html = dom.borrow().outer_html(runner.root());
        assert!(html.contains("Field navigation: Reading layout"));
        assert!(html.contains("Preview field: Reading layout"));
        assert!(html.contains("Enter field: Reading layout"));
        assert!(html.contains("aria-disabled=\"true\""));
    }
}
