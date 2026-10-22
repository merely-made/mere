/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! [`checkbox`] / [`toggle`]: a `bool`-backed control flipped on click.
//!
//! The control is a `span` carrying the role, ARIA state and a class; its one
//! `control-mark` child is the check fill or switch thumb. [`SHELL_CSS`]
//! (crate::SHELL_CSS) draws both, so there is no glyph or bracket text for a
//! stylesheet to fight and the label stays a separate node beside the control.

use crate::{El, Focusable, OnClick, PointerClick, clickable, el};

/// The toggle handler for [`checkbox`] / [`toggle`]: flip the bool on click.
fn flip(checked: &mut bool, _: PointerClick) {
    *checked = !*checked;
}

/// The element a checkbox / toggle draws: the box or track, holding its mark.
pub type CheckElement<State, Action> = El<El<(), State, Action>, State, Action>;

/// The concrete view type a checkbox / toggle produces: an `on_click`-wrapped
/// element reflecting the checked state.
pub type Checkbox =
    Focusable<OnClick<CheckElement<bool, ()>, bool, (), fn(&mut bool, PointerClick)>>;

/// Build the element for `kind` (`"checkbox"` / `"toggle"`).
fn check_element<State: 'static, Action: 'static>(
    kind: &'static str,
    checked: bool,
    disabled: bool,
) -> CheckElement<State, Action> {
    let toggle = kind == "toggle";
    let class = match (toggle, checked, disabled) {
        (false, false, false) => "checkbox",
        (false, true, false) => "checkbox checked",
        (false, false, true) => "checkbox disabled",
        (false, true, true) => "checkbox checked disabled",
        (true, false, false) => "toggle",
        (true, true, false) => "toggle checked",
        (true, false, true) => "toggle disabled",
        (true, true, true) => "toggle checked disabled",
    };
    let element = el::<_, State, Action>(
        "span",
        el::<_, State, Action>("span", ()).attr("class", "control-mark"),
    )
    .attr("role", if toggle { "switch" } else { "checkbox" })
    .attr("aria-label", if toggle { "Toggle" } else { "Checkbox" })
    .attr("aria-checked", if checked { "true" } else { "false" })
    .attr("class", class);
    if disabled {
        element.attr("aria-disabled", "true")
    } else {
        element
    }
}

fn build_check(kind: &'static str, checked: bool) -> Checkbox {
    let handler: fn(&mut bool, PointerClick) = flip;
    clickable(check_element(kind, checked, false), handler)
}

/// A reusable checkbox whose state *is* a `bool`. Clicking it toggles the bool;
/// it reflects the state as `role="checkbox"` + `aria-checked` (for the a11y
/// tree) and a `checkbox` / `checkbox checked` class. The shell sheet draws the
/// box and its fill; the element holds no text, so the label is the caller's
/// own sibling node. Composes onto an app's bool field via
/// [`lens`](crate::lens), like [`text_field`](crate::text_field) onto a
/// [`TextInput`](crate::TextInput).
pub fn checkbox(checked: bool) -> Checkbox {
    build_check("checkbox", checked)
}

/// [`checkbox`] with its concrete return type named (for a host storing the
/// runner in a struct field; see
/// [`text_field_typed`](crate::text_field_typed)).
pub fn checkbox_typed(checked: bool) -> Checkbox {
    build_check("checkbox", checked)
}

/// A checkbox that cannot be flipped: `aria-disabled="true"` and a `disabled`
/// class for the dimmed look, with no click handler and no tab stop. `checked`
/// still shows the held value. Generic over the caller's state because it has
/// no handler to bind it to a `bool`.
pub fn checkbox_disabled<State: 'static, Action: 'static>(
    checked: bool,
) -> CheckElement<State, Action> {
    check_element("checkbox", checked, true)
}

/// A toggle switch — behaviourally a [`checkbox`] (a `bool`, flipped on click),
/// distinguished only by a `toggle` class so the shell sheet draws it as a
/// switch.
pub fn toggle(checked: bool) -> Checkbox {
    build_check("toggle", checked)
}
