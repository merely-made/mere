// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Stage address additions, then commit one validated contact.

use super::{AddressIntake, IntakeError, IntakeOutcome, NewLocalContact, nonblank};
use crate::{Anchor, Contact, ContactBook, Endpoint, EndpointKind, Handle};

impl ContactBook {
    /// Add resolver address claims without replacing local relationship state.
    ///
    /// The primary typed handle must select exactly one contact. With no match,
    /// an unused host-selected fallback creates a keyless Local contact. Aliases
    /// never choose or join people; an alias held elsewhere refuses the whole
    /// intake. All input is checked before mutation. Existing addresses keep
    /// their exact spelling, trust (including alarms) and usage history; new
    /// ones start Unverified. Nothing changes keys, notes, petnames, tiers,
    /// recency, or stronger handle bindings. Reordered/duplicate claims replay
    /// without changes. This is a model operation, not a persistence transaction.
    pub fn intake_addresses(
        &mut self,
        intake: &AddressIntake,
        new: Option<NewLocalContact>,
    ) -> Result<IntakeOutcome, IntakeError> {
        let primary = intake.handle.comparison_key()?;
        let claims: Vec<_> = std::iter::once(&intake.handle)
            .chain(&intake.handles)
            .map(|claim| claim.comparison_key().map(|key| (claim, key)))
            .collect::<Result<_, _>>()?;
        for endpoint in &intake.endpoints {
            nonblank(&endpoint.address, "endpoint")?;
            if let EndpointKind::Other(kind) = &endpoint.kind {
                nonblank(kind, "endpoint kind")?;
            }
        }
        let matches: Vec<_> = self
            .iter()
            .filter(|contact| {
                contact
                    .handles
                    .iter()
                    .any(|held| intake.handle.matches(held, &primary))
            })
            .map(|contact| contact.anchor().clone())
            .collect();
        let (mut contact, created) = match matches.as_slice() {
            [anchor] => (
                self.get(anchor).expect("selected contact exists").clone(),
                false,
            ),
            [] => {
                let new = new.ok_or(IntakeError::LocalContactRequired)?;
                nonblank(&new.petname, "petname")?;
                let anchor = Anchor::Local(new.id);
                if self.get(&anchor).is_some() {
                    return Err(IntakeError::LocalAnchorOccupied(anchor));
                }
                (Contact::new_local(new.petname, new.id), true)
            },
            _ => return Err(IntakeError::Ambiguous(matches)),
        };
        for (claim, comparison) in &claims {
            let anchors: Vec<_> = self
                .iter()
                .filter(|other| {
                    other.anchor() != contact.anchor()
                        && other
                            .handles
                            .iter()
                            .any(|held| claim.matches(held, comparison))
                })
                .map(|other| other.anchor().clone())
                .collect();
            if !anchors.is_empty() {
                return Err(IntakeError::HandleConflict {
                    handle: (*claim).clone(),
                    anchors,
                });
            }
        }
        let mut outcome = IntakeOutcome {
            anchor: contact.anchor().clone(),
            created,
            handles_added: 0,
            endpoints_added: 0,
        };
        for (claim, comparison) in claims {
            if !contact
                .handles
                .iter()
                .any(|held| claim.matches(held, &comparison))
            {
                contact
                    .handles
                    .push(Handle::new(claim.kind.clone(), &claim.value));
                outcome.handles_added += 1;
            }
        }
        for claim in &intake.endpoints {
            if !contact
                .endpoints
                .iter()
                .any(|held| held.kind == claim.kind && held.address == claim.address)
            {
                contact
                    .endpoints
                    .push(Endpoint::new(claim.kind.clone(), &claim.address));
                outcome.endpoints_added += 1;
            }
        }
        if outcome.changed() {
            self.insert(contact);
        }
        Ok(outcome)
    }
}
