/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Hest is the command vocabulary of the Mere stack.
//!
//! [`menu`] declares and offers commands: the command set, the person's
//! choices, and a palette's session. [`catalogue`] holds shared ids, labels,
//! effect classes and destructive flags. These are declarations; hosts still
//! decide confirmation, authorization and enactment.
//! Hosts own storage, drawing, and enactment. Serde is the only dependency.
//!
//! [`Invocation`] carries a command's opaque arguments and declared effect;
//! [`Outcome`] records only a host-established terminal result. The plan is
//! `design_docs/archive_docs/2026-10-28/2026-10-20_hest_command_architecture_plan.md`
//! in the Mere repository. Workspace consumers use this Hest 0.2 API.

pub mod catalogue;
mod invocation;
pub mod menu;
pub use invocation::{Invocation, Outcome};

/// The kind of change a host declares for an offered command.
///
/// This describes the semantic operation, not incidental persistence or
/// permission to execute it. Destructiveness is an independent declaration.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EffectClass {
    /// Changes local presentation or personal view state.
    Curation,
    /// Authors or changes the application's domain data.
    DomainTruth,
    /// Requests an effect outside the application's domain model.
    ExternalEffect,
}
