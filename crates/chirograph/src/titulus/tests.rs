// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

fn form() -> ActionFormV1 {
    ActionFormV1::new("fixture.save/v1").with_field(ActionFormFieldV1::choice(
        "digest",
        "Saved facts",
        [
            ActionFormChoiceV1::new("facts-a", "Morning facts"),
            ActionFormChoiceV1::new("facts-b", "Evening facts"),
        ],
    ))
}

#[test]
fn a_form_composes_only_the_exact_values_the_endpoint_offered() {
    // The bound that makes this safe to hand a host: it returns the
    // endpoint's own opaque value, never something the host composed.
    let values = BTreeMap::from([("digest".to_string(), "facts-b".to_string())]);
    let payload: serde_json::Value =
        serde_json::from_slice(&form().compose_payload("fixture.save/v1", &values).unwrap())
            .unwrap();
    assert_eq!(
        payload,
        serde_json::json!({"schema": "fixture.save/v1", "digest": "facts-b"})
    );
}

#[test]
fn a_value_outside_the_offered_choices_is_refused() {
    // A host that invents a value is the failure this form exists to make
    // impossible; it must not reach the endpoint as a plausible payload.
    let values = BTreeMap::from([("digest".to_string(), "facts-invented".to_string())]);
    assert!(matches!(
        form().compose_payload("fixture.save/v1", &values),
        Err(ActionFormError::InvalidChoice { .. })
    ));
}

#[test]
fn the_form_must_match_the_schema_it_is_composed_against() {
    let values = BTreeMap::from([("digest".to_string(), "facts-a".to_string())]);
    assert!(matches!(
        form().compose_payload("fixture.other/v1", &values),
        Err(ActionFormError::SchemaMismatch { .. })
    ));
}

#[test]
fn a_missing_required_value_and_an_unknown_name_are_both_refused() {
    assert!(matches!(
        form().compose_payload("fixture.save/v1", &BTreeMap::new()),
        Err(ActionFormError::MissingField(_))
    ));
    let stray = BTreeMap::from([("nowhere".to_string(), "x".to_string())]);
    assert!(matches!(
        form().compose_payload("fixture.save/v1", &stray),
        Err(ActionFormError::UnknownField(_))
    ));
}

#[test]
fn an_optional_field_may_be_omitted() {
    let optional = ActionFormV1::new("fixture.save/v1").with_field(
        ActionFormFieldV1::choice("digest", "Saved facts", [ActionFormChoiceV1::new("a", "A")])
            .optional(),
    );
    let payload: serde_json::Value = serde_json::from_slice(
        &optional
            .compose_payload("fixture.save/v1", &BTreeMap::new())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(payload, serde_json::json!({"schema": "fixture.save/v1"}));
}

#[test]
fn schema_is_a_reserved_field_name_because_the_form_writes_it_itself() {
    let clashing = ActionFormV1::new("fixture.save/v1").with_field(ActionFormFieldV1::choice(
        "schema",
        "Schema",
        [ActionFormChoiceV1::new("x", "X")],
    ));
    assert_eq!(clashing.validate(), Err(ActionFormError::ReservedFieldName));
}

#[test]
fn a_content_hash_addresses_bytes_and_prints_as_hex() {
    let hash = ContentHash::of(b"a projected resource");
    assert_eq!(hash, ContentHash::of(b"a projected resource"));
    assert_ne!(hash, ContentHash::of(b"a different one"));
    let rendered = hash.to_string();
    assert_eq!(rendered.len(), 64);
    assert!(rendered.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn portable_content_reference_uses_canonical_ni_and_blake3_spellings() {
    let reference = PortableContentRefV1::of(b"portable bytes");
    reference.verify_bytes(b"portable bytes").unwrap();
    let json = serde_json::to_value(&reference).unwrap();
    let portable = json["portable_id"].as_str().unwrap();
    assert!(portable.starts_with(Sha256NamedInformation::PREFIX));
    assert!(!portable.contains('='));
    assert!(json["transport"].as_str().unwrap().starts_with("blake3:"));
    assert_eq!(
        serde_json::from_value::<PortableContentRefV1>(json).unwrap(),
        reference,
    );
}

#[test]
fn portable_and_transport_conflicts_fail_closed() {
    let bytes = b"portable bytes";
    let mut wrong_portable = PortableContentRefV1::of(bytes);
    wrong_portable.portable_id = Sha256NamedInformation::of(b"different");
    assert_eq!(
        wrong_portable.verify_bytes(bytes),
        Err(ContentReferenceError::Sha256Mismatch),
    );

    let mut wrong_transport = PortableContentRefV1::of(bytes);
    wrong_transport.transport = ContentHash::of(b"different");
    assert_eq!(
        wrong_transport.verify_bytes(bytes),
        Err(ContentReferenceError::Blake3Mismatch),
    );
}

#[test]
fn named_information_parser_rejects_private_or_noncanonical_spellings() {
    let named = Sha256NamedInformation::of(b"named bytes");
    assert_eq!(
        named.to_string().parse::<Sha256NamedInformation>(),
        Ok(named)
    );
    assert!(
        "urn:sha256:abababababababababababababababababababababababababababababababab"
            .parse::<Sha256NamedInformation>()
            .is_err()
    );
    assert!(
        format!("{}{}=", Sha256NamedInformation::PREFIX, "a".repeat(43))
            .parse::<Sha256NamedInformation>()
            .is_err()
    );
}

#[test]
fn named_information_matches_the_rfc_6920_hello_world_vector() {
    const RFC_URI: &str = "ni:///sha-256;f4OxZX_x_FO5LcGBSKHWXfwtSx-j1ncoSt3SABJtkGk";
    let named = Sha256NamedInformation::of(b"Hello World!");
    assert_eq!(named.to_string(), RFC_URI);
    assert_eq!(RFC_URI.parse::<Sha256NamedInformation>(), Ok(named));
}

#[test]
fn a_card_round_trips_and_carries_only_what_a_host_may_render() {
    let card = PortableCardV1 {
        title: "Identity vault".to_string(),
        values: vec![CardValueV1 {
            label: "Protection".to_string(),
            value: "OS-protected".to_string(),
        }],
        badges: vec!["Personae".to_string()],
        media: vec![ContentHash::of(b"thumb")],
    };
    let json = serde_json::to_string(&card).unwrap();
    assert_eq!(serde_json::from_str::<PortableCardV1>(&json).unwrap(), card);
}
