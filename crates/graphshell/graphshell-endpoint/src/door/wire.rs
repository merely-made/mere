// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The first-party door's envelope, as both halves speak it.
//!
//! An application speaks [`AppMessage`] and hears [`AppHostMessage`], each
//! one native-messaging frame ([`super::frame`]). The carrier payload inside
//! is the browser door's; the envelope is separate so this door can version
//! independently of a shipped browser extension. The client half is
//! [`super::app_client`]; the server half is Graphshell's `native::app_broker`.
//! Both name these types, so they cannot drift.

use chirograph::{CarrierNotice, CarrierRequest, CarrierResponse};
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};

use crate::door::app_admission::AppId;
use crate::door::custody::{CustodyAnswer, CustodyCall, CustodyRefusal};

/// Schema of the connect answer an application sends.
pub const APP_CONNECT_SCHEMA: &str = "mere.graphshell/app-connect/v1";

/// Schema of the host challenge. Named for the browser door, which defined
/// it first; the application door sends the same challenge.
pub const BROWSER_CHALLENGE_SCHEMA: &str = "mere.graphshell/browser-challenge/v1";

/// First host message on every native-messaging conversation, on either door.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserChallenge {
    pub schema: String,
    pub host_nonce: String,
}

impl BrowserChallenge {
    /// A challenge with a fresh 32-byte host nonce, base64url without padding.
    pub fn fresh() -> Self {
        let mut nonce = [0u8; 32];
        OsRng.fill_bytes(&mut nonce);
        Self {
            schema: BROWSER_CHALLENGE_SCHEMA.to_string(),
            host_nonce: base64::Engine::encode(
                &base64::engine::general_purpose::URL_SAFE_NO_PAD,
                nonce,
            ),
        }
    }
}

/// Application-to-host messages.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AppMessage {
    /// Answer the host challenge and open the session.
    Connect {
        schema: String,
        host_nonce: String,
        client_nonce: String,
    },
    /// One carrier request against the admitted endpoint.
    Request { request: CarrierRequest },
    /// Poll for one available notice with bounded stream intake.
    TakeNotice,
    /// Wait until the admitted endpoint rings with a revision notice.
    WaitNotice,
    /// One call on the custody route ([`crate::door::custody`]).
    Call { id: u64, call: CustodyCall },
}

/// Host-to-application messages.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AppHostMessage {
    Challenge {
        challenge: BrowserChallenge,
    },
    Connected {
        app: AppId,
        session: String,
    },
    Response {
        response: CarrierResponse,
    },
    Notice {
        notice: Option<CarrierNotice>,
    },
    /// The answer to one custody call, by its id.
    Answer {
        id: u64,
        answer: Result<CustodyAnswer, CustodyRefusal>,
    },
    Failure {
        message: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_challenge_carries_a_32_byte_nonce_under_the_shared_schema() {
        let challenge = BrowserChallenge::fresh();
        assert_eq!(challenge.schema, BROWSER_CHALLENGE_SCHEMA);
        let nonce = base64::Engine::decode(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
            &challenge.host_nonce,
        )
        .unwrap();
        assert_eq!(nonce.len(), 32);
        assert_ne!(challenge, BrowserChallenge::fresh());
    }

    #[test]
    fn the_envelope_is_tagged_snake_case() {
        let json = serde_json::to_value(AppMessage::TakeNotice).unwrap();
        assert_eq!(json, serde_json::json!({ "type": "take_notice" }));
        let json = serde_json::to_value(AppHostMessage::Failure {
            message: "no".into(),
        })
        .unwrap();
        assert_eq!(
            json,
            serde_json::json!({ "type": "failure", "message": "no" })
        );
    }
}
