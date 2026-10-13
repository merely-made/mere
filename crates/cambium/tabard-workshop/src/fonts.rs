// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
//! Reusable application font preferences. Applying persists through the host;
//! editing does not alter the theme library or implicitly change the app.
use cambium::{
    AnyView, GenetCtx, GenetElement, PointerClick, TextInput, button, el, lens, text,
    text_field_typed,
};
use std::collections::BTreeMap;
use tabard::typography::{FontFamily, FontPreferences, FontRole};

pub struct FontRoleEditor {
    fields: BTreeMap<FontRole, TextInput>,
    baseline: FontPreferences,
    requested: Option<FontPreferences>,
    pub notice: Option<String>,
    available_families: Vec<String>,
    choosing: Option<FontRole>,
    pub search: TextInput,
}
impl FontRoleEditor {
    pub fn new(preferences: FontPreferences) -> Self {
        let fields = FontRole::ALL
            .into_iter()
            .map(|role| {
                let family = preferences
                    .0
                    .get(&role)
                    .and_then(|stack| stack.first())
                    .map_or("", FontFamily::name);
                (role, TextInput::new(family))
            })
            .collect();
        Self {
            fields,
            baseline: preferences,
            requested: None,
            notice: None,
            available_families: Vec::new(),
            choosing: None,
            search: TextInput::new(""),
        }
    }
    /// Host-local inventory is never serialized into preferences.
    pub fn set_available_families(&mut self, families: Vec<String>) {
        self.available_families = families;
    }
    pub fn field(&self, role: FontRole) -> &TextInput {
        &self.fields[&role]
    }
    pub fn field_mut(&mut self, role: FontRole) -> &mut TextInput {
        self.fields.get_mut(&role).unwrap()
    }
    pub fn take_requested(&mut self) -> Option<FontPreferences> {
        self.requested.take()
    }
    pub fn applied(&mut self, preferences: FontPreferences) {
        self.baseline = preferences;
        self.notice = None;
    }
    pub fn request_apply(&mut self) {
        let mut preferences = self.baseline.clone();
        for role in FontRole::ALL {
            let value = self.field(role).text();
            let previous = self
                .baseline
                .0
                .get(&role)
                .and_then(|stack| stack.first())
                .map_or("", FontFamily::name);
            // Preserve ordered fallback stacks for fields that were not edited.
            if value == previous {
                continue;
            }
            if value.trim().is_empty() {
                preferences.0.remove(&role);
            } else {
                match FontFamily::new(value) {
                    Ok(family) => {
                        preferences.0.insert(role, vec![family]);
                    },
                    Err(error) => {
                        self.notice = Some(error);
                        return;
                    },
                }
            }
        }
        self.requested = Some(preferences);
    }
}
/// Controlled text slots for family entry and local inventory search.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FontRoleField {
    Family(FontRole),
    Search,
}
impl FontRoleEditor {
    pub fn text_field(&self, field: FontRoleField) -> &TextInput {
        match field {
            FontRoleField::Family(role) => self.field(role),
            FontRoleField::Search => &self.search,
        }
    }
    pub fn text_field_mut(&mut self, field: FontRoleField) -> &mut TextInput {
        match field {
            FontRoleField::Family(role) => self.field_mut(role),
            FontRoleField::Search => &mut self.search,
        }
    }
}
/// Native hosts project these slots into their own state; the editor keeps
/// ordinary caret, keyboard and IME ownership through shared text controls.
pub fn focused_font_field(
    dom: &cambium::DomHandle,
    node: genet_scripted_dom::NodeId,
) -> Option<FontRoleField> {
    use layout_dom_api::{LayoutDom, LocalName, Namespace};
    let dom = dom.borrow();
    if dom.attribute(
        node,
        &Namespace::from(""),
        &LocalName::from("data-font-search"),
    ) == Some("true")
    {
        return Some(FontRoleField::Search);
    }
    let key = dom.attribute(
        node,
        &Namespace::from(""),
        &LocalName::from("data-font-role"),
    )?;
    FontRole::ALL
        .into_iter()
        .find(|role| role.key() == key)
        .map(FontRoleField::Family)
}

