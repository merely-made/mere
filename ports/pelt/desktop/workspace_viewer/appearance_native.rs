// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Native acceptance follows the ordinary Theme → Edit themes → saved choice
//! controls. Mesquite drives the actual shared editor; the existing workspace
//! compositor captures the application after explicit activation.

use super::*;
use tabard::theme::choice::ThemeChoice;

pub(super) struct NativeAppearanceProof {
    stage: u8,
    pub(super) expected: Option<ThemeChoice>,
    artifact: PathBuf,
    baseline: Option<(
        TileId,
        PeltSessionIdentity,
        String,
        bool,
        bool,
        usize,
        WorkspaceRect,
    )>,
    started: Instant,
    initial_choice: Option<ThemeChoice>,
    expected_initial: Option<ThemeChoice>,
}

impl NativeAppearanceProof {
    pub(super) fn from_env() -> Result<Option<Self>, String> {
        if std::env::var_os("PELT_APPEARANCE_SCENARIO").is_none() {
            return Ok(None);
        }
        let artifact = std::env::var_os("PELT_APPEARANCE_BROWSER_CAPTURE")
            .ok_or("PELT_APPEARANCE_SCENARIO requires PELT_APPEARANCE_BROWSER_CAPTURE")?;
        Ok(Some(Self {
            stage: 0,
            expected: None,
            artifact: artifact.into(),
            baseline: None,
            started: Instant::now(),
            initial_choice: None,
            expected_initial: std::env::var("PELT_APPEARANCE_EXPECT_CHOICE")
                .ok()
                .map(|value| {
                    ThemeChoice::parse(&value).ok_or("invalid PELT_APPEARANCE_EXPECT_CHOICE")
                })
                .transpose()?,
        }))
    }
    pub(super) fn editor_closed(&mut self, choice: ThemeChoice) {
        self.expected = Some(choice);
        self.stage = 3;
    }

    pub(super) fn needs_browser_frame(&self) -> bool {
        matches!(self.stage, 0 | 1 | 3 | 4 | 5)
    }
}

impl WorkspaceApp {
    pub(in crate::workspace_viewer) fn drive_native_appearance(&mut self) -> Result<(), String> {
        let Some(mut proof) = self.appearance_native.take() else {
            return Ok(());
        };
        let result = self.drive_native_appearance_step(&mut proof);
        self.appearance_native = Some(proof);
        result
    }

    pub(in crate::workspace_viewer) fn click_appearance_control(
        &mut self,
        action: &str,
    ) -> Result<(), String> {
        let (width, height) = self.logical_size();
        let frame = self.frisket.frame(width, height)?;
        let target = self
            .frisket
            .chrome_rect(action)
            .ok_or_else(|| format!("missing {action} control"))?;
        let drawer = frame
            .appearance_rect
            .ok_or("missing appearance scrollport")?;
        if target.y < drawer.y || target.y + target.height > drawer.y + drawer.height {
            self.pointer_move(drawer.x + drawer.width / 2.0, drawer.y + 20.0);
            let dy = if target.y < drawer.y {
                target.y - drawer.y - 16.0
            } else {
                target.y + target.height - drawer.y - drawer.height + 20.0
            };
            if !self.scroll_primary_at_cursor(0.0, dy) {
                return Err(format!("appearance wheel could not reveal {action}"));
            }
            self.frisket.frame(width, height)?;
        }
        self.click_chrome_physical(action)
    }

