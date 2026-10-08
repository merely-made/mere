// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use cambium::{
    AnyView, GenetCtx, GenetElement, PointerClick, button, el, lens, map_message_result, slider,
    text_field_typed,
};
use tabard::theme::registry::{Harmony, Mode};
use tabard::theme::seed::{derive_from_def_for_mode, harmonized_seeds};
use tinct::{Srgb, SyntaxRole};

use crate::{SeedRole, WorkshopState};

pub type WorkshopView = Box<dyn AnyView<WorkshopState, (), GenetCtx, GenetElement>>;

/// The workshop frame stays neutral while each specimen receives the draft's
/// derived appearance. Hosts mount this sheet and the shared view as-is.
pub const WORKSHOP_CSS: &str = include_str!("workshop.css");

pub fn workshop_view(state: &WorkshopState) -> WorkshopView {
    Box::new(
        el("main", vec![header(state), body(state), footer(state)])
            .attr("class", "tabard-workshop")
            .attr("data-surface", "tabard.workshop.v1"),
    )
}

fn header(state: &WorkshopState) -> WorkshopView {
    let save_text = if state.is_dirty() {
        "Unsaved theme"
    } else {
        "Saved theme"
    };
    Box::new(
        el(
            "header",
            (
                el(
                    "div",
                    (
                        el("div", "T")
                            .attr("class", "tabard-mark")
                            .attr("aria-hidden", "true"),
                        el("div", (el("h1", "Tabard"), el("p", "Appearance workshop")))
                            .attr("class", "brand-title"),
                    ),
                )
                .attr("class", "brand"),
                el(
                    "div",
                    (
                        el("span", save_text).attr("class", "save-state"),
                        button("Undo", |s: &mut WorkshopState, _: PointerClick| s.undo())
                            .attr("data-action", "undo"),
                        button("Redo", |s: &mut WorkshopState, _: PointerClick| s.redo())
                            .attr("data-action", "redo"),
                        button("Save theme", |s: &mut WorkshopState, _: PointerClick| {
                            s.save()
                        })
                        .attr("data-action", "save")
                        .attr("class", "primary-button"),
                    ),
                )
                .attr("class", "header-actions"),
            ),
        )
        .attr("class", "workshop-header"),
    )
}

fn body(state: &WorkshopState) -> WorkshopView {
    Box::new(el("div", vec![editor(state), previews(state)]).attr("class", "workshop-body"))
}

