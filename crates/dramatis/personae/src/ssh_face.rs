// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What one face may do over SSH.
//!
//! A person has personae, plural, and they are not interchangeable: the
//! work face reaches everything, the research face reads, the burner gets
//! in and does one thing. This module is where that difference becomes
//! mechanical — a face's policy fixes the principals its certificates may
//! name and the actions its grants may carry, so *which persona was
//! unlocked* decides reach, rather than which machine happened to run the
//! command.
//!
//! Two independent enforcement points, which is what makes the burner
//! safe rather than merely narrow:
//!
//! 1. **The certificate.** Actions the policy omits are extensions the
//!    certificate never carries, so the host permits no pty, no
//!    forwarding, nothing.
//! 2. **The host's enrollment line.** `principals="markik"` in a
//!    `cert-authority` line means a certificate naming any other principal
//!    is refused *by sshd*, before the certificate's own contents matter.
//!
//! A policy lives in the profile as an ordinary slot, so it travels with
//! the face and needs no change to the profile wire format. Reading and
//! writing that slot is custody, in castellan (dramatis repo plan, DR-B).
//!
//! Feature `ssh`.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::carry::{
    ACTION_SSH_AGENT_FORWARD, ACTION_SSH_LOGIN, ACTION_SSH_PORT_FORWARD, ACTION_SSH_PTY,
};
use crate::vault::ProtocolKey;

/// The `mod_id` a face's SSH policy is stored under.
pub const SSH_FACE_MOD_ID: &str = "ssh-face";

/// One face's SSH reach.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FacePolicy {
    /// Unix accounts certificates for this face may name.
    pub principals: Vec<String>,
    /// `ssh.*` actions its grants carry, which become certificate extensions.
    pub actions: BTreeSet<String>,
    /// A command to force in place of whatever the client asks for.
    pub force_command: Option<String>,
    /// Addresses its certificates work from, in OpenSSH's CIDR list form.
    pub source_address: Option<String>,
}

impl FacePolicy {
    /// The full-reach face: login, terminal, and both forwardings.
    pub fn work(principal: impl Into<String>) -> Self {
        Self {
            principals: vec![principal.into()],
            actions: [
                ACTION_SSH_LOGIN,
                ACTION_SSH_PTY,
                ACTION_SSH_AGENT_FORWARD,
                ACTION_SSH_PORT_FORWARD,
            ]
            .iter()
            .map(|action| (*action).to_string())
            .collect(),
            force_command: None,
            source_address: None,
        }
    }

    /// A terminal and nothing to carry with it: no agent, no ports.
    ///
    /// The reason to have this rather than reuse [`Self::work`]: agent
    /// forwarding hands the far end the ability to sign as you, which is
    /// exactly the authority a research face should not be lending out.
    pub fn research(principal: impl Into<String>) -> Self {
        Self {
            principals: vec![principal.into()],
            actions: [ACTION_SSH_LOGIN, ACTION_SSH_PTY]
                .iter()
                .map(|action| (*action).to_string())
                .collect(),
            force_command: None,
            source_address: None,
        }
    }

    /// One principal, one command, no extensions at all.
    pub fn burner(principal: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            principals: vec![principal.into()],
            actions: [ACTION_SSH_LOGIN.to_string()].into_iter().collect(),
            force_command: Some(command.into()),
            source_address: None,
        }
    }

    /// Whether this policy is a narrowing of `parent`.
    ///
    /// The delegation grammar's own attenuation rule, applied to the face
    /// layer: a face may drop principals and actions, and may add a forced
    /// command, but may never widen either set.
    pub fn attenuates(&self, parent: &Self) -> bool {
        self.actions.is_subset(&parent.actions)
            && self
                .principals
                .iter()
                .all(|principal| parent.principals.contains(principal))
    }

    /// Actions as the borrowed strings the grant builder wants.
    pub fn action_refs(&self) -> Vec<&str> {
        self.actions.iter().map(String::as_str).collect()
    }
}

/// The protocol key a face policy stores under.
pub fn policy_key() -> ProtocolKey {
    ProtocolKey::new(SSH_FACE_MOD_ID, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_faces_narrow_in_the_order_they_are_named() {
        let work = FacePolicy::work("markik");
        let research = FacePolicy::research("markik");
        let burner = FacePolicy::burner("markik", "uptime");

        assert!(research.attenuates(&work));
        assert!(burner.attenuates(&research));
        assert!(!work.attenuates(&research), "a face may not widen");

        // The burner carries exactly one action, so its certificates carry
        // no extensions at all.
        assert_eq!(burner.actions.len(), 1);
        assert!(burner.actions.contains(ACTION_SSH_LOGIN));
        assert!(burner.force_command.is_some());

        // Research keeps a terminal but lends no authority onward.
        assert!(!research.actions.contains(ACTION_SSH_AGENT_FORWARD));
        assert!(research.actions.contains(ACTION_SSH_PTY));
    }

    #[test]
    fn a_face_may_not_borrow_a_principal_it_was_not_given() {
        let work = FacePolicy::work("markik");
        let mut other = FacePolicy::research("markik");
        other.principals.push("root".into());
        assert!(!other.attenuates(&work));
    }
}
