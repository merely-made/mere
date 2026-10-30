// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Typed envelopes shared by applications and the Djinn door broker.
//!
//! Each [`AppMessage`] or [`AppHostMessage`] occupies one native-messaging
//! frame. Payload slots are generic so custody and projection can use the same
//! envelope without depending on each other. The common handshake is carried
//! by `super::app_client` on native platforms; `super::projection` and
//! `super::custody_client` supply their own typed operations.

#[cfg(all(
    feature = "door-serve",
    any(unix, windows),
    not(target_arch = "wasm32")
))]
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};

use crate::door::app_admission::AppId;

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
    #[cfg(all(
        feature = "door-serve",
        any(unix, windows),
        not(target_arch = "wasm32")
    ))]
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
pub enum AppMessage<Request, Call> {
    /// Answer the host challenge and open the session.
    Connect {
        schema: String,
        host_nonce: String,
        client_nonce: String,
    },
    /// One carrier request against the admitted endpoint.
    Request { request: Request },
    /// Poll for one available notice with bounded stream intake.
    TakeNotice,
    /// Wait until the admitted endpoint rings with a revision notice.
    WaitNotice,
    /// One call on the custody route ([`crate::door::custody`]).
    Call { id: u64, call: Call },
}

/// Host-to-application messages.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AppHostMessage<Response, Notice, Answer> {
    Challenge {
        challenge: BrowserChallenge,
    },
    Connected {
        app: AppId,
        session: String,
    },
    Response {
        response: Response,
    },
    Notice {
        notice: Option<Notice>,
    },
    /// The answer to one custody call, by its id.
    Answer {
        id: u64,
        answer: Answer,
    },
    Failure {
        message: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic_envelope_keeps_the_existing_wire_shape() {
        let message: AppMessage<String, u32> = AppMessage::Call { id: 7, call: 42 };
        assert_eq!(
            serde_json::to_string(&message).unwrap(),
            r#"{"type":"call","id":7,"call":42}"#
        );
        let answer: AppHostMessage<String, u32, bool> =
            serde_json::from_str(r#"{"type":"answer","id":7,"answer":true}"#).unwrap();
        assert!(matches!(
            answer,
            AppHostMessage::Answer {
                id: 7,
                answer: true
            }
        ));
    }

    #[cfg(feature = "projection")]
    #[test]
    fn projection_slots_preserve_exact_wire_fixtures() {
        use chirograph::{
            CarrierNotice, CarrierRequest, CarrierRequestBody, CarrierResponse,
            CarrierResponseBody, ProjectionSession, Revision, SceneEpoch,
        };
        type Incoming = AppMessage<CarrierRequest, ()>;
        type Outgoing = AppHostMessage<CarrierResponse, CarrierNotice, ()>;
        let cases = [
            (
                Outgoing::Response {
                    response: CarrierResponse {
                        id: 7,
                        body: Ok(CarrierResponseBody::Closed),
                    },
                },
                r#"{"type":"response","response":{"id":7,"body":{"Ok":"Closed"}}}"#,
            ),
            (
                Outgoing::Notice {
                    notice: Some(CarrierNotice {
                        session: ProjectionSession("graph".into()),
                        epoch: SceneEpoch(3),
                        revision: Revision(4),
                    }),
                },
                r#"{"type":"notice","notice":{"session":"graph","epoch":3,"revision":4}}"#,
            ),
        ];
        let request = Incoming::Request {
            request: CarrierRequest {
                id: 7,
                body: CarrierRequestBody::Discover,
            },
        };
        let fixture = r#"{"type":"request","request":{"id":7,"body":"Discover"}}"#;
        assert_eq!(serde_json::to_string(&request).unwrap(), fixture);
        assert_eq!(serde_json::from_str::<Incoming>(fixture).unwrap(), request);
        for (message, fixture) in cases {
            assert_eq!(serde_json::to_string(&message).unwrap(), fixture);
            assert_eq!(serde_json::from_str::<Outgoing>(fixture).unwrap(), message);
        }
    }

    #[cfg(all(feature = "custody", any(unix, windows), not(target_arch = "wasm32")))]
    #[test]
    fn released_custody_answer_preserves_exact_wire_fixture() {
        use crate::door::custody::{CustodyAnswer, CustodyRefusal};
        type Outgoing = AppHostMessage<(), (), Result<CustodyAnswer, CustodyRefusal>>;
        let answer = Outgoing::Answer {
            id: 9,
            answer: Ok(CustodyAnswer::Released {
                master: [0x5a; 32],
                keys: vec![],
            }),
        };
        let fixture = r#"{"type":"answer","id":9,"answer":{"Ok":{"answer":"released","master":"WlpaWlpaWlpaWlpaWlpaWlpaWlpaWlpaWlpaWlpaWlo","keys":[]}}}"#;
        assert_eq!(serde_json::to_string(&answer).unwrap(), fixture);
        assert_eq!(serde_json::from_str::<Outgoing>(fixture).unwrap(), answer);
    }

    #[cfg(all(
        feature = "door-serve",
        any(unix, windows),
        not(target_arch = "wasm32")
    ))]
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
        let json = serde_json::to_value(AppMessage::<(), ()>::TakeNotice).unwrap();
        assert_eq!(json, serde_json::json!({ "type": "take_notice" }));
        let json = serde_json::to_value(AppHostMessage::<(), (), ()>::Failure {
            message: "no".into(),
        })
        .unwrap();
        assert_eq!(
            json,
            serde_json::json!({ "type": "failure", "message": "no" })
        );
    }
}