fn editor(state: &WorkshopState) -> WorkshopView {
    let field = map_message_result(
        lens(
            |name: &mut cambium::TextInput| {
                text_field_typed(name)
                    .attr("role", "textbox")
                    .attr("aria-label", "Theme name")
                    .attr("data-field", "name")
            },
            |s: &mut WorkshopState| &mut s.name,
        ),
        |s: &mut WorkshopState, result| {
            s.sync_name();
            result
        },
    );
    let seed_buttons: Vec<WorkshopView> = SeedRole::ALL
        .into_iter()
        .map(|role| {
            let color = css_color(role.color(state.draft_theme()));
            Box::new(
                button(
                    role.label(),
                    move |s: &mut WorkshopState, _: PointerClick| s.set_seed(role),
                )
                .attr("data-seed", role.label().to_ascii_lowercase())
                .attr(
                    "aria-pressed",
                    if role == state.seed_role() {
                        "true"
                    } else {
                        "false"
                    },
                )
                .attr("class", "seed-button")
                .attr("style", format!("border-left: 7px solid {color};")),
            ) as WorkshopView
        })
        .collect();
    let mut controls: Vec<WorkshopView> = vec![
        Box::new(
            el(
                "div",
                (
                    el("h2", "Your theme"),
                    el("p", "A few seeds. A whole visual language.").attr("class", "muted"),
                ),
            )
            .attr("class", "editor-intro"),
        ),
        Box::new(
            el(
                "label",
                (el("span", "Theme name").attr("class", "field-label"), field),
            )
            .attr("class", "name-control"),
        ),
        Box::new(
            el(
                "div",
                (
                    el("h3", "Seed colours"),
                    el("div", seed_buttons).attr("class", "seed-picker"),
                ),
            )
            .attr("class", "control-group"),
        ),
    ];
    for (i, label) in ["Hue", "Saturation", "Lightness"].into_iter().enumerate() {
        if i == 0 && state.hue_follows_primary() {
            controls.push(Box::new(
                el(
                    "p",
                    "Hue follows Primary in this harmony. Choose Independent to edit it directly.",
                )
                .attr("class", "control-note")
                .attr("id", "harmony-hue-note"),
            ));
            continue;
        }
        let value = state.channels[i].value;
        let display = if i == 0 {
            format!("{:.0}°", value * 360.0)
        } else {
            format!("{:.0}%", value * 100.0)
        };
        let field_key = ["seed-hue", "seed-saturation", "seed-lightness"][i];
        let control = map_message_result(
            lens(
                move |value: &mut cambium::Slider| slider(value),
                move |s: &mut WorkshopState| &mut s.channels[i],
            ),
            |s: &mut WorkshopState, result| {
                s.sync_channels();
                result
            },
        );
        controls.push(Box::new(
            el(
                "div",
                (
                    el(
                        "div",
                        (
                            el("span", label),
                            el("span", display).attr("class", "channel-value"),
                        ),
                    )
                    .attr("class", "channel-label"),
                    el("div", control).attr("data-field", field_key),
                ),
            )
            .attr("class", "channel-control"),
        ));
    }
    controls.push(Box::new(
        el(
            "div",
            (
                el("div", "Authored seed").attr("class", "field-label"),
                el(
                    "div",
                    (
                        el("span", ()).attr("class", "large-swatch").attr(
                            "style",
                            format!(
                                "background: {};",
                                css_color(state.seed_role().color(state.draft_theme()))
                            ),
                        ),
                        el(
                            "span",
                            tinct::color_to_hex(state.seed_role().color(state.draft_theme())),
                        )
                        .attr("class", "hex-value"),
                    ),
                )
                .attr("class", "seed-value"),
            ),
        )
        .attr("class", "control-group"),
    ));
    let harmony = state.draft_theme().harmony;
    let active = match harmony {
        Harmony::Custom => "custom",
        Harmony::Locked {
            secondary_deg,
            tertiary_deg,
        } if secondary_deg == 120.0 && tertiary_deg == 240.0 => "triadic",
        Harmony::Locked {
            secondary_deg,
            tertiary_deg,
        } if secondary_deg == 30.0 && tertiary_deg == -30.0 => "analogous",
        Harmony::Locked {
            secondary_deg,
            tertiary_deg,
        } if secondary_deg == 180.0 && tertiary_deg == 150.0 => "complementary",
        Harmony::Locked { .. } => "lock",
    };
    let harmonies: Vec<WorkshopView> = [
        ("Independent", "custom"),
        ("Triadic", "triadic"),
        ("Analogous", "analogous"),
        ("Complementary", "complementary"),
    ]
    .into_iter()
    .map(|(label, key)| {
        Box::new(
            button(label, move |s: &mut WorkshopState, _: PointerClick| {
                s.set_harmony(key)
            })
            .attr("data-harmony", key)
            .attr("aria-pressed", if active == key { "true" } else { "false" }),
        ) as WorkshopView
    })
    .collect();
    controls.push(Box::new(
        el(
            "div",
            (
                el("h3", "Accent harmony"),
                el("div", harmonies).attr("class", "harmony-picker"),
                el(
                    "p",
                    "Harmony derives the accent hues together. Your original seeds stay available.",
                )
                .attr("class", "control-note"),
            ),
        )
        .attr("class", "control-group"),
    ));
    let themes: Vec<WorkshopView> = state
        .registry()
        .list()
        .into_iter()
        .map(|theme| {
            let id = theme.id.clone();
            Box::new(
                button(
                    theme.name.clone(),
                    move |s: &mut WorkshopState, _: PointerClick| s.select_theme(&id),
                )
                .attr("data-theme-id", theme.id.clone())
                .attr("class", "library-theme"),
            ) as WorkshopView
        })
        .collect();
    controls.push(Box::new(
        el(
            "div",
            (
                el("h3", "Theme library"),
                el("div", themes).attr("class", "library-list"),
            ),
        )
        .attr("class", "control-group"),
    ));
    controls.push(Box::new(
        el(
            "div",
            (
                button("New copy", |s: &mut WorkshopState, _: PointerClick| {
                    s.new_copy()
                })
                .attr("data-action", "new-copy"),
                button(
                    "Discard changes",
                    |s: &mut WorkshopState, _: PointerClick| s.discard(),
                )
                .attr("data-action", "discard"),
                button(
                    "Reopen library",
                    |s: &mut WorkshopState, _: PointerClick| s.reopen(),
                )
                .attr("data-action", "reopen"),
            ),
        )
        .attr("class", "library-actions"),
    ));
    Box::new(
        el("aside", controls)
            .attr("class", "theme-editor")
            .attr("aria-label", "Theme editor"),
    )
}

