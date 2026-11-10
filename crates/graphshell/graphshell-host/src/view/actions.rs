// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::fmt::Write as _;

use chirograph::{ActionIcon, AdvertisedAction};
use pictograph::symbols::Symbol;

use super::escape;

/// Resolve a guest's icon hint through the public symbol catalog. Host-only
/// names and unknown names leave the action available as text.
pub fn guest_action_symbol(icon: Option<&ActionIcon>) -> Option<Symbol> {
    let ActionIcon::Named(name) = icon?;
    let symbol = Symbol::named(name)?;
    match symbol {
        Symbol::Read
        | Symbol::Map
        | Symbol::Inspect
        | Symbol::Trail
        | Symbol::Search
        | Symbol::Tabs
        | Symbol::Fit
        | Symbol::Action => Some(symbol),
    }
}

pub(super) fn render_actions(html: &mut String, actions: &[AdvertisedAction]) {
    if actions.is_empty() {
        return;
    }
    html.push_str("<div class=\"actions\">");
    for action in actions {
        write!(
            html,
            "<button type=\"button\" data-intent=\"{}\" title=\"{}\">",
            escape(&action.intent.0),
            escape(&action.explanation),
        )
        .unwrap();
        if let Some(symbol) = guest_action_symbol(action.icon.as_ref()) {
            write!(
                html,
                r#"<svg aria-hidden="true" focusable="false" width="1em" height="1em" viewBox="-32 -32 64 64" style="vertical-align:middle;margin-inline-end:.35em"><path fill="currentColor" d="{}"/></svg>"#,
                symbol.svg_path(12.0),
            )
            .unwrap();
        }
        write!(html, "{}</button>", escape(&action.label)).unwrap();
    }
    html.push_str("</div>");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guest_action_icons_keep_the_label_and_refuse_reserved_names() {
        let mut action: AdvertisedAction = serde_json::from_str(
            r#"{"intent":"fixture.open","label":"Open <card>","explanation":"","payload_schema":"fixture.open/v1","effect":"Curation"}"#,
        )
        .unwrap();
        let render = |action: &AdvertisedAction| {
            let mut html = String::new();
            render_actions(&mut html, std::slice::from_ref(action));
            html
        };
        let text = render(&action);
        action.icon = Some(ActionIcon::Named("read".into()));
        let decorated = render(&action);
        assert!(decorated.contains(r#"<svg aria-hidden="true""#));
        assert!(decorated.contains("</svg>Open &lt;card&gt;</button>"));
        for name in [
            "lock",
            "shield",
            "verified",
            "permission",
            "window-close",
            "warning",
            "host-seal",
            "unknown",
            "READ",
        ] {
            action.icon = Some(ActionIcon::Named(name.into()));
            assert_eq!(render(&action), text, "{name}");
        }
    }
}
