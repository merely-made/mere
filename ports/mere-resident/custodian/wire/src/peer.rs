// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Who is on the other end, as the OS says so, read by Armillary's
//! `peer-facts` (capability door plan D7) and re-exported here. Both ends use
//! it: the custodian to select a principal, the client to verify the
//! custodian before sending anything confidential. What a custodian must be
//! is this crate's policy ([`ServerExpectation`]).

use serde::{Deserialize, Serialize};

pub use peer_facts::{Peer, SignedIdentity};
#[cfg(unix)]
pub use peer_facts::{effective_uid, unix_peer};
#[cfg(target_os = "macos")]
pub use peer_facts::signed_peer;
#[cfg(windows)]
pub use peer_facts::{current_user_sid, pipe_client_peer, process_image, service_sid};
#[cfg(windows)]
pub use crate::transport::windows::{PipeSecurity, owner_pipe_sddl};

/// What a client accepts as the custodian: the installed service identity.
/// On Linux a socket-activated listener reports systemd (uid 0) as its
/// creator, so root is listed beside the service uid there. On macOS a
/// same-user agent is identified by its code signature, since its uid is
/// everyone's in that session.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServerExpectation {
    #[serde(default)]
    pub unix_uids: Vec<u32>,
    #[serde(default)]
    pub windows_sids: Vec<String>,
    #[serde(default)]
    pub signed: Vec<SignedIdentity>,
}

impl ServerExpectation {
    /// Whether `peer` is an accepted custodian.
    pub fn admits(&self, peer: &Peer) -> bool {
        match peer {
            Peer::Unix { uid, .. } => self.unix_uids.contains(uid),
            Peer::Windows { sid, .. } => self.windows_sids.iter().any(|s| s == sid),
            Peer::Signed { team, identifier } => self
                .signed
                .iter()
                .any(|s| &s.team == team && &s.identifier == identifier),
        }
    }
}