pub type FontRoleView = Box<dyn AnyView<FontRoleEditor, (), GenetCtx, GenetElement>>;
/// Family entry accepts installed names, bundled names and CSS generics. The
/// host supplies availability and fallback diagnostics outside this draft.
pub fn font_role_view(state: &FontRoleEditor) -> FontRoleView {
    let mut children: Vec<FontRoleView> = vec![
        Box::new(el("h3", text("Font roles"))),
        Box::new(el(
            "p",
            text(
                "Choose a font family for each role. Leave it blank to use the current default. Sizes and spacing stay the same.",
            ),
        )),
    ];
    for role in FontRole::ALL {
        children.push(Box::new(
            el(
                "div",
                (
                    el("span", text(role.label())),
                    lens(
                        move |input: &mut TextInput| {
                            text_field_typed(input)
                                .attr("class", "tabard-font-family-input")
                                .attr("role", "textbox")
                                .attr("aria-label", format!("{} font", role.label()))
                                .attr("data-font-role", role.key())
                        },
                        move |state: &mut FontRoleEditor| state.field_mut(role),
                    ),
                    button(
                        "Default",
                        move |state: &mut FontRoleEditor, _: PointerClick| {
                            *state.field_mut(role) = TextInput::new("");
                        },
                    )
                    .attr("data-font-default", role.key()),
                    button(
                        "Choose…",
                        move |state: &mut FontRoleEditor, _: PointerClick| {
                            state.choosing = Some(role);
                            state.search = TextInput::new("");
                        },
                    )
                    .attr("data-font-choose", role.key()),
                ),
            )
            .attr("class", "tabard-font-role"),
        ));
    }
    if let Some(role) = state.choosing {
        let mut picker: Vec<FontRoleView> = vec![
            Box::new(el(
                "h4",
                text(format!("Choose {} font", role.label().to_lowercase())),
            )),
            Box::new(lens(
                |search: &mut TextInput| {
                    text_field_typed(search)
                        .attr("class", "tabard-font-family-input")
                        .attr("role", "textbox")
                        .attr("aria-label", "Find a font family")
                        .attr("data-font-search", "true")
                },
                |state: &mut FontRoleEditor| &mut state.search,
            )),
        ];
        let query = state.search.text().to_lowercase();
        let matches: Vec<_> = state
            .available_families
            .iter()
            .filter(|name| name.to_lowercase().contains(&query))
            .collect();
        for name in matches.iter().take(30) {
            let family = (*name).clone();
            picker.push(Box::new(
                button(
                    family.clone(),
                    move |state: &mut FontRoleEditor, _: PointerClick| {
                        *state.field_mut(role) = TextInput::new(family.clone());
                        state.choosing = None;
                    },
                )
                .attr("data-font-family", (*name).clone()),
            ));
        }
        picker.push(Box::new(el("p", text(format!("{} available families match. Showing up to 30; type to narrow the list. Availability does not guarantee every glyph.", matches.len())))));
        picker.push(Box::new(button(
            "Close font list",
            |state: &mut FontRoleEditor, _: PointerClick| state.choosing = None,
        )));
        children.push(Box::new(
            el("div", picker).attr("class", "tabard-font-picker"),
        ));
    }
    children.push(Box::new(
        button(
            "Apply font roles",
            |state: &mut FontRoleEditor, _: PointerClick| state.request_apply(),
        )
        .attr("data-action", "apply-font-roles"),
    ));
    if let Some(notice) = &state.notice {
        children.push(Box::new(
            el("p", text(notice.clone())).attr("role", "status"),
        ));
    }
    Box::new(
        el("section", children)
            .attr("class", "tabard-font-roles")
            .attr("aria-label", "Font roles"),
    )
}
pub const FONT_ROLE_CSS: &str = ".tabard-font-role { display:flex;flex-wrap:wrap;align-items:center;gap:8px;margin:8px 0; } .tabard-font-role > span { width:90px; } .tabard-font-family-input { display:block;min-width:160px;min-height:30px;padding:5px 8px;border:1px solid currentColor;background:transparent;color:inherit;font:inherit; }";

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn draft_preserves_unedited_stacks_and_default_is_explicit_apply() {
        let preferences = FontPreferences(BTreeMap::from([(
            FontRole::Reading,
            vec![
                FontFamily::new("Absent").unwrap(),
                FontFamily::new("serif").unwrap(),
            ],
        )]));
        let mut editor = FontRoleEditor::new(preferences.clone());
        assert!(editor.take_requested().is_none());
        editor.request_apply();
        assert_eq!(editor.take_requested(), Some(preferences));
        *editor.field_mut(FontRole::Reading) = TextInput::new("");
        assert!(editor.take_requested().is_none());
        editor.request_apply();
        assert!(editor.take_requested().unwrap().is_empty());
    }
}

#[cfg(test)]
mod picker_tests {
    use super::*;
    use cambium_genet_winit_host::{FocusedTextSlot, Harness, Init, inert_hooks};
    use taproot::Selector;
    #[test]
    fn native_picker_filters_available_families_and_only_stages_the_choice() {
        let mut state = FontRoleEditor::new(Default::default());
        state.set_available_families(vec!["Face A".into(), "Face B".into()]);
        let mut hooks = inert_hooks();
        hooks.focused_text = Box::new(|runner| {
            let node = runner.focus()?;
            let field = focused_font_field(&runner.dom(), node)?;
            Some(FocusedTextSlot {
                node,
                get: Box::new(move |state: &FontRoleEditor| state.text_field(field)),
                get_mut: Box::new(move |state: &mut FontRoleEditor| state.text_field_mut(field)),
            })
        });
        let mut host = Harness::with_hooks(
            Init {
                state,
                logic: font_role_view as fn(&FontRoleEditor) -> FontRoleView,
                sheet: FONT_ROLE_CSS.into(),
                fonts: vec![],
                images: vec![],
            },
            hooks,
        );
        host.layout_at(600.0, 900.0);
        assert!(host.click_on(&Selector::role("button").with_attr("data-font-choose", "reading")));
        assert!(host.click_on(&Selector::role("textbox").with_attr("data-font-search", "true")));
        host.key_injected("Face B");
        assert_eq!(host.state().search.text(), "Face B");
        assert!(!host.click_on(&Selector::role("button").with_attr("data-font-family", "Face A")));
        assert!(host.click_on(&Selector::role("button").with_attr("data-font-family", "Face B")));
        assert_eq!(host.state().field(FontRole::Reading).text(), "Face B");
        host.update(|state| assert!(state.take_requested().is_none()));
        assert!(
            host.click_on(&Selector::role("button").with_attr("data-action", "apply-font-roles"))
        );
        host.update(|state| {
            assert_eq!(
                state.take_requested().unwrap().0[&FontRole::Reading][0].name(),
                "Face B"
            )
        });
    }
}
