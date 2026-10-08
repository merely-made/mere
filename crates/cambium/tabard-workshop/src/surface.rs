// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use cambium::{
    AnyView, FileEvent, FileFilter, GenetCtx, GenetElement, PointerClick, button, custom_leaf, el,
    highlighted_code, lens, map_message_result, open_file, slider, text_field_typed,
    textarea_typed, title_bar,
};
use tabard::theme::registry::Harmony;
use tabard::theme::seed::{derive_from_def_for_mode, harmonized_seeds};
use tinct::Srgb;

use crate::{ExportFormat, READER_LEAF_KEY, STYLESHEET_LEAF_KEY, SeedRole, WorkshopState};

pub type WorkshopView = Box<dyn AnyView<WorkshopState, (), GenetCtx, GenetElement>>;

/// The workshop frame stays neutral while each specimen receives the draft's
/// derived appearance. This is the frame sheet; hosts mount
/// [`crate::workshop_stylesheet`] to include the shared component sheets.
pub const WORKSHOP_CSS: &str = include_str!("workshop.css");

pub const WORKSHOP_CODE_SAMPLE: &str = "// A little colour, everywhere\nfn garden() {\n    let season = \"spring\";\n    grow(season, 24);\n}";

pub fn workshop_view(state: &WorkshopState) -> WorkshopView {
    workshop_view_with_captions(state, Box::new(el("div", ())))
}

/// Mount the same authoring surface with host-supplied window controls.
/// The host keeps command and close-policy ownership; the workshop supplies
/// the title, ornament and ordinary authoring actions.
pub fn workshop_view_with_captions(state: &WorkshopState, captions: WorkshopView) -> WorkshopView {
    Box::new(
        el(
            "main",
            vec![
                header(state, captions),
                close_confirmation(state),
                body(state),
                footer(state),
            ],
        )
        .attr("class", "tabard-workshop")
        .attr("data-surface", "tabard.workshop.v1"),
    )
}

fn close_confirmation(state: &WorkshopState) -> WorkshopView {
    if !state.close_requested() {
        return Box::new(el("div", ()).attr("hidden", ""));
    }
    Box::new(
        el(
            "section",
            (
                el(
                    "p",
                    format!(
                        "Save changes to {} before closing?",
                        state.draft_theme().name
                    ),
                ),
                button(
                    "Save and close",
                    |s: &mut WorkshopState, _: PointerClick| s.save_and_close(),
                )
                .attr("data-action", "save-close"),
                button(
                    "Close without saving",
                    |s: &mut WorkshopState, _: PointerClick| s.discard_and_close(),
                )
                .attr("data-action", "discard-close"),
                button("Keep editing", |s: &mut WorkshopState, _: PointerClick| {
                    s.cancel_close()
                })
                .attr("data-action", "cancel-close"),
            ),
        )
        .attr("class", "confirmation close-confirmation")
        .attr("role", "group")
        .attr("aria-label", "Unsaved theme close confirmation"),
    )
}

