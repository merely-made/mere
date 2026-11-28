// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use crate::model::relation::stored::predicate_uuid;
use crate::profile::core::{self, ids};
use serde_json::json;

use super::*;

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct Record {
    #[serde(with = "super")]
    predicate: String,
    #[serde(default, with = "super::option")]
    maybe: Option<String>,
}

fn record(predicate: &str) -> Record {
    Record {
        predicate: predicate.into(),
        maybe: Some(predicate.into()),
    }
}

#[test]
fn a_core_predicate_stores_its_uuid_alone_and_an_open_one_its_iri_too() {
    let cites = core::iri(ids::REL_CITES);
    let wire = serde_json::to_value(record(cites)).unwrap();
    assert_eq!(wire["predicate"], json!({ "uuid": predicate_uuid(cites) }));
    assert_eq!(
        serde_json::from_value::<Record>(wire).unwrap(),
        record(cites)
    );

    // A workspace term is not a core one, so it keeps its IRI.
    let frame = "https://mere.computer/ns/rel#arrangement.frame-member";
    assert!(core::terms().entry(frame).is_none());
    for predicate in [frame, "urn:test:open"] {
        let wire = serde_json::to_value(record(predicate)).unwrap();
        assert_eq!(wire["predicate"]["iri"], predicate);
        assert_eq!(
            serde_json::from_value::<Record>(wire).unwrap(),
            record(predicate)
        );
    }

    // Positional: both fields always, and the round trip holds.
    for predicate in [cites, "urn:test:open"] {
        let bytes = postcard::to_allocvec(&record(predicate)).unwrap();
        assert_eq!(
            postcard::from_bytes::<Record>(&bytes).unwrap(),
            record(predicate)
        );
    }
}

#[test]
fn text_stored_before_d28_reads_and_a_record_that_does_not_resolve_is_refused() {
    let cites = core::iri(ids::REL_CITES);
    let old = json!({ "predicate": cites, "maybe": null });
    assert_eq!(
        serde_json::from_value::<Record>(old).unwrap(),
        Record {
            predicate: cites.into(),
            maybe: None,
        }
    );
    // The control reads; each break is refused.
    let good =
        json!({ "predicate": { "uuid": predicate_uuid("urn:test:open"), "iri": "urn:test:open" } });
    assert!(serde_json::from_value::<Record>(good).is_ok());
    let unresolved = json!({ "predicate": { "uuid": predicate_uuid("urn:test:open") } });
    assert!(serde_json::from_value::<Record>(unresolved).is_err());
    let wrong = json!({ "predicate": { "uuid": predicate_uuid(cites), "iri": "urn:test:open" } });
    assert!(serde_json::from_value::<Record>(wrong).is_err());
}