fn previews(state: &WorkshopState) -> WorkshopView {
    let mode_buttons: Vec<WorkshopView> = [Mode::Light, Mode::Dark, Mode::HcLight, Mode::HcDark]
        .into_iter()
        .map(|mode| {
            let selected = &mode == state.mode();
            let label = mode.label();
            let key = mode.as_key();
            Box::new(
                button(label, move |s: &mut WorkshopState, _: PointerClick| {
                    s.set_mode(mode.clone())
                })
                .attr("data-mode", key)
                .attr("aria-pressed", if selected { "true" } else { "false" }),
            ) as WorkshopView
        })
        .collect();
    let mut panels: Vec<WorkshopView> = vec![Box::new(
        el(
            "div",
            (
                el(
                    "div",
                    (
                        el("h2", "See it in context"),
                        el("p", "One theme, across the things you use.").attr("class", "muted"),
                    ),
                ),
                el("div", mode_buttons)
                    .attr("class", "mode-picker")
                    .attr("role", "group")
                    .attr("aria-label", "Preview mode"),
            ),
        )
        .attr("class", "preview-heading"),
    )];
    if state.draft_theme().mode_sheet(state.mode()).is_some() {
        panels.push(Box::new(el("p", "This mode has an authored stylesheet. These specimens show the derived seed palette; the stylesheet needs its own application preview.").attr("class", "preview-notice").attr("role", "status")));
    }
    panels.push(specimen_grid(state));
    panels.push(palette_strip(state));
    Box::new(
        el("section", panels)
            .attr("class", "preview-area")
            .attr("aria-label", "Theme previews"),
    )
}

