// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

#![cfg(feature = "projection-door")]
use ostiary::wire::{AppHostMessage, AppMessage};
#[test]
fn projection_slots_preserve_exact_wire_fixtures() {
    use chirograph::{
        CarrierNotice, CarrierRequest, CarrierRequestBody, CarrierResponse, CarrierResponseBody,
        ProjectionSession, Revision, SceneEpoch,
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
