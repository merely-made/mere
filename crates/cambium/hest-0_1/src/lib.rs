/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Hest is the command vocabulary of the Mere stack.
//!
//! [`menu`] declares and offers commands: the command set, the person's
//! choices, and a palette's session. [`catalogue`] holds shared ids and labels.
//! Hosts own storage, drawing, and enactment. Serde is the only dependency.
//!
//! Effect classification, invocation, and outcomes are later phases of
//! `design_docs/cambium_docs/implementation_strategy/2026-10-20_hest_command_architecture_plan.md`
//! in the Mere repository. H1 moves the published `command-menu` API unchanged.

pub mod catalogue;
pub mod menu;
