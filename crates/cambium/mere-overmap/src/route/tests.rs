// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use chirograph::CardValueV1;

use super::*;

fn card(values: &[(&str, &str)]) -> PortableCardV1 {
    PortableCardV1 {
        title: "Codicil blake3:f612a".into(),
        values: values
            .iter()
            .map(|(label, value)| CardValueV1 {
                label: (*label).into(),
                value: (*value).into(),
            })
            .collect(),
        badges: Vec::new(),
        media: Vec::new(),
    }
}

#[test]
fn a_codicil_card_names_the_one_session_it_was_saved_from() {
    let one = "00000000-0000-0000-0000-000000000001";
    let two = "00000000-0000-0000-0000-000000000002";
    assert_eq!(
        archive_session(&card(&[("Codicil", "blake3:f612a"), ("Session", one)])),
        Some(SessionId(Uuid::parse_str(one).unwrap()))
    );
    assert_eq!(
        archive_session(&card(&[("Session", one), ("Session", one)])),
        Some(SessionId(Uuid::parse_str(one).unwrap())),
        "the same session twice is still one"
    );
    assert_eq!(
        archive_session(&card(&[("Session", one), ("Session", two)])),
        None,
        "a composition names no single session"
    );
    assert_eq!(archive_session(&card(&[("Codicil", "blake3:f612a")])), None);
    assert_eq!(archive_session(&card(&[("Session", "not a uuid")])), None);
}
