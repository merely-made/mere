/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A controlled, numeric range scrubber.
//!
//! [`RangeScrubber`] is immutable props. The embedding application owns the
//! selected value and rebuilds the view after receiving a [`RangeScrubberEvent`].

use std::rc::Rc;

use crate::pod::GenetElement;
use crate::{
    GenetCtx, Key, NamedKey, PointerEvent, PointerPhase, ValueEvent, View, el, on_key, on_pointer,
    on_value,
};

/// A labelled value rendered as a semantic pin beside the scrubber track.
#[derive(Clone, Debug, PartialEq)]
pub struct RangeScrubberPin {
    /// The application value represented by this pin.
    pub value: f64,
    /// Text kept in the DOM for readers and adjacent visual treatments.
    pub label: String,
}

impl RangeScrubberPin {
    pub fn new(value: f64, label: impl Into<String>) -> Self {
        Self {
            value,
            label: label.into(),
        }
    }
}

/// Immutable, application-owned inputs for [`range_scrubber`].
#[derive(Clone, Debug, PartialEq)]
pub struct RangeScrubber {
    /// Inclusive lower bound.
    pub min: f64,
    /// Inclusive upper bound.
    pub max: f64,
    /// Current application-owned value.
    pub value: f64,
    /// Arrow-key and quantization interval.
    pub step: f64,
    /// Page-key interval.
    pub page_step: f64,
    /// Accessible name for the slider root.
    pub label: String,
    /// Application-described points on the track.
    pub pins: Vec<RangeScrubberPin>,
    /// A disabled explanation. An invalid range is disabled regardless.
    pub disabled_reason: Option<String>,
}

impl RangeScrubber {
    /// Construct controlled range props. Inputs are retained verbatim so an
    /// invalid source range is visibly inert rather than silently reinterpreted.
    pub fn new(min: f64, max: f64, value: f64) -> Self {
        Self {
            min,
            max,
            value,
            step: 1.0,
            page_step: 10.0,
            label: "Value".into(),
            pins: Vec::new(),
            disabled_reason: None,
        }
    }

    /// Set the quantized arrow and page increments.
    pub fn with_steps(mut self, step: f64, page_step: f64) -> Self {
        self.step = step;
        self.page_step = page_step;
        self
    }

    /// Set the accessible name.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Set semantic, application-labelled pins.
    pub fn with_pins(mut self, pins: impl IntoIterator<Item = RangeScrubberPin>) -> Self {
        self.pins = pins.into_iter().collect();
        self
    }

    /// Make the control inert with an explanation retained in the DOM.
    pub fn disabled(mut self, reason: impl Into<String>) -> Self {
        self.disabled_reason = Some(reason.into());
        self
    }

    /// Whether inputs form an interactive finite range.
    pub fn is_valid(&self) -> bool {
        self.min.is_finite()
            && self.max.is_finite()
            && self.min <= self.max
            && self.step.is_finite()
            && self.step > 0.0
            && self.page_step.is_finite()
            && self.page_step > 0.0
    }

    /// Whether this view accepts user input.
    pub fn is_disabled(&self) -> bool {
        !self.is_valid() || self.disabled_reason.is_some()
    }

    /// The single finite clamp-and-quantize policy used by every input route.
    /// Endpoints stay exact even when `max` is not an integral number of steps.
    pub fn reconcile(&self, input: f64) -> Option<f64> {
        if !self.is_valid() || !input.is_finite() {
            return None;
        }
        let bounded = input.clamp(self.min, self.max);
        if bounded == self.min || bounded == self.max {
            return Some(bounded);
        }
        Some(
            (self.min + ((bounded - self.min) / self.step).round() * self.step)
                .clamp(self.min, self.max),
        )
    }

    fn current(&self) -> f64 {
        self.reconcile(self.value)
            .unwrap_or_else(|| if self.min.is_finite() { self.min } else { 0.0 })
    }

