/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A radio-button group with component-owned selection and keyboard behavior.
//!
//! State is which option is selected; clicking one selects it and (since the
//! group is a single index) deselects the rest. The group owns the WAI-ARIA
//! radio pattern's Space activation, wrapping Arrow-key movement, roving
//! `tabindex`, and focus transfer. `Home` and `End` are retained as Cambium
//! extensions. Options may be disabled ([`RadioOption`]); navigation skips
//! them. The indicator is its own element, drawn by the shell sheet. ARIA roles expose the component's semantics; the generic runner
//! does not infer behavior from them.

use std::rc::Rc;

use crate::pod::GenetElement;
use crate::{GenetCtx, Key, NamedKey, View, el, on_click, on_key, request_focus};

/// The state of a radio group: the index of the selected option in the
/// `options` slice passed to [`radio_group`]. Composable via [`lens`](crate::lens).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RadioGroup {
    /// Index of the chosen option. Out of range renders all options unselected.
    pub selected: usize,
    /// Accessible name announced for the group.
    pub label: String,
    /// Retained target used to move DOM focus after keyboard navigation.
    pub focus_request: Option<usize>,
}

impl RadioGroup {
    /// A group with `selected` chosen.
    pub fn new(selected: usize) -> Self {
        Self {
            selected,
            label: "Options".into(),
            focus_request: None,
        }
    }

    /// Set the accessible name announced for the group.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }
}

impl Default for RadioGroup {
    fn default() -> Self {
        Self::new(0)
    }
}

/// One option of a [`radio_options`] group.
///
/// A disabled option keeps its place and its saved selection but cannot be
/// activated, focused or reached by arrow keys; it exposes
/// `aria-disabled="true"`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RadioOption {
    pub label: String,
    pub disabled: bool,
    /// Extra attributes for the option's element (host test hooks, ids).
    pub attrs: Vec<(String, String)>,
}

impl RadioOption {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            disabled: false,
            attrs: Vec::new(),
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn with_attr(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.attrs.push((name.into(), value.into()));
        self
    }
}

/// A radio group over a [`RadioGroup`] and option labels: one row per option,
/// clicking a row selects it (and only it). See [`radio_options`] for disabled
/// options.
pub fn radio_group<OptionLabel>(
    state: &RadioGroup,
    options: &[OptionLabel],
) -> impl View<RadioGroup, (), GenetCtx, Element = GenetElement> + use<OptionLabel>
where
    OptionLabel: AsRef<str>,
{
    let options: Vec<RadioOption> = options
        .iter()
        .map(|label| RadioOption::new(label.as_ref()))
        .collect();
    radio_options(state, &options)
}

/// A radio group whose options may be disabled.
///
/// Each row is a `role="radio"` element (`radio`, `radio selected`,
/// `radio disabled`) with `aria-checked` and `aria-disabled`, holding a
/// `radio-indicator` span the shell sheet draws and a `radio-label` span with
/// the text, inside a `role="radiogroup"` container. Arrow keys wrap over the
/// enabled options; `Home` / `End` jump to the first / last enabled one; Space
/// activates the focused one. Exactly one enabled option is a tab stop: the
/// selected one, else the first enabled. A disabled selection stays selected
/// (it is saved state) but is not focusable. `+ use<>` keeps the opaque type
/// from borrowing `state` / `options` (the options are cloned in).
pub fn radio_options(
    state: &RadioGroup,
    options: &[RadioOption],
) -> impl View<RadioGroup, (), GenetCtx, Element = GenetElement> + use<> {
    let enabled: Rc<Vec<usize>> = Rc::new(
        options
            .iter()
            .enumerate()
            .filter_map(|(index, option)| (!option.disabled).then_some(index))
            .collect(),
    );
    let active = if enabled.contains(&state.selected) {
        Some(state.selected)
    } else {
        enabled.first().copied()
    };
    // One clickable row per option. The per-option closures share one type (one
    // closure definition capturing a `usize`), so the `Vec` is homogeneous.
    let items: Vec<_> = options
        .iter()
        .enumerate()
        .map(|(index, option)| {
            let selected = index == state.selected;
            let disabled = option.disabled;
            let tab_stop = active == Some(index);
            let enabled = enabled.clone();
            let mut row = el::<_, RadioGroup, ()>(
                "div",
                (
                    el::<_, RadioGroup, ()>("span", ()).attr("class", "radio-indicator"),
                    el::<_, RadioGroup, ()>("span", option.label.clone())
                        .attr("class", "radio-label"),
                ),
            )
            .attr("role", "radio")
            .attr("aria-checked", if selected { "true" } else { "false" })
            .attr("aria-disabled", if disabled { "true" } else { "false" })
            .attr("tabindex", if tab_stop { "0" } else { "-1" })
            .attr(
                "class",
                match (selected, disabled) {
                    (true, false) => "radio selected",
                    (false, false) => "radio",
                    (true, true) => "radio selected disabled",
                    (false, true) => "radio disabled",
                },
            );
            for (name, value) in &option.attrs {
                row = row.attr(name.clone(), value.clone());
            }
            let item = on_click(row, move |s: &mut RadioGroup, _| {
                if !disabled {
                    s.selected = index;
                    s.focus_request = None;
                }
            });
            let keyboard = on_key(item, move |state: &mut RadioGroup, event| {
                let Some(position) = enabled.iter().position(|candidate| *candidate == index)
                else {
                    return;
                };
                let count = enabled.len();
                let next = match event.key {
                    Key::Named(NamedKey::ArrowLeft | NamedKey::ArrowUp) => {
                        enabled[(position + count - 1) % count]
                    },
                    Key::Named(NamedKey::ArrowRight | NamedKey::ArrowDown) => {
                        enabled[(position + 1) % count]
                    },
                    // Cambium extensions retained from the original runner behavior.
                    Key::Named(NamedKey::Home) => enabled[0],
                    Key::Named(NamedKey::End) => enabled[count - 1],
                    Key::Named(NamedKey::Space) => index,
                    _ => return,
                };
                state.selected = next;
                state.focus_request = Some(next);
                event.prevent_default();
            })
            .focusable(tab_stop);
            request_focus(keyboard, state.focus_request == Some(index) && !disabled)
        })
        .collect();
    el::<_, RadioGroup, ()>("div", items)
        .attr("role", "radiogroup")
        .attr("aria-label", state.label.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_holds_selection() {
        assert_eq!(RadioGroup::new(2).selected, 2);
        assert_eq!(RadioGroup::default().selected, 0);
        assert_eq!(RadioGroup::default().label, "Options");
        assert_eq!(RadioGroup::default().focus_request, None);
    }

    #[test]
    fn accepts_owned_option_labels() {
        let options = ["Left".to_owned(), "Right".to_owned()];
        let _view = radio_group(&RadioGroup::default(), &options);
    }
}
