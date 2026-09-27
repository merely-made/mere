// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The desktop's half of the scenario lane.
//!
//! The lane itself is rootstock's ([`cambium_rootstock::scenario`]), shared
//! with the browser host. This module is what the desktop adds: [`LaneConfig`]
//! names the scenario and where its evidence goes, and [`NativeLane`] reads a
//! frame back at once, writes each capture as a PNG, reads a `file` step's file
//! from beside the scenario, writes the receipt to a file, and closes the
//! application once it is written.
//!
//! An application builds a lane from [`LaneConfig::from_env`] and calls
//! [`ScenarioLane::drive`] from its `after_frame` hook. With no scenario named,
//! it builds no lane and runs as usual.
//!
//! Evidence: each `capture <name>` writes `<name>.png` into the capture
//! directory when one is configured and records its size and digest. The
//! receipt goes to the configured receipt path, or to `scenario.done` in the
//! capture directory, and doubles as the sentinel a harness waits for.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use cambium::OpenedFile;
use cambium_rootstock::CaptureFn;
use cambium_rootstock::meristem_bounds::RootView;
use cambium_rootstock::scenario::{self as lane, LaneHost};

use crate::{AppCtx, Frame, read_frame};

pub use cambium_rootstock::scenario::{CaptureRecord, LaneApp, ProbeSnapshot};

/// Where a lane reads its scenario and writes its evidence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaneConfig {
    pub scenario: PathBuf,
    pub capture_dir: Option<PathBuf>,
    pub receipt: Option<PathBuf>,
}

impl LaneConfig {
    /// Read `<PREFIX>_SCENARIO`, `<PREFIX>_CAPTURE_DIR` and `<PREFIX>_RECEIPT`.
    /// `None` when no scenario is named.
    pub fn from_env(prefix: &str) -> Option<Self> {
        let var = |suffix: &str| std::env::var_os(format!("{prefix}_{suffix}")).map(PathBuf::from);
        Some(Self {
            scenario: var("SCENARIO")?,
            capture_dir: var("CAPTURE_DIR"),
            receipt: var("RECEIPT"),
        })
    }

    /// The explicit receipt path, else `scenario.done` in the capture directory.
    pub fn receipt_path(&self) -> Option<PathBuf> {
        self.receipt.clone().or_else(|| {
            self.capture_dir
                .as_ref()
                .map(|dir| dir.join("scenario.done"))
        })
    }
}

/// The desktop's half of a lane: files for evidence, and frames read at once.
pub struct NativeLane {
    config: LaneConfig,
}

impl LaneHost for NativeLane {
    fn arm_capture(
        &mut self,
        capture: &mut Option<CaptureFn>,
    ) -> Box<dyn FnMut() -> Option<Frame>> {
        let sink = Rc::new(RefCell::new(None::<Frame>));
        let out = sink.clone();
        *capture = Some(Box::new(move |surface, view, width, height| {
            *out.borrow_mut() = read_frame(surface, view, width, height);
        }));
        Box::new(move || sink.borrow_mut().take())
    }

    fn keep(&mut self, name: &str, frame: &Frame) -> Result<Option<PathBuf>, String> {
        let Some(dir) = &self.config.capture_dir else {
            return Ok(None);
        };
        let safe: String = name
            .chars()
            .map(|ch| {
                if ch.is_ascii_alphanumeric() || ch == '-' {
                    ch
                } else {
                    '_'
                }
            })
            .collect();
        let path = dir.join(format!("{safe}.png"));
        write_png(&path, frame)
            .map(|()| Some(path.clone()))
            .map_err(|error| format!("could not write {}: {error}", path.display()))
    }

    fn open_file(&mut self, argument: &str) -> Result<OpenedFile, String> {
        let base = self.config.scenario.parent().unwrap_or(Path::new("."));
        let path = base.join(argument);
        crate::read_file(&path).ok_or_else(|| format!("file {} unreadable", path.display()))
    }

    fn publish(&mut self, _ok: bool, text: &str) {
        eprintln!("[scenario] {text}");
        if let Some(path) = self.config.receipt_path() {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Err(error) = std::fs::write(&path, format!("{text}\n")) {
                eprintln!("[scenario] could not write {}: {error}", path.display());
            }
        }
    }
}

/// A running scenario on the desktop: rootstock's lane with [`NativeLane`].
pub struct ScenarioLane<A>(lane::ScenarioLane<A, NativeLane>);

impl<A> ScenarioLane<A> {
    /// Read and parse the configured scenario.
    pub fn new(config: LaneConfig, app: A) -> Result<Self, String> {
        let text = std::fs::read_to_string(&config.scenario).map_err(|error| {
            format!("scenario {} unreadable: {error}", config.scenario.display())
        })?;
        let path = config.scenario.display().to_string();
        lane::ScenarioLane::new(&text, app, NativeLane { config })
            .map(Self)
            .map_err(|error| format!("scenario {path} rejected: {error}"))
    }

    pub fn app(&self) -> &A {
        self.0.app()
    }

    pub fn app_mut(&mut self) -> &mut A {
        self.0.app_mut()
    }

    pub fn captures(&self) -> &[CaptureRecord] {
        self.0.captures()
    }

    /// Whether the receipt has been written.
    pub fn finished(&self) -> bool {
        self.0.finished()
    }

    /// Pump one step. Call from the `after_frame` hook, so every step reads a
    /// state that was actually rendered. Asks the host to close once the
    /// receipt is written.
    pub fn drive<State: 'static, Logic, V>(&mut self, ctx: &mut AppCtx<'_, State, Logic, V>)
    where
        A: LaneApp<State, Logic, V>,
        Logic: FnMut(&State) -> V,
        V: RootView<State>,
    {
        self.0.drive(ctx);
    }
}

fn write_png(path: &Path, frame: &Frame) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let file = std::fs::File::create(path).map_err(|error| error.to_string())?;
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), frame.width, frame.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|error| error.to_string())?;
    writer
        .write_image_data(&frame.rgba)
        .map_err(|error| error.to_string())?;
    writer.finish().map_err(|error| error.to_string())
}
