// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Native mounting only. The embedding app owns the window/event loop, source
//! admission, current authority, driver pump, target transaction and close.

use crate::workshop_surface::{STYLESHEET, Surface, SurfaceState, surface};
use cambium::{TITLE_BAR_CSS, el};
use cambium_genet_winit_host::{
    CaptionLabels, HostHooks, Init, WindowCommands, WinitHost, inert_hooks,
    platform_caption_controls,
};

pub type NativeLogic = Box<dyn FnMut(&SurfaceState) -> Surface>;
pub type NativeHost = WinitHost<SurfaceState, NativeLogic, Surface>;

pub fn native_init(
    state: SurfaceState,
    commands: &WindowCommands,
) -> Init<SurfaceState, NativeLogic> {
    let commands = commands.clone();
    Init {
        state,
        logic: Box::new(move |state| {
            let captions: Surface = platform_caption_controls(&commands, &CaptionLabels::default());
            Box::new(el("div", vec![captions, surface(state)]))
        }),
        sheet: format!("{TITLE_BAR_CSS}\n{STYLESHEET}"),
        fonts: Vec::new(),
        images: Vec::new(),
    }
}

/// App service hooks drain `SurfaceState::take_actions`, invoke their owning
/// adapter, and publish acknowledged `WorkshopView` updates through the runner.
pub fn native_hooks() -> HostHooks<SurfaceState, NativeLogic, Surface> {
    inert_hooks()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workshop_surface::{SourceChoice, SurfaceAction};
    use cambium_genet_winit_host::Harness;
    use taproot::Selector;

    #[test]
    fn workshop_native_controls_select_versions_and_queue_explicit_run_only() {
        let mut state = SurfaceState::new();
        state.choices = (0..3)
            .map(|i| SourceChoice {
                key: i.to_string(),
                title: format!("Retained source {i}"),
                version: format!("version-{i}"),
                selected: false,
                available: true,
                refusal: None,
            })
            .collect();
        let mut host =
            Harness::with_hooks(native_init(state, &WindowCommands::new()), native_hooks());
        host.layout_at(980., 900.);
        let choose = |key| Selector::role("checkbox").with_attr("data-source", key);
        assert!(host.click_on(&choose("0")));
        assert!(host.click_on(&choose("1")));
        assert_eq!(host.state().selected(), ["0", "1"]);
        host.relayout();
        assert!(host.click_on(&Selector::role("button").with_attr("data-action", "alembic-run")));
        let mut actions = Vec::new();
        host.update(|state| actions = state.take_actions());
        assert_eq!(actions, [SurfaceAction::Run]);
    }

    #[test]
    fn workshop_native_busy_surface_keeps_selection_fixed() {
        let mut state = SurfaceState::new();
        state.busy = true;
        state.choices = vec![SourceChoice {
            key: "private".into(),
            title: "Unavailable source".into(),
            version: "held".into(),
            selected: false,
            available: false,
            refusal: Some("Private storage is not available for this helper.".into()),
        }];
        let mut host =
            Harness::with_hooks(native_init(state, &WindowCommands::new()), native_hooks());
        host.layout_at(980., 900.);
        // Native event dispatch must not turn an unavailable row into input.
        let _ = host.click_on(&Selector::role("checkbox").with_attr("data-source", "private"));
        assert!(host.state().selected().is_empty());
        let mut actions = Vec::new();
        host.update(|state| actions = state.take_actions());
        assert!(actions.is_empty());
    }

    #[test]
    fn workshop_native_retained_text_is_available_to_accessibility() {
        let mut state = SurfaceState::new();
        let text = "The café note cites version 2 of its source.";
        state.reading = Some(crate::workshop_surface::RetainedReading {
            title: "Field notes".into(),
            version: "v2".into(),
            text: text.into(),
        });
        let mut host =
            Harness::with_hooks(native_init(state, &WindowCommands::new()), native_hooks());
        host.layout_at(980., 900.);
        assert!(
            host.a11y_projection()
                .nodes()
                .iter()
                .any(|node| node.name.as_deref() == Some(text)),
            "a person must be able to read the exact retained bytes through native accessibility"
        );
    }
}
