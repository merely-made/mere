// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Portable authored-theme JSON and explicit artifact export. Parsing returns
//! an inert definition; registering, forking and saving it remain explicit.

use std::io;
use std::path::Path;

use crate::Theme;
use crate::library::{atomic_write, validate_definition};

/// Whether an export may replace an existing file.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WriteMode {
    /// Publish the complete artifact only if its path does not already exist.
    #[default]
    CreateNew,
    /// Explicitly replace the existing file atomically, retaining permissions.
    Replace,
}

/// Parse the existing [`Theme`] JSON shape and validate its authored data.
/// IDs use the registry's trim/ASCII-lowercase normalization. Built-in source
/// markers are preserved; callers must fork these to user identities before
/// editing or inserting them into an authored library.
pub fn parse_theme_json(contents: &str) -> io::Result<Theme> {
    let mut theme: Theme = serde_json::from_str(contents).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid theme JSON: {error}"),
        )
    })?;
    theme.id = theme.id.trim().to_ascii_lowercase();
    validate_definition(&theme)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    Ok(theme)
}

/// Pretty JSON preserving all authored fields, with a trailing newline.
/// Validation prevents exporting a definition that could not be imported.
pub fn theme_json(theme: &Theme) -> io::Result<String> {
    validate_definition(theme)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    let mut json = serde_json::to_string_pretty(theme).map_err(io::Error::other)?;
    json.push('\n');
    Ok(json)
}

/// Write a complete artifact using the library's shared atomic writer. The
/// default [`WriteMode::CreateNew`] refuses existing paths; replacement must
/// be explicit. This operation never changes an authored-library save point.
/// A separate lock coordinates cooperating writers; changed bytes are checked
/// before publication. CreateNew uses a hard link, so filesystems lacking hard
/// link support return a visible I/O error without publishing a partial file.
pub fn write_artifact(
    path: impl AsRef<Path>,
    contents: impl AsRef<[u8]>,
    mode: WriteMode,
) -> io::Result<()> {
    atomic_write(path.as_ref(), contents.as_ref(), mode, None)
}
