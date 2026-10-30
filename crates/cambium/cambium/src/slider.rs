/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A drag-to-set [`slider`], the control on top of the pointer-drag foundation.
//!
//! State is a normalized value in `0.0..=1.0`; pressing or dragging on the track
//! sets it from the pointer's position (`local.x / size.x`). Composable via
//! [`lens`](crate::lens) like the other controls; the app scales the fraction to
//! its own range.

use crate::pod::GenetElement;
use crate::{
    GenetCtx, Key, NamedKey, PointerEvent, PointerPhase, ValueEvent, View, el, on_key, on_pointer,
    on_value,
};

/// The state of a [`slider`]: a normalized value in `0.0..=1.0`. Composable via
/// [`lens`](crate::lens).
#[derive(Clone, Debug, PartialEq)]
pub struct Slider {
    /// The value as a fraction of the track (`0.0` = left, `1.0` = right).
    pub value: f32,
    /// Arrow-key increment in normalized units.
    pub step: f32,
    /// Page-key increment in normalized units.
    pub page_step: f32,
    /// Accessible name announced for the control.
    pub label: String,
}

impl Slider {
    /// A slider at `value` (clamped to `0.0..=1.0`).
    pub fn new(value: f32) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
            step: 0.01,
            page_step: 0.1,
            label: "Value".into(),
        }
    }

    /// Configure keyboard increments. Values are clamped to `0.0..=1.0`.
    pub fn with_steps(mut self, step: f32, page_step: f32) -> Self {
        self.step = step.clamp(0.0, 1.0);
        self.page_step = page_step.clamp(0.0, 1.0);
        self
    }

    /// Set the accessible name announced for the control.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }
}

impl Default for Slider {
    fn default() -> Self {
        Self::new(0.0)
    }
}

/// A drag-to-set slider over a [`Slider`]. The track is a `slider-track` element
/// (positioned, so the thumb anchors to it) holding an absolutely-placed
/// `slider-thumb` at `value` along it. Pressing or dragging anywhere on the
/// track sets `value` to the pointer fraction (`local.x / size.x`, clamped);
/// pointer capture keeps the drag live if the cursor leaves the track.
///
/// `+ use<>` keeps the opaque type from borrowing `state` (the percentage is
/// formatted into an owned style string).
pub fn slider(state: &Slider) -> impl View<Slider, (), GenetCtx, Element = GenetElement> + use<> {
    let pct = state.value.clamp(0.0, 1.0) * 100.0;
    // The thumb: an absolute box at `left: pct%` of the relative track.
    let thumb = el::<_, Slider, ()>("div", ())
        .attr("class", "slider-thumb")
        .attr("style", format!("position: absolute; left: {pct}%;"));
    let pointer = on_pointer(
        el::<_, Slider, ()>("div", thumb)
            .attr("class", "slider-track")
            .attr("role", "slider")
            .attr("aria-label", state.label.clone())
            .attr("aria-valuemin", "0")
            .attr("aria-valuemax", "1")
            .attr("aria-valuenow", state.value.clamp(0.0, 1.0).to_string())
            .attr("aria-valuetext", format!("{pct:.0}%"))
            .attr("tabindex", "0")
            .attr("style", "position: relative;"),
        |s: &mut Slider, e: PointerEvent| {
            // Down / Move / Up all set the value from the pointer fraction, so a
            // press jumps to that point and a drag tracks it.
            if e.phase != PointerPhase::Cancel && e.size.0 > 0.0 {
                s.value = (e.local.0 / e.size.0).clamp(0.0, 1.0);
            }
        },
    );
    let keyed = on_key(pointer, |s: &mut Slider, event| {
        let handled = match &event.key {
            Key::Named(NamedKey::ArrowLeft | NamedKey::ArrowDown) => {
                s.value = (s.value - s.step).clamp(0.0, 1.0);
                true
            },
            Key::Named(NamedKey::ArrowRight | NamedKey::ArrowUp) => {
                s.value = (s.value + s.step).clamp(0.0, 1.0);
                true
            },
            Key::Named(NamedKey::PageDown) => {
                s.value = (s.value - s.page_step).clamp(0.0, 1.0);
                true
            },
            Key::Named(NamedKey::PageUp) => {
                s.value = (s.value + s.page_step).clamp(0.0, 1.0);
                true
            },
            Key::Named(NamedKey::Home) => {
                s.value = 0.0;
                true
            },
            Key::Named(NamedKey::End) => {
                s.value = 1.0;
                true
            },
            _ => false,
        };
        if handled {
            event.prevent_default();
        }
    });
    on_value(keyed, |s: &mut Slider, event: ValueEvent| {
        if event.value.is_finite() {
            s.value = event.value.clamp(0.0, 1.0) as f32;
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_clamps() {
        assert_eq!(Slider::new(0.5).value, 0.5);
        assert_eq!(Slider::new(-1.0).value, 0.0);
        assert_eq!(Slider::new(2.0).value, 1.0);
    }

    #[test]
    fn accessibility_values_route_to_the_slider_and_respect_its_range() {
        use std::{cell::RefCell, rc::Rc};

        use crate::{AnyView, DomHandle, GenetAppRunner};
        use genet_scripted_dom::ScriptedDom;

        type SliderView = Box<dyn AnyView<Slider, (), GenetCtx, GenetElement>>;
        fn view(state: &Slider) -> SliderView {
            Box::new(slider(state))
        }
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let mut runner = GenetAppRunner::<_, _, _, ()>::new(dom, view, Slider::new(0.5));
        let root = runner.root();
        for (requested, expected) in [(0.37, 0.37), (-1.0, 0.0), (2.0, 1.0)] {
            runner.dispatch_value(root, ValueEvent { value: requested });
            assert_eq!(runner.state().value, expected);
        }
        for refused in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            runner.dispatch_value(root, ValueEvent { value: refused });
            assert_eq!(runner.state().value, 1.0);
        }
    }

    #[test]
    fn pointer_cancel_keeps_last_drag_value() {
        use std::{cell::RefCell, rc::Rc};

        use crate::{AnyView, DomHandle, GenetAppRunner, PointerEvent, PointerPhase};
        use genet_scripted_dom::ScriptedDom;

        type SliderView = Box<dyn AnyView<Slider, (), GenetCtx, GenetElement>>;
        fn view(state: &Slider) -> SliderView {
            Box::new(slider(state))
        }
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let mut runner = GenetAppRunner::<_, _, _, ()>::new(dom, view, Slider::new(0.5));
        let track = runner.root();

        runner.dispatch_pointer_down(
            track,
            PointerEvent::new(PointerPhase::Down, (30.0, 0.0), (100.0, 10.0)),
        );
        assert!((runner.state().value - 0.3).abs() < f32::EPSILON);
        runner.dispatch_pointer_up(PointerEvent::new(
            PointerPhase::Cancel,
            (90.0, 0.0),
            (100.0, 10.0),
        ));
        assert!((runner.state().value - 0.3).abs() < f32::EPSILON);
    }
}
