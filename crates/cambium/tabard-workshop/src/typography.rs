// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Retained font controls over Tabard's portable preferences.
use crate::{WorkshopState, WorkshopView};
use cambium::{PointerClick, TextInput, button, el};
use tabard::workshop::Edit;
use tabard::{FontRole, ThemeFontStyle};

pub(crate) struct FontControls {
    pub role: FontRole,
    pub family: TextInput,
    pub size: TextInput,
    pub height: TextInput,
    pub error: Option<String>,
    pub installed: Option<Vec<String>>,
}
impl Default for FontControls {
    fn default() -> Self {
        Self {
            role: FontRole::Interface,
            family: TextInput::new(""),
            size: TextInput::new(""),
            height: TextInput::new(""),
            error: None,
            installed: None,
        }
    }
}
impl FontControls {
    pub fn refresh(&mut self, preferences: Option<&ThemeFontStyle>) {
        self.family = TextInput::new(
            preferences
                .and_then(|p| p.family.clone())
                .unwrap_or_default(),
        );
        self.size = TextInput::new(
            preferences
                .and_then(|p| p.size_px)
                .map(|v| v.to_string())
                .unwrap_or_default(),
        );
        self.height = TextInput::new(
            preferences
                .and_then(|p| p.line_height)
                .map(|v| v.to_string())
                .unwrap_or_default(),
        );
        self.error = None;
    }
    fn parse(&self) -> Result<ThemeFontStyle, String> {
        fn number(text: &str, label: &str) -> Result<Option<f32>, String> {
            if text.trim().is_empty() {
                Ok(None)
            } else {
                text.trim()
                    .parse()
                    .map(Some)
                    .map_err(|_| format!("{label} must be a number, or empty to inherit."))
            }
        }
        let p = ThemeFontStyle {
            family: (!self.family.text().trim().is_empty())
                .then(|| self.family.text().trim().to_owned()),
            size_px: number(self.size.text(), "Font size")?,
            line_height: number(self.height.text(), "Line height")?,
        };
        p.validate()?;
        Ok(p)
    }
    pub fn pending(&self, saved: Option<&ThemeFontStyle>) -> bool {
        self.parse()
            .map_or(true, |p| &p != saved.unwrap_or(&ThemeFontStyle::default()))
    }
}
impl WorkshopState {
    /// A host-supplied catalogue from its existing discovery owner. `None`
    /// means availability is unknown, rather than every family being missing.
    pub fn set_installed_font_families(&mut self, mut names: Vec<String>) {
        names.sort_by_key(|n| n.to_ascii_lowercase());
        names.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
        self.fonts.installed = Some(names);
    }
    pub fn installed_font_families(&self) -> Option<&[String]> {
        self.fonts.installed.as_deref()
    }
    pub fn font_role(&self) -> FontRole {
        self.fonts.role
    }
    pub fn set_font_role(&mut self, role: FontRole) {
        if !self.apply_fonts() {
            return;
        }
        self.fonts.role = role;
        self.fonts
            .refresh(self.draft.theme().typography.0.get(&role));
    }
    pub fn apply_fonts(&mut self) -> bool {
        if !self
            .fonts
            .pending(self.draft_theme().typography.0.get(&self.fonts.role))
        {
            self.fonts.error = None;
            return true;
        }
        let preferences = match self.fonts.parse() {
            Ok(p) => p,
            Err(e) => {
                self.fonts.error = Some(e);
                self.status = "Fix the font preferences before continuing.".into();
                return false;
            },
        };
        let mut typography = self.draft_theme().typography.clone();
        if let Err(e) = typography.set(self.fonts.role, preferences) {
            self.fonts.error = Some(e);
            return false;
        }
        if self.draft.edit(Edit::Typography(typography)) {
            self.status = "Font preferences applied to the previews.".into();
        }
        self.fonts
            .refresh(self.draft.theme().typography.0.get(&self.fonts.role));
        true
    }
    pub fn reset_font_role(&mut self) {
        let mut typography = self.draft_theme().typography.clone();
        typography.0.remove(&self.fonts.role);
        self.draft.edit(Edit::Typography(typography));
        self.fonts
            .refresh(self.draft.theme().typography.0.get(&self.fonts.role));
        self.status = "This font role uses the application's defaults.".into();
    }
    pub fn font_diagnostics(&self) -> Vec<String> {
        self.fonts
            .installed
            .as_ref()
            .map(|names| self.draft_theme().typography.missing_families(names))
            .unwrap_or_default()
    }
}