fn specimen_grid(state: &WorkshopState) -> WorkshopView {
    let palette = state.preview_palette();
    let tokens = derive_from_def_for_mode(state.draft_theme(), state.mode());
    let chrome = tokens.chrome;
    let chrome_body = Box::new(
        el(
            "div",
            (
                el(
                    "div",
                    (
                        el("span", "●  ●  ●").attr("class", "window-dots"),
                        el("span", "A small window onto the world"),
                    ),
                )
                .attr("class", "chrome-bar")
                .attr(
                    "style",
                    format!(
                        "background: {}; color: {};",
                        css_color(chrome.toolbar_bg),
                        css_color(chrome.body_text)
                    ),
                ),
                el(
                    "div",
                    (
                        el("div", "gemini://garden.example/")
                            .attr("class", "address-field")
                            .attr(
                                "style",
                                format!(
                                    "background: {}; color: {};",
                                    css_color(chrome.field_bg),
                                    css_color(chrome.field_text)
                                ),
                            ),
                        el("p", "A place to begin").attr("class", "specimen-title"),
                        el("p", "Chrome, controls, and the space around your content.")
                            .attr("class", "specimen-copy"),
                        el("button", "Explore")
                            .attr("id", "chrome-accent")
                            .attr("tabindex", "-1")
                            .attr("class", "specimen-action")
                            .attr(
                                "style",
                                format!(
                                    "background: {}; color: {};",
                                    css_color(palette.primary),
                                    css_color(palette.on_primary)
                                ),
                            ),
                    ),
                )
                .attr("class", "chrome-content"),
            ),
        )
        .attr("id", "chrome-specimen")
        .attr("class", "specimen chrome-specimen")
        .attr(
            "style",
            format!(
                "background: {}; color: {};",
                css_color(chrome.surface_bg),
                css_color(chrome.body_text)
            ),
        ),
    ) as WorkshopView;
    let reader_body = Box::new(el("article", (
        el("span", "FIELD NOTES / 04").attr("class", "specimen-eyebrow").attr("style", format!("color: {};", css_color(palette.text_dim))),
        el("h3", "The garden after rain").attr("class", "reader-title").attr("style", format!("color: {};", css_color(palette.text_header))),
        el("p", "Every path begins with a small act of attention. The leaves hold yesterday’s weather; the ground makes room for what comes next.").attr("class", "reader-copy"),
        el("div", (el("span", "Continue reading →").attr("class", "reader-link").attr("style", format!("color: {};", css_color(state.preview_syntax().link))), el("span", "  A path already visited").attr("style", format!("color: {};", css_color(state.preview_syntax().verbatim))))),
    )).attr("id", "reader-specimen").attr("class", "specimen reader-specimen").attr("style", format!("background: {}; color: {};", css_color(palette.bg), css_color(palette.text)))) as WorkshopView;
    let syntax = state.preview_syntax();
    let spans: Vec<WorkshopView> = [
        ("// A little colour, everywhere\n", SyntaxRole::Comment),
        ("fn ", SyntaxRole::Keyword),
        ("garden", SyntaxRole::Function),
        ("() {\n    ", SyntaxRole::Punctuation),
        ("let ", SyntaxRole::Keyword),
        ("season", SyntaxRole::Type),
        (" = ", SyntaxRole::Punctuation),
        ("\"spring\"", SyntaxRole::String),
        (";\n    ", SyntaxRole::Punctuation),
        ("grow", SyntaxRole::Function),
        ("(season, ", SyntaxRole::Punctuation),
        ("24", SyntaxRole::Number),
        (");\n}", SyntaxRole::Punctuation),
    ]
    .into_iter()
    .map(|(text, role)| {
        Box::new(
            el("span", text).attr("style", format!("color: {};", css_color(syntax.role(role)))),
        ) as WorkshopView
    })
    .collect();
    let syntax_body = Box::new(
        el(
            "div",
            (
                el("div", "garden.rs")
                    .attr("class", "code-filename")
                    .attr("style", format!("color: {};", css_color(palette.text_dim))),
                el("pre", spans).attr("class", "syntax-code"),
            ),
        )
        .attr("id", "syntax-specimen")
        .attr("class", "specimen syntax-specimen")
        .attr(
            "style",
            format!(
                "background: {}; color: {};",
                css_color(syntax.surface),
                css_color(palette.text)
            ),
        ),
    ) as WorkshopView;
    let graph_body = Box::new(
        el(
            "div",
            (
                el("div", "Ideas find their neighbours").attr("class", "graph-heading"),
                el(
                    "div",
                    (
                        graph_node(
                            "Notes",
                            "A thought to return to",
                            tokens.graph_node_chrome.workspace_badge_background,
                            tokens.graph_node_chrome.workspace_badge_text,
                            tokens.graph_node_focus_ring,
                        ),
                        el("span", "— relates to →")
                            .attr("class", "graph-edge")
                            .attr("style", format!("color: {};", css_color(palette.text_dim))),
                        graph_node(
                            "Garden",
                            "Collected observations",
                            palette.secondary,
                            palette.on_secondary,
                            tokens.graph_node_selection,
                        ),
                    ),
                )
                .attr("class", "graph-row"),
                el(
                    "div",
                    (
                        el("span", "Selected").attr("class", "graph-tag").attr(
                            "style",
                            format!(
                                "background: {}; color: {};",
                                css_color(tokens.selection_highlight_background),
                                css_color(tokens.selection_highlight_text)
                            ),
                        ),
                        el("span", "  Pinned · linked · remembered").attr("class", "graph-caption"),
                    ),
                ),
            ),
        )
        .attr("id", "graph-specimen")
        .attr("class", "specimen graph-specimen")
        .attr(
            "style",
            format!(
                "background: {}; color: {};",
                css_color(tokens.workbench_panel_background),
                css_color(chrome.body_text)
            ),
        ),
    ) as WorkshopView;
    Box::new(
        el(
            "div",
            vec![
                card("01", "Application chrome", chrome_body),
                card("02", "Reading", reader_body),
                card("03", "Syntax", syntax_body),
                card("04", "Graph & relationships", graph_body),
            ],
        )
        .attr("class", "specimen-grid"),
    )
}

