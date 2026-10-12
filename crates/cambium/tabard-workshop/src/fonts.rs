// Copyright 2026 Mark Alan Boykin
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
        }
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
                ),
            )
            .attr("class", "tabard-font-role"),
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
