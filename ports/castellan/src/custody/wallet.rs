// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The wallet's secret half, moved from pandect (dramatis repo plan, DR-B,
//! ruling D8): the identity seed and the unlock ladder that opens it, this
//! host's delegated-device identity, the remote-auth wrapping keys, the
//! private-epoch bridge, first-launch bootstrap, and the grant flows that sign
//! or wrap with them (issue, refresh, revoke, enrollment install, legacy
//! re-issue). pandect keeps the public manifests, the roster, the grant
//! vocabulary and the file layout these write into.

mod bootstrap;
mod devices;
mod enroll;
mod epochs;
mod issue;
mod migrate;
mod refresh;
mod revoke;
mod sealer;
mod secrets;
mod wrapping;

#[cfg(test)]
mod revocation_tests;
#[cfg(test)]
mod store_test_support;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod trust_tests;
#[cfg(test)]
mod validate_tests;

// What the moved flows name from pandect, as they named it from inside it.
use pandect::wallet_grant::*;
use pandect::wallet_store::*;

pub use bootstrap::{WalletBootstrapMode, bootstrap_wallet_state, ensure_wallet_state};
pub use devices::{
    ensure_local_device_identity, load_local_device_identity, load_remote_auth_wrapping_key_bridge,
    save_local_device_identity, save_remote_auth_wrapping_key_bridge,
};
pub use enroll::{
    install_remote_auth_enrollment_bundle, install_remote_auth_enrollment_bundle_with_wrapping_key,
};
pub use epochs::{
    ensure_persona_epoch_bridge, load_current_private_epoch, load_persona_epoch_bridge,
    save_persona_epoch_bridge, stage_persona_private_epoch,
};
pub use issue::{
    issue_remote_auth_device_grant, issue_remote_auth_device_grant_from_pairing,
    issue_remote_auth_device_grant_from_ticket,
};
pub use migrate::reissue_legacy_grant;
pub use revoke::revoke_remote_auth_device;
pub use sealer::epoch_sealer_for_persona;
pub use secrets::{
    identity_seed_locked_at_startup, load_identity_seed, load_identity_seed_read_only,
    relock_wallet_after_manual_unlock, save_identity_seed, unlock_wallet_with_auto_os,
    wallet_local_secrets_locked,
};

pub(crate) use refresh::{
    refresh_remote_auth_private_read_grants, upsert_persona_capability_slots,
};
pub(crate) use wrapping::{
    load_remote_auth_wrapping_key, remove_remote_auth_wrapping_key, upsert_remote_auth_wrapping_key,
};