fn header(state: &WorkshopState, captions: WorkshopView) -> WorkshopView {
    let save_text = if state.is_dirty() {
        "Unsaved theme"
    } else {
        "Saved theme"
    };
    title_bar(
        Box::new(
            el("div", "T")
                .attr("class", "tabard-mark")
                .attr("aria-hidden", "true"),
        ),
        Box::new(
            el("div", (el("h1", "Tabard"), el("p", "Appearance workshop")))
                .attr("class", "brand-title"),
        ),
        Box::new(
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
        captions,
    )
}

fn body(state: &WorkshopState) -> WorkshopView {
    Box::new(el("div", vec![editor(state), previews(state)]).attr("class", "workshop-body"))
}

fn text_control(
    key: &'static str,
    label: &'static str,
    multiline: bool,
    invalid: bool,
) -> WorkshopView {
    Box::new(lens(
        move |input: &mut cambium::TextInput| {
            let field = if multiline {
                textarea_typed(input)
            } else {
                text_field_typed(input)
            };
            field
                .attr("role", "textbox")
                .attr("aria-label", label)
                .attr("data-field", key)
                .attr("aria-invalid", if invalid { "true" } else { "false" })
        },
        move |s: &mut WorkshopState| s.text_field_mut(key).expect("registered workshop field"),
    ))
}

fn export_controls(state: &WorkshopState) -> WorkshopView {
    let formats: Vec<WorkshopView> = ExportFormat::ALL
        .into_iter()
        .map(|format| {
            Box::new(
                button(
                    format.label(),
                    move |s: &mut WorkshopState, _: PointerClick| s.set_export_format(format),
                )
                .attr("data-export-format", format.as_key())
                .attr(
                    "aria-pressed",
                    if state.export_format() == format {
                        "true"
                    } else {
                        "false"
                    },
                ),
            ) as WorkshopView
        })
        .collect();
    let mut children: Vec<WorkshopView> = vec![
        Box::new(el("h3", "Take your theme with you")),
        Box::new(
            el("div", formats)
                .attr("class", "export-formats")
                .attr("role", "group")
                .attr("aria-label", "Export format"),
        ),
        Box::new(
            el(
                "p",
                match state.export_format() {
                    ExportFormat::ThemeJson => {
                        "Complete editable theme, including harmony and mode stylesheets."
                    },
                    ExportFormat::Css => {
                        "CSS color variables for the selected standard preview mode."
                    },
                    ExportFormat::Dtcg => "Design tokens for the selected standard preview mode.",
                },
            )
            .attr("class", "control-note"),
        ),
        Box::new(
            button("Export…", |s: &mut WorkshopState, _: PointerClick| {
                s.request_export()
            })
            .attr("data-action", "export")
            .attr("class", "primary-button"),
        ),
    ];
    if let Some(path) = state.replacement_path() {
        children.push(Box::new(
            el(
                "div",
                (
                    el(
                        "p",
                        format!("{} already exists. Replace that file?", path.display()),
                    ),
                    button("Replace file", |s: &mut WorkshopState, _: PointerClick| {
                        s.replace_export()
                    })
                    .attr("data-action", "replace-export"),
                    button(
                        "Keep existing file",
                        |s: &mut WorkshopState, _: PointerClick| s.cancel_export(),
                    )
                    .attr("data-action", "cancel-export"),
                ),
            )
            .attr("class", "confirmation")
            .attr("role", "group")
            .attr("aria-label", "Replace exported file confirmation"),
        ));
    }
    Box::new(el("section", children).attr("class", "control-group export-controls"))
}

fn stylesheet_panel(state: &WorkshopState) -> WorkshopView {
    let preview = state.stylesheet_preview();
    let name = preview.borrow().accessible_name().to_owned();
    let diagnostics = preview.borrow().diagnostics();
    let mut children: Vec<WorkshopView> = vec![
        Box::new(el("h3", "Application stylesheet")),
        Box::new(
            el(
                "p",
                "An isolated application document using the exact selected stylesheet.",
            )
            .attr("class", "control-note"),
        ),
        Box::new(
            custom_leaf(STYLESHEET_LEAF_KEY, 550, 330)
                .attr("style", "display:block;width:100%;height:330px;")
                .attr("class", "application-canvas")
                .attr("data-view-kind", "application-stylesheet")
                .attr("role", "img")
                .attr("aria-label", name),
        ),
        Box::new(
            button(
                "Use preview as default",
                |s: &mut WorkshopState, _: PointerClick| s.use_preview_as_default(),
            )
            .attr("data-action", "default-mode"),
        ),
        Box::new(
            button(
                if state.advanced_open {
                    "Hide stylesheet editor"
                } else {
                    "Edit this mode's stylesheet"
                },
                |s: &mut WorkshopState, _: PointerClick| s.advanced_open = !s.advanced_open,
            )
            .attr("data-action", "toggle-stylesheet")
            .attr(
                "aria-expanded",
                if state.advanced_open { "true" } else { "false" },
            ),
        ),
    ];
    if state.advanced_open {
        children.push(Box::new(el("p", "CSS applies to the application document. Try body, .toolbar, .address-field, button, h1, p, a or .token-keyword. Empty CSS restores this mode's derived colors. Save includes applied and staged CSS.").attr("class", "control-note")));
        children.push(text_control(
            "mode-sheet",
            "Mode stylesheet CSS",
            true,
            false,
        ));
        children.push(Box::new(
            el(
                "div",
                (
                    button(
                        "Apply to preview",
                        |s: &mut WorkshopState, _: PointerClick| s.apply_stylesheet(),
                    )
                    .attr("data-action", "apply-stylesheet"),
                    button(
                        "Clear to derived",
                        |s: &mut WorkshopState, _: PointerClick| s.clear_stylesheet(),
                    )
                    .attr("data-action", "clear-stylesheet"),
                ),
            )
            .attr("class", "stylesheet-actions"),
        ));
    }
    for diagnostic in diagnostics {
        children.push(Box::new(
            el("p", diagnostic)
                .attr("class", "field-error")
                .attr("role", "status"),
        ));
    }
    Box::new(
        el("section", children)
            .attr("class", "stylesheet-panel")
            .attr("aria-label", "Application stylesheet preview"),
    )
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
                        text_control(
                            "seed-hex",
                            "Seed hex color",
                            false,
                            state.hex_error.is_some(),
                        ),
                    ),
                )
                .attr("class", "seed-value"),
            ),
        )
        .attr("class", "control-group"),
    ));
    controls.push(Box::new(
        button("Apply color", |s: &mut WorkshopState, _: PointerClick| {
            s.apply_hex();
        })
        .attr("data-action", "apply-hex"),
    ));
    if let Some(error) = &state.hex_error {
        controls.push(Box::new(
            el("p", error.clone())
                .attr("class", "field-error")
                .attr("role", "alert"),
        ));
    }
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
    controls.push(Box::new(open_file(
        button(
            "Import theme…",
            |s: &mut WorkshopState, _: PointerClick| s.request_import(),
        )
        .attr("data-action", "import"),
        state.import_requested(),
        FileFilter::extensions(["json"]),
        |s: &mut WorkshopState, event: FileEvent| s.accept_import(event),
    )));
    controls.push(Box::new(
        button(
            "Delete theme…",
            |s: &mut WorkshopState, _: PointerClick| s.request_delete(),
        )
        .attr("data-action", "delete"),
    ));
    if state.delete_requested() {
        controls.push(Box::new(
            el(
                "div",
                (
                    el(
                        "p",
                        format!("Delete {} from your library?", state.draft_theme().name),
                    ),
                    button(
                        "Delete permanently",
                        |s: &mut WorkshopState, _: PointerClick| s.confirm_delete(),
                    )
                    .attr("data-action", "confirm-delete"),
                    button("Keep theme", |s: &mut WorkshopState, _: PointerClick| {
                        s.cancel_delete()
                    })
                    .attr("data-action", "cancel-delete"),
                ),
            )
            .attr("class", "confirmation")
            .attr("role", "group")
            .attr("aria-label", "Delete theme confirmation"),
        ));
    }
    controls.push(export_controls(state));
    Box::new(
        el("aside", controls)
            .attr("class", "theme-editor")
            .attr("aria-label", "Theme editor"),
    )
}

