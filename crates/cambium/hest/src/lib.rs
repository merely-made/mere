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
//! Invocation and outcomes are later phases of
//! `design_docs/cambium_docs/implementation_strategy/2026-10-20_hest_command_architecture_plan.md`
//! in the Mere repository. The `command-menu` compatibility crate stays on
//! the frozen Hest 0.1 API while this crate advances to 0.2.

pub mod catalogue;
pub mod menu;

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
