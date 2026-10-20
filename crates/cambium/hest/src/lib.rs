/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! **Hest** (an old word for a command) is the command architecture of the
//! Mere stack. This release only holds the name.
//!
//! The pieces exist today but live apart: `command-menu` declares and offers
//! commands (the set, the person's choices, the palette's session, the shared
//! ids), Cambium draws them, Chirograph carries invocations and results, and
//! hosts keep their own enums. The plan is one vocabulary for the whole
//! lifecycle: declare, offer, invoke, enact and audit. `command-menu`
//! becomes `hest::menu`. See
//! `design_docs/cambium_docs/implementation_strategy/2026-10-20_hest_command_architecture_plan.md`
//! in the Mere repository.