pub(crate) fn controls(state: &WorkshopState) -> WorkshopView {
    let mut children: Vec<WorkshopView> = vec![
        Box::new(el("h3", "Typography")),
        Box::new(el("p", "Choose a role. Empty fields keep the application's defaults. Fonts are resolved on this device; explicit stylesheet rules can override these preferences.").attr("class", "control-note")),
    ];
    let roles: Vec<WorkshopView> = FontRole::ALL
        .into_iter()
        .map(|role| {
            Box::new(
                button(
                    role.label(),
                    move |s: &mut WorkshopState, _: PointerClick| s.set_font_role(role),
                )
                .attr("data-font-role", role.key())
                .attr("aria-pressed", (state.font_role() == role).to_string()),
            ) as WorkshopView
        })
        .collect();
    children.push(Box::new(el("div", roles).attr("class", "font-role-picker")));
    for (key, label) in [
        ("font-family", "Font family"),
        ("font-size", "Font size (CSS pixels)"),
        ("font-line-height", "Line height (multiplier)"),
    ] {
        children.push(Box::new(el("label", label).attr("class", "field-label")));
        children.push(crate::surface::text_control(
            key,
            label,
            false,
            state.fonts.error.is_some(),
        ));
    }
    let generic: Vec<WorkshopView> = ["system-ui", "sans-serif", "serif", "monospace"]
        .into_iter()
        .map(|family| {
            Box::new(
                button(family, move |s: &mut WorkshopState, _: PointerClick| {
                    s.fonts.family = TextInput::new(family);
                    s.apply_fonts();
                })
                .attr("data-font-family", family)
                .attr("class", format!("font-family-{family}")),
            ) as WorkshopView
        })
        .collect();
    children.push(Box::new(
        el("div", generic).attr("class", "font-family-picker"),
    ));
    if let Some(installed) = &state.fonts.installed {
        let query = state.fonts.family.text().to_ascii_lowercase();
        let names: Vec<WorkshopView> = installed
            .iter()
            .filter(|name| {
                !name.starts_with('.')
                    && (query.is_empty() || name.to_ascii_lowercase().contains(&query))
            })
            .take(8)
            .map(|name| {
                let family = name.clone();
                Box::new(
                    button(
                        name.clone(),
                        move |s: &mut WorkshopState, _: PointerClick| {
                            s.fonts.family = TextInput::new(family.clone());
                            s.apply_fonts();
                        },
                    )
                    .attr("data-installed-font", name.clone()),
                ) as WorkshopView
            })
            .collect();
        children.push(Box::new(
            el(
                "p",
                "Installed families: type a name to filter these suggestions.",
            )
            .attr("class", "control-note"),
        ));
        children.push(Box::new(
            el("div", names).attr("class", "font-family-picker"),
        ));
    }
    children.push(Box::new(
        el(
            "div",
            (
                button(
                    "Apply font preferences",
                    |s: &mut WorkshopState, _: PointerClick| {
                        s.apply_fonts();
                    },
                )
                .attr("data-action", "apply-fonts"),
                button(
                    "Use app defaults for this role",
                    |s: &mut WorkshopState, _: PointerClick| s.reset_font_role(),
                )
                .attr("data-action", "reset-font-role"),
            ),
        )
        .attr("class", "font-family-picker"),
    ));
    for diagnostic in state
        .font_diagnostics()
        .into_iter()
        .chain(state.fonts.error.clone())
    {
        children.push(Box::new(
            el("p", diagnostic).attr("class", "field-error").attr(
                "role",
                if state.fonts.error.is_some() {
                    "alert"
                } else {
                    "status"
                },
            ),
        ));
    }
    Box::new(
        el("section", children)
            .attr("class", "control-group typography-controls")
            .attr("role", "group")
            .attr(
                "aria-label",
                format!("{} font preferences", state.font_role().label()),
            ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn role_navigation_keeps_imported_empty_role_data_unchanged() {
        let mut state = WorkshopState::in_memory();
        let mut definition = state.draft_theme().clone();
        definition
            .typography
            .0
            .insert(FontRole::Interface, ThemeFontStyle::default());
        state.import_theme_json(&tabard::portable::theme_json(&definition).unwrap());
        let before = state.draft_theme().clone();
        state.set_font_role(FontRole::Reading);
        assert_eq!(state.draft_theme(), &before);
        assert_eq!(state.font_role(), FontRole::Reading);
    }
}