    fn drive_native_appearance_step(
        &mut self,
        proof: &mut NativeAppearanceProof,
    ) -> Result<(), String> {
        if proof.stage == 6 {
            return Ok(());
        }
        if proof.started.elapsed() > Duration::from_secs(120) {
            return Err(format!(
                "Pelt appearance workflow timed out at stage {}",
                proof.stage
            ));
        }
        match proof.stage {
            0 => {
                if self.redraws == 0 {
                    return Ok(());
                }
                let current = self.appearance.store().choice();
                if let Some(expected) = &proof.expected_initial {
                    if current != expected {
                        return Err(
                            "fresh Pelt process did not restore the exact saved application choice"
                                .into(),
                        );
                    }
                }
                proof.initial_choice = Some(current.clone());
                let tile = self
                    .workspace
                    .focused_tile()
                    .ok_or("appearance workflow needs a focused tile")?;
                let controller = self
                    .workspace
                    .controller(tile)
                    .ok_or("appearance workflow needs a held document")?;
                let rect = self
                    .workspace
                    .content_rect(tile)
                    .ok_or("appearance workflow needs laid-out content")?;
                proof.baseline = Some((
                    tile,
                    controller.session_identity(),
                    controller.address().into(),
                    controller.can_go_back(),
                    controller.can_go_forward(),
                    self.workspace.tree().tiles().len(),
                    rect,
                ));
                self.click_chrome_physical("appearance")?;
                proof.stage = 1;
                self.request_redraw();
            },
            1 => {
                self.click_appearance_control("edit-themes")?;
                proof.stage = 2;
                self.request_redraw();
            },
            2 => {},
            3 => {
                if Some(self.appearance.store().choice()) != proof.initial_choice.as_ref() {
                    return Err(
                        "workshop preview implicitly changed Pelt's application choice".into(),
                    );
                }
                let choice = proof
                    .expected
                    .as_ref()
                    .ok_or("editor did not produce a saved choice")?;
                let index = self
                    .theme_catalog
                    .user_themes()
                    .iter()
                    .position(|theme| theme.id == choice.theme_id)
                    .ok_or("saved editor theme is absent from the shared library")?;
                self.click_appearance_control(&format!("appearance-saved-{index}"))?;
                proof.stage = 4;
                self.request_redraw();
            },
            4 => {
                let expected = proof.expected.as_ref().unwrap();
                let mode = expected
                    .theme_mode
                    .as_ref()
                    .ok_or("saved editor choice has no explicit mode")?;
                let index = self
                    .theme_catalog
                    .modes(self.appearance.store().choice())
                    .iter()
                    .position(|candidate| candidate == mode)
                    .ok_or("saved editor mode is not selectable")?;
                self.click_appearance_control(&format!("appearance-mode-{index}"))?;
                proof.stage = 5;
                self.request_redraw();
            },
            5 => {
                if Some(self.appearance.store().choice()) != proof.expected.as_ref() {
                    return Err(
                        "saved appearance controls did not persist the exact definition/mode"
                            .into(),
                    );
                }
                let (tile, identity, address, back, forward, count, rect) =
                    proof.baseline.as_ref().unwrap();
                let controller = self
                    .workspace
                    .controller(*tile)
                    .ok_or("appearance activation dropped the controller")?;
                if self.workspace.focused_tile() != Some(*tile)
                    || controller.session_identity() != *identity
                    || controller.address() != address
                    || controller.can_go_back() != *back
                    || controller.can_go_forward() != *forward
                    || self.workspace.tree().tiles().len() != *count
                    || self.workspace.content_rect(*tile) != Some(*rect)
                {
                    return Err("appearance activation changed Pelt's held controller, history, focus or content aperture".into());
                }
                self.config.workspace_receipt = Some(WorkspaceReceipt::Appearance);
                self.config.artifact = Some(proof.artifact.clone());
                self.pending_workspace_assertion = Some("shared workshop native controls saved a definition; ordinary Pelt appearance controls applied and persisted its exact mode while the held controller, history, focused tile and content aperture remained unchanged".into());
                self.receipt_complete = true;
                proof.stage = 6;
            },
            _ => unreachable!(),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn browser_presentation_continues_the_scenario_and_yields_while_the_editor_owns_input() {
        let mut proof = NativeAppearanceProof {
            stage: 0,
            expected: None,
            artifact: PathBuf::from("browser.png"),
            baseline: None,
            started: Instant::now(),
            initial_choice: None,
            expected_initial: None,
        };
        assert!(
            proof.needs_browser_frame(),
            "the first presentation owes the drawer click"
        );
        proof.stage = 2;
        assert!(
            !proof.needs_browser_frame(),
            "the editor's host drives the active tool window"
        );
        proof.editor_closed(ThemeChoice::default());
        assert!(
            proof.needs_browser_frame(),
            "editor completion owes saved-choice input"
        );
        proof.stage = 6;
        assert!(
            !proof.needs_browser_frame(),
            "the completed workspace receipt owns termination"
        );
    }
}
