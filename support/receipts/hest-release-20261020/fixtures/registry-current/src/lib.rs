// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

#[cfg(test)]
mod tests {
    #[test]
    fn declared_effect_and_invocation_cross_the_published_facade() {
        let mut command = hest::menu::Command::new(
            "fixture:write", "Write", "fixture", hest::EffectClass::DomainTruth,
        );
        command.destructive = true;
        let facade: cambium::Command = command;
        let row = cambium::CommandItem::from(&facade);
        assert_eq!(row.id, "fixture:write");
        assert!(facade.destructive);
        let request: cambium::Invocation = facade.invocation("fixture.write/v1", vec![0,255,128]);
        let encoded = serde_json::to_vec(&request).unwrap();
        let shared: hest::Invocation = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(shared, request);
        assert_eq!(shared.effect, hest::EffectClass::DomainTruth);
        assert_eq!(shared.arguments, vec![0,255,128]);
    }
    #[test]
    fn only_terminal_outcomes_cross_the_published_facade() {
        for shared in [hest::Outcome::Done, hest::Outcome::Stale,
            hest::Outcome::Refused { reason: "confirmation required".into() }] {
            let facade: cambium::Outcome = shared.clone();
            let bytes = serde_json::to_vec(&facade).unwrap();
            assert_eq!(serde_json::from_slice::<hest::Outcome>(&bytes).unwrap(), shared);
        }
    }
}