fn previews(state: &WorkshopState) -> WorkshopView {
    let mode_buttons: Vec<WorkshopView> = state
        .available_modes()
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
        panels.push(Box::new(el("p", "The application stylesheet preview renders this mode's authored CSS. Reader, syntax and graph specimens show the seed-derived appearance.").attr("class", "preview-notice").attr("role", "status")));
    }
    panels.push(specimen_grid(state));
    panels.push(palette_strip(state));
    panels.push(stylesheet_panel(state));
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
    let reader = state.reader_preview();
    let reader_name = reader.borrow().accessible_name().to_owned();
    let syntax = state.preview_syntax();
    let reader_body = Box::new(
        el(
            "article",
            custom_leaf(READER_LEAF_KEY, 280, 215)
                .attr("style", "display:block;width:100%;height:215px;")
                .attr("class", "reader-canvas")
                .attr("data-view-kind", "shared-reader")
                .attr("role", "img")
                .attr("aria-label", reader_name),
        )
        .attr("id", "reader-specimen")
        .attr("class", "specimen reader-specimen")
        .attr(
            "style",
            format!(
                "background: {}; color: {};",
                css_color(syntax.surface),
                css_color(syntax.emphasis)
            ),
        ),
    ) as WorkshopView;
    let syntax_body = Box::new(
        el(
            "div",
            (
                el("div", "garden.rs")
                    .attr("class", "code-filename")
                    .attr("style", format!("color: {};", css_color(syntax.comment))),
                highlighted_code::<WorkshopState, ()>(WORKSHOP_CODE_SAMPLE, "rust", &syntax)
                    .attr("id", "syntax-code")
                    .attr("class", "syntax-highlight syntax-code")
                    .attr("aria-label", "Read-only Rust syntax preview"),
            ),
        )
        .attr("id", "syntax-specimen")
        .attr("class", "specimen syntax-specimen")
        .attr(
            "style",
            format!(
                "background: {}; color: {};",
                css_color(syntax.surface),
                css_color(syntax.emphasis)
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
                    state.graph.view(|s: &mut WorkshopState| &mut s.graph),
                )
                .attr("class", "graph-component"),
                el("p", "Select a node to inspect its emphasis.").attr("class", "graph-caption"),
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