    fn fraction(&self, value: f64) -> f64 {
        let span = self.max - self.min;
        if span > 0.0 {
            ((value - self.min) / span).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

/// An application-visible scrubber interaction. `Preview` is a pointer drag;
/// `Commit` comes from release, keyboard, or accessibility value setting.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RangeScrubberEvent {
    Preview(f64),
    Commit(f64),
}

fn number(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        value.to_string()
    }
}

/// Render a controlled range scrubber and report interactions to `on_event`.
pub fn range_scrubber<State, Action, OA, F>(
    props: RangeScrubber,
    on_event: F,
) -> impl View<State, Action, GenetCtx, Element = GenetElement>
where
    State: 'static,
    Action: 'static,
    OA: crate::OptionalAction<Action>,
    F: Fn(&mut State, RangeScrubberEvent) -> OA + 'static,
{
    let disabled = props.is_disabled();
    let current = props.current();
    let pct = props.fraction(current) * 100.0;
    let pins = props
        .pins
        .iter()
        .filter_map(|pin| {
            let value = props.reconcile(pin.value)?;
            Some(
                el::<_, State, Action>("span", pin.label.clone())
                    .attr("class", "range-scrubber-pin")
                    .attr("role", "note")
                    .attr("aria-label", pin.label.clone())
                    .attr("data-value", number(value))
                    .attr(
                        "style",
                        format!("left: {}%;", props.fraction(value) * 100.0),
                    ),
            )
        })
        .collect::<Vec<_>>();
    let reason = props.disabled_reason.clone().unwrap_or_else(|| {
        if props.is_valid() {
            String::new()
        } else {
            "Range inputs are invalid.".into()
        }
    });
    let aria_min = if props.min.is_finite() {
        props.min
    } else {
        0.0
    };
    let aria_max = if props.max.is_finite() {
        props.max
    } else {
        aria_min
    };
    let aria_step = if props.step.is_finite() {
        props.step
    } else {
        0.0
    };
    let aria_page_step = if props.page_step.is_finite() {
        props.page_step
    } else {
        0.0
    };
    let root = el::<_, State, Action>(
        "div",
        (
            el::<_, State, Action>("div", el::<_, State, Action>("div", ()))
                .attr("class", "range-scrubber-track")
                .attr("aria-hidden", "true")
                .attr(
                    "style",
                    format!("position: relative; --range-scrubber-thumb: {pct:.6}%;"),
                ),
            el::<_, State, Action>("div", pins).attr("class", "range-scrubber-pins"),
            el::<_, State, Action>("span", reason.clone())
                .attr("class", "range-scrubber-disabled-reason")
                .attr("aria-live", "polite"),
        ),
    )
    .attr(
        "class",
        if disabled {
            "range-scrubber disabled"
        } else {
            "range-scrubber"
        },
    )
    .attr("role", "slider")
    .attr("aria-label", props.label.clone())
    .attr("aria-valuemin", number(aria_min))
    .attr("aria-valuemax", number(aria_max))
    .attr("aria-valuenow", number(current))
    .attr("aria-valuetext", number(current))
    .attr("aria-disabled", disabled.to_string())
    .attr("aria-description", reason.clone())
    .attr("data-step", number(aria_step))
    .attr("data-page-step", number(aria_page_step))
    .attr("data-cambium-set-value", (!disabled).to_string())
    .attr("tabindex", if disabled { "-1" } else { "0" });

    let handler = Rc::new(on_event);
    let pointer_props = props.clone();
    let pointer_handler = handler.clone();
    let pointer = on_pointer(root, move |state: &mut State, event: PointerEvent| {
        if pointer_props.is_disabled()
            || !event.size.0.is_finite()
            || event.size.0 <= 0.0
            || !event.local.0.is_finite()
        {
            return;
        }
        let fraction = f64::from((event.local.0 / event.size.0).clamp(0.0, 1.0));
        let raw = pointer_props.min + (pointer_props.max - pointer_props.min) * fraction;
        if let Some(value) = pointer_props.reconcile(raw) {
            let kind = if event.phase == PointerPhase::Up {
                RangeScrubberEvent::Commit(value)
            } else {
                RangeScrubberEvent::Preview(value)
            };
            pointer_handler(state, kind);
        }
    });
    let key_props = props.clone();
    let key_handler = handler.clone();
    let keyed = on_key(pointer, move |state: &mut State, event| {
        if key_props.is_disabled() {
            return;
        }
        let current = key_props.current();
        let raw = match event.key {
            Key::Named(NamedKey::ArrowLeft | NamedKey::ArrowDown) => current - key_props.step,
            Key::Named(NamedKey::ArrowRight | NamedKey::ArrowUp) => current + key_props.step,
            Key::Named(NamedKey::PageDown) => current - key_props.page_step,
            Key::Named(NamedKey::PageUp) => current + key_props.page_step,
            Key::Named(NamedKey::Home) => key_props.min,
            Key::Named(NamedKey::End) => key_props.max,
            _ => return,
        };
        if let Some(value) = key_props.reconcile(raw) {
            key_handler(state, RangeScrubberEvent::Commit(value));
            event.prevent_default();
        }
    })
    .focusable(!disabled);
    let value_props = props;
    on_value(keyed, move |state: &mut State, event: ValueEvent| {
        if !value_props.is_disabled()
            && let Some(value) = value_props.reconcile(event.value)
        {
            handler(state, RangeScrubberEvent::Commit(value));
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    use crate::{AnyView, DomHandle, GenetAppRunner, KeyEvent, PointerButton};
    use genet_scripted_dom::ScriptedDom;

    struct State {
        value: f64,
        events: Vec<RangeScrubberEvent>,
    }
    type TestView = Box<dyn AnyView<State, (), GenetCtx, GenetElement>>;

    fn view(state: &State) -> TestView {
        Box::new(range_scrubber(
            RangeScrubber::new(2.0, 11.0, state.value).with_steps(2.0, 4.0),
            |state: &mut State, event| {
                state.value = match event {
                    RangeScrubberEvent::Preview(value) | RangeScrubberEvent::Commit(value) => value,
                };
                state.events.push(event);
            },
        ))
    }

    #[test]
    fn reconcile_clamps_and_quantizes_with_exact_endpoints() {
        let props = RangeScrubber::new(2.0, 11.0, 2.0).with_steps(2.0, 4.0);
        assert_eq!(props.reconcile(-4.0), Some(2.0));
        assert_eq!(props.reconcile(3.1), Some(4.0));
        assert_eq!(props.reconcile(10.1), Some(10.0));
        assert_eq!(props.reconcile(99.0), Some(11.0));
    }

    #[test]
    fn invalid_inputs_disable_and_never_reconcile() {
        let reversed = RangeScrubber::new(8.0, 2.0, 3.0);
        assert!(reversed.is_disabled());
        assert_eq!(reversed.reconcile(3.0), None);
        let invalid_step = RangeScrubber::new(0.0, 1.0, 0.5).with_steps(f64::NAN, 1.0);
        assert!(invalid_step.is_disabled());
        assert_eq!(invalid_step.reconcile(0.5), None);
    }

    #[test]
    fn nonfinite_requested_value_is_ignored() {
        let props = RangeScrubber::new(0.0, 1.0, 0.5);
        assert_eq!(props.reconcile(f64::INFINITY), None);
        assert_eq!(props.reconcile(f64::NAN), None);
    }

    #[test]
    fn pointer_keys_and_accessibility_share_controlled_policy() {
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let mut runner = GenetAppRunner::<_, _, _, ()>::new(
            dom,
            view,
            State {
                value: 2.0,
                events: Vec::new(),
            },
        );
        let root = runner.root();
        runner.dispatch_pointer_down(
            root,
            PointerEvent::new(PointerPhase::Down, (50.0, 0.0), (100.0, 10.0)),
        );
        assert_eq!(runner.state().value, 6.0);
        assert_eq!(
            runner.state().events.last(),
            Some(&RangeScrubberEvent::Preview(6.0))
        );
        runner.dispatch_pointer_up(
            PointerEvent::new(PointerPhase::Up, (100.0, 0.0), (100.0, 10.0))
                .with_button(PointerButton::Primary),
        );
        assert_eq!(runner.state().value, 11.0);
        runner.set_focus(Some(root));
        runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::ArrowLeft)));
        assert_eq!(runner.state().value, 10.0);
        runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::PageDown)));
        assert_eq!(runner.state().value, 6.0);
        runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::PageUp)));
        assert_eq!(runner.state().value, 10.0);
        runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Home)));
        assert_eq!(runner.state().value, 2.0);
        runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::End)));
        assert_eq!(runner.state().value, 11.0);
        runner.dispatch_value(root, ValueEvent { value: 3.1 });
        assert_eq!(runner.state().value, 4.0);
        let before = runner.state().events.len();
        runner.dispatch_value(root, ValueEvent { value: f64::NAN });
        assert_eq!(runner.state().events.len(), before);
    }

    #[test]
    fn disabled_scrubber_never_reports_input() {
        fn disabled_view(_state: &State) -> TestView {
            Box::new(range_scrubber(
                RangeScrubber::new(0.0, 10.0, 5.0).disabled("Unavailable"),
                |state: &mut State, event| state.events.push(event),
            ))
        }

        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let mut runner = GenetAppRunner::<_, _, _, ()>::new(
            dom,
            disabled_view,
            State {
                value: 5.0,
                events: Vec::new(),
            },
        );
        let root = runner.root();
        runner.dispatch_pointer_down(
            root,
            PointerEvent::new(PointerPhase::Down, (75.0, 0.0), (100.0, 10.0)),
        );
        runner.set_focus(Some(root));
        runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::End)));
        runner.dispatch_value(root, ValueEvent { value: 9.0 });
        assert!(runner.state().events.is_empty());
    }

    #[test]
    fn becoming_disabled_reconciles_focus() {
        struct FocusState {
            disabled: bool,
        }

        fn focus_view(
            state: &FocusState,
        ) -> Box<dyn AnyView<FocusState, (), GenetCtx, GenetElement>> {
            let mut props = RangeScrubber::new(0.0, 10.0, 5.0);
            if state.disabled {
                props = props.disabled("Unavailable");
            }
            Box::new(range_scrubber(props, |state: &mut FocusState, _| {
                state.disabled = true;
            }))
        }

        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let mut runner =
            GenetAppRunner::<_, _, _, ()>::new(dom, focus_view, FocusState { disabled: false });
        let root = runner.root();
        runner.set_focus(Some(root));
        runner.dispatch_value(root, ValueEvent { value: 7.0 });
        assert!(runner.state().disabled);
        assert_eq!(runner.focus(), None);
    }
}
