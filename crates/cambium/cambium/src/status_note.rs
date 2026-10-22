/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A status note: a short live-region message beside the control it reports on.

use crate::{El, el};

/// A `role="status"` note (an implicit polite live region) carrying `text`.
///
/// The element is a `div` with the `status-note` class (which the shell sheet
/// spaces) followed by `class` when it is non-empty, so a host styles its own
/// notes without replacing the base class. Further attributes chain with
/// [`El::attr`]; to name the region use [`status_note_labelled`].
pub fn status_note<State: 'static, Action: 'static>(
    class: &str,
    text: impl Into<String>,
) -> El<String, State, Action> {
    let classes = if class.is_empty() {
        "status-note".to_owned()
    } else {
        format!("status-note {class}")
    };
    el::<_, State, Action>("div", text.into())
        .attr("role", "status")
        .attr("class", classes)
}

/// [`status_note`] with an accessible name for the live region.
pub fn status_note_labelled<State: 'static, Action: 'static>(
    class: &str,
    label: impl Into<String>,
    text: impl Into<String>,
) -> El<String, State, Action> {
    status_note(class, text).attr("aria-label", label)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DomHandle, GenetAppRunner};
    use genet_scripted_dom::ScriptedDom;
    use layout_dom_api::LayoutDom;
    use std::{cell::RefCell, rc::Rc};

    #[test]
    fn note_is_a_status_region_with_a_composed_class() {
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let runner = GenetAppRunner::<_, _, _, ()>::new(
            dom.clone(),
            |_: &()| status_note_labelled("tools-note", "Overlay availability", "Refused"),
            (),
        );
        let root = runner.root();
        let dom = dom.borrow();
        let attr = |name: &str| {
            dom.attribute(root, &"".into(), &name.into())
                .map(str::to_owned)
        };
        assert_eq!(attr("role").as_deref(), Some("status"));
        assert_eq!(attr("class").as_deref(), Some("status-note tools-note"));
        assert_eq!(attr("aria-label").as_deref(), Some("Overlay availability"));
    }
}
