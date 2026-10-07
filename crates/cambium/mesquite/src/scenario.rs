// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Configuration and text receipt compatibility for the shared Mesquite lane.
//! This module has no scenario pump; both receipt formats use `Lane`.

use cambium::{FileEvent, FileRequest, OpenedFile};
use cambium_rootstock::{FileAnswer, FileChooser};
use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
};

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

/// One frame the lane read back.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaptureRecord {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub digest: u64,
    pub blank: bool,
    pub file: Option<PathBuf>,
}

pub(crate) type FileReader = fn(&Path) -> Option<OpenedFile>;

pub(crate) struct ScriptFiles {
    pub config: LaneConfig,
    reader: FileReader,
    parked: Rc<RefCell<Option<FileAnswer>>>,
    installed: bool,
}

struct ParkedFiles(Rc<RefCell<Option<FileAnswer>>>);
impl FileChooser for ParkedFiles {
    fn open(&mut self, _: &FileRequest, answer: FileAnswer) {
        if let Some(earlier) = self.0.borrow_mut().replace(answer) {
            earlier.send(FileEvent::default());
        }
    }
}

impl ScriptFiles {
    pub fn new(config: LaneConfig, reader: FileReader) -> Self {
        Self {
            config,
            reader,
            parked: Rc::default(),
            installed: false,
        }
    }
    pub fn install(&mut self, files: &mut Option<Box<dyn FileChooser>>) {
        if !self.installed {
            *files = Some(Box::new(ParkedFiles(self.parked.clone())));
            self.installed = true;
        }
    }
    pub fn answer(&mut self, argument: &str) -> Result<(), String> {
        let answer = self
            .parked
            .borrow_mut()
            .take()
            .ok_or_else(|| "no file request is waiting".to_string())?;
        if argument == "cancel" {
            answer.send(FileEvent::default());
        } else {
            let path = self
                .config
                .scenario
                .parent()
                .unwrap_or(Path::new("."))
                .join(argument);
            let file = (self.reader)(&path)
                .ok_or_else(|| format!("file {} unreadable", path.display()))?;
            answer.send(FileEvent { files: vec![file] });
        }
        Ok(())
    }
    pub fn capture_path(&self, name: &str) -> Option<PathBuf> {
        self.config.capture_dir.as_ref().map(|dir| {
            let safe: String = name
                .chars()
                .map(|c| {
                    if c.is_ascii_alphanumeric() || c == '-' {
                        c
                    } else {
                        '_'
                    }
                })
                .collect();
            dir.join(format!("{safe}.png"))
        })
    }
}
