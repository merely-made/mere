/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Compatibility re-exports of Hest's command menu and shared catalogue.
//!
//! Existing `command_menu::` imports keep the same API. New consumers use
//! `hest::menu` and `hest::catalogue` directly.

pub use hest::catalogue;
pub use hest::menu::{Command, CommandChoices, CommandSet, MenuSession};