fn graph_node(title: &str, detail: &str, bg: Srgb, fg: Srgb, ring: Srgb) -> WorkshopView {
    Box::new(
        el(
            "div",
            (
                el("strong", title.to_owned()),
                el("span", detail.to_owned()),
            ),
        )
        .attr("class", "graph-node")
        .attr(
            "style",
            format!(
                "background: {}; color: {}; border: 2px solid {};",
                css_color(bg),
                css_color(fg),
                css_color(ring)
            ),
        ),
    )
}

fn card(number: &str, title: &str, content: WorkshopView) -> WorkshopView {
    Box::new(
        el(
            "section",
            (
                el(
                    "div",
                    (
                        el("span", number.to_owned()).attr("class", "specimen-number"),
                        el("h3", title.to_owned()),
                    ),
                )
                .attr("class", "specimen-label"),
                content,
            ),
        )
        .attr("class", "specimen-card"),
    )
}

fn palette_strip(state: &WorkshopState) -> WorkshopView {
    let palette = state.preview_palette();
    let effective = harmonized_seeds(state.draft_theme());
    let swatches: Vec<WorkshopView> = [
        ("Canvas", palette.bg),
        ("Surface", palette.surface),
        ("Text", palette.text),
        ("Primary", effective.primary),
        ("Secondary", effective.secondary),
        ("Tertiary", effective.tertiary),
        ("Success", palette.success),
        ("Danger", palette.danger),
    ]
    .into_iter()
    .map(|(label, color)| {
        Box::new(
            el(
                "div",
                (
                    el("div", ())
                        .attr("class", "palette-swatch")
                        .attr("style", format!("background: {};", css_color(color))),
                    el("span", label),
                    el("span", tinct::color_to_hex(color)).attr("class", "palette-hex"),
                ),
            )
            .attr("class", "palette-item"),
        ) as WorkshopView
    })
    .collect();
    Box::new(
        el(
            "section",
            (
                el("h3", "Derived palette"),
                el("div", swatches).attr("class", "palette-strip"),
            ),
        )
        .attr("class", "palette-section"),
    )
}

fn footer(state: &WorkshopState) -> WorkshopView {
    Box::new(
        el(
            "footer",
            (
                el("span", state.status().to_owned())
                    .attr("id", "workshop-status")
                    .attr("role", "status")
                    .attr("aria-live", "polite"),
                el(
                    "span",
                    if state.library_path().is_some() {
                        "Authored themes are saved to your library."
                    } else {
                        "Themes are kept for this session."
                    },
                )
                .attr("class", "footer-note"),
            ),
        )
        .attr("class", "workshop-footer"),
    )
}

fn css_color(color: Srgb) -> String {
    format!(
        "rgba({}, {}, {}, {:.4})",
        color.r,
        color.g,
        color.b,
        f64::from(color.a) / 255.0
    )
}
