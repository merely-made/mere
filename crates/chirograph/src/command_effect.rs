// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Chirograph adapts the shared declaration; Hest stays independent of the wire.

use crate::IntentEffect;
use hest::EffectClass;

impl From<EffectClass> for IntentEffect {
    fn from(effect: EffectClass) -> Self {
        match effect {
            EffectClass::Curation => Self::Curation,
            EffectClass::DomainTruth => Self::DomainTruth,
            EffectClass::ExternalEffect => Self::ExternalEffect,
        }
    }
}

impl From<IntentEffect> for EffectClass {
    fn from(effect: IntentEffect) -> Self {
        match effect {
            IntentEffect::Curation => Self::Curation,
            IntentEffect::DomainTruth => Self::DomainTruth,
            IntentEffect::ExternalEffect => Self::ExternalEffect,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_effect_round_trips_without_changing_the_wire_representation() {
        for effect in [
            EffectClass::Curation,
            EffectClass::DomainTruth,
            EffectClass::ExternalEffect,
        ] {
            let wire = IntentEffect::from(effect);
            assert_eq!(EffectClass::from(wire), effect);
            assert_eq!(
                serde_json::to_string(&wire).unwrap(),
                serde_json::to_string(&effect).unwrap()
            );
        }
    }
}
