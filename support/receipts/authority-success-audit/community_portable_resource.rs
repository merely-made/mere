// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Shared-resource and coop interaction draft, independent of a renderer.
//!
//! These are disclosed view values, not publication, collection or grant wire
//! types. Gemot supplies contribution/collection identities and effective reads;
//! Eidetic supplies exact manifest identity and content verification; the activity
//! owner supplies its address and lifecycle report. Each action goes back to its
//! owner for current authority, custody, quota and revision checks at dispatch.
//! An offer, membership report, selection or accepted UI completion grants none
//! of those rights. The core model never reads a store or authors an operation;
//! opt-in owner adapters read existing stores through their established APIs.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::coop::LifecycleReport;

#[path = "C:/Users/mark_/Code/repos/mere/ports/moot/src/resource_coop/control.rs"] pub mod control;
#[cfg(feature = "resource-handlers")]
pub mod handlers;
#[cfg(feature = "resource-owners")]
pub mod owners;
#[cfg(feature = "presentation")]
pub mod view;

/// A disclosed Gemot contribution and its exact Eidetic manifest. Equal bytes
/// do not merge independent contributions. This does not name a moving "latest".
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ResourceRef {
    pub moot: [u8; 32],
    pub contribution: [u8; 32],
    pub manifest: [u8; 32],
}

/// A domain-owned activity associated with this Moot. Opaque owner and activity
/// addresses stay distinct; neither grants membership or creates shared state.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ActivityRef {
    pub moot: [u8; 32],
    pub owner: String,
    pub activity: String,
}

/// Owner-disclosed bytes available to this reading, separate from durable custody.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentRead {
    AddressOnly,
    Resolved,
    Unavailable { reason: String },
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetentionRead {
    #[default]
    Unknown,
    Held {
        owner_receipt: String,
        retained_at_ms: u64,
    },
    Unavailable {
        reason: String,
    },
}
/// Owner-disclosed terms captured at preparation and rechecked at dispatch.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionTerms {
    pub revision: [u8; 32],
    pub description: String,
}

/// Hosting evidence supplied by its owner. An observation reports a particular
/// retrieval, never continuous reachability. No clock-driven promise fold lives here.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HostingRead {
    Unknown,
    Promise {
        owner_record: String,
    },
    Observed {
        owner_receipt: String,
        observed_at_ms: u64,
    },
    Expired {
        owner_record: String,
    },
    Withdrawn {
        owner_record: String,
    },
}

/// Disclosed affordance at this read. Offered means "request this from the owner",
/// not "authorized at dispatch". An absent entry is unknown, not known false.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Offer {
    Offered,
    Withheld { reason: String },
    Unsupported { reason: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ResourceAction {
    Open,
    RetainCopy,
    ForkContent,
    OfferHosting,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ActivityAction {
    Join,
    Leave,
}

/// A metadata card already disclosed by the owner. It contains no payload bytes,
/// roster inference, presence state or permission synthesized from a lifecycle.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceRead {
    pub reference: ResourceRef,
    /// Stable root of the signed sharing act, when its owner disclosed it.
    /// This does not assert authorship of the underlying content.
    #[serde(default)]
    pub shared_by: Option<[u8; 32]>,
    pub title: String,
    pub schema: String,
    pub content: ContentRead,
    #[serde(default)]
    pub retention: RetentionRead,
    #[serde(default)]
    pub terms: BTreeMap<ResourceAction, ActionTerms>,
    pub hosting: HostingRead,
    pub offers: BTreeMap<ResourceAction, Offer>,
}

/// A copy of the current Gemot collection view's exact frontier and membership
/// commitment. The owner must resolve and verify these again before forking.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectionRead {
    pub id: [u8; 32],
    pub name: String,
    pub heads: Vec<[u8; 32]>,
    pub membership_commitment: [u8; 32],
    pub fork: Option<Offer>,
    /// The reviewed destination is view data and part of the exact intention.
    /// It is neither a new collection wire record nor authority to author it.
    #[serde(default)]
    pub fork_destination: Option<ForkDestination>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForkDestination {
    pub id: [u8; 32],
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivityRead {
    pub reference: ActivityRef,
    pub title: String,
    pub lifecycle: LifecycleReport,
    /// References may be unresolved or withheld in this reading. Their absence
    /// from resources does not fabricate content or deny the whole activity.
    pub resources: Vec<ResourceRef>,
    pub offers: BTreeMap<ActivityAction, Offer>,
}

/// A fresh owner read for one reader and Moot. It is neither durable nor a
/// capability. Its supplying application keeps owner authorization at dispatch.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
    pub moot: [u8; 32],
    pub reader: [u8; 32],
    pub collection: Option<CollectionRead>,
    pub resources: Vec<ResourceRead>,
    pub activities: Vec<ActivityRead>,
}

impl Snapshot {
    pub(crate) fn check(&self) -> Result<(), Refusal> {
        let mut contributions = BTreeSet::new();
        for resource in &self.resources {
            if resource.reference.moot != self.moot {
                return Err(Refusal::ForeignMoot);
            }
            if !contributions.insert(resource.reference.contribution) {
                return Err(Refusal::DuplicateContribution);
            }
        }
        let mut activities = BTreeSet::new();
        for activity in &self.activities {
            if activity.reference.moot != self.moot
                || activity
                    .resources
                    .iter()
                    .any(|resource| resource.moot != self.moot)
            {
                return Err(Refusal::ForeignMoot);
            }
            if activity.reference.owner.trim().is_empty()
                || activity.reference.activity.trim().is_empty()
            {
                return Err(Refusal::MissingActivityOwner);
            }
            if !activities.insert(&activity.reference) {
                return Err(Refusal::DuplicateActivity);
            }
            if activity.lifecycle.version != crate::coop::VERSION {
                return Err(Refusal::InvalidLifecycle(
                    "unsupported report version".into(),
                ));
            }
            activity
                .lifecycle
                .check_invariants()
                .map_err(Refusal::InvalidLifecycle)?;
        }
        Ok(())
    }
}

/// Typed local refusal. None of these is a verdict about a remote peer's rights.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    ForeignMoot,
    DifferentReader,
    DuplicateContribution,
    ChangedContribution,
    ChangedOfferTerms,
    ChangedCollection,
    DuplicateActivity,
    MissingActivityOwner,
    InvalidLifecycle(String),
    UnknownResource,
    UnknownActivity,
    NoCollection,
    NoResourceSelected,
    NoActivitySelected,
    UnknownOffer,
    InvalidForkDestination,
    OwnerOffer(Offer),
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::ChangedCollection => "The reviewed collection changed",
            Self::ChangedOfferTerms => "The reviewed resource offer terms changed",
            Self::ForeignMoot => "This read belongs to another Moot",
            Self::DifferentReader => "This read belongs to another reader",
            Self::DuplicateContribution => "The read repeats a contribution identity",
            Self::ChangedContribution => {
                "The exact contribution changed its content or sharing root"
            },
            Self::DuplicateActivity => "The read repeats an activity identity",
            Self::MissingActivityOwner => "The activity needs an owner and an exact address",
            Self::InvalidLifecycle(reason) => {
                return write!(formatter, "The activity report is invalid: {reason}");
            },
            Self::UnknownResource => "The resource is absent from the current read",
            Self::UnknownActivity => "The activity is absent from the current read",
            Self::NoCollection => "There is no collection in this read",
            Self::NoResourceSelected => "Choose a resource first",
            Self::NoActivitySelected => "Choose an activity first",
            Self::UnknownOffer => "The owner has not disclosed this action",
            Self::InvalidForkDestination => {
                "Review a named, different destination collection first"
            },
            Self::OwnerOffer(Offer::Withheld { reason }) => {
                return write!(formatter, "The owner withheld this action: {reason}");
            },
            Self::OwnerOffer(Offer::Unsupported { reason }) => {
                return write!(formatter, "This action is unavailable: {reason}");
            },
            Self::OwnerOffer(Offer::Offered) => "The action needs a current owner check",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for Refusal {}

/// The existing domain receives this intention and rechecks current authority.
/// A collection fork is reference/lineage work, separate from copying content.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    Refresh {
        moot: [u8; 32],
    },
    Resource {
        reference: ResourceRef,
        action: ResourceAction,
        observed_terms: Option<ActionTerms>,
    },
    CollectionFork {
        moot: [u8; 32],
        observed: CollectionRead,
    },
    Activity {
        reference: ActivityRef,
        action: ActivityAction,
    },
}

/// A local completion binding. It cannot be serialized, imported or used as a
/// grant; its private identity belongs to one preparation in one pane.
#[derive(Clone, Debug)]
pub struct Request {
    pub reader: [u8; 32],
    pub target: Target,
    identity: Arc<()>,
}

/// Reported by the actual action owner. Acceptance does not refresh custody,
/// hosting or participation; only a new owner Snapshot changes those reads.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Outcome {
    Accepted { owner_receipt: String },
    Refused { reason: String },
    Unsupported { reason: String },
}

/// Independent attention and request feedback for one reading. Closing a host
/// pane or editing its Forme never implicitly prepares one of these actions.
#[derive(Debug)]
pub struct Pane {
    snapshot: Snapshot,
    selected_resource: Option<ResourceRef>,
    selected_activity: Option<ActivityRef>,
    pending: Option<Request>,
    outcome: Option<Outcome>,
}

impl Pane {
    pub fn prepare_refresh(&mut self) -> Request {
        self.prepare(Target::Refresh {
            moot: self.snapshot.moot,
        })
    }
    pub fn complete_refresh(
        &mut self,
        request: &Request,
        snapshot: Snapshot,
    ) -> Result<bool, Refusal> {
        if !self.is_pending(request) || !matches!(request.target, Target::Refresh { .. }) {
            return Ok(false);
        }
        self.refresh(snapshot)?;
        Ok(true)
    }
    pub fn new(snapshot: Snapshot) -> Result<Self, Refusal> {
        snapshot.check()?;
        Ok(Self {
            snapshot,
            selected_resource: None,
            selected_activity: None,
            pending: None,
            outcome: None,
        })
    }

    pub fn snapshot(&self) -> &Snapshot {
        &self.snapshot
    }
    pub fn selected_resource(&self) -> Option<ResourceRef> {
        self.selected_resource
    }
    pub fn selected_activity(&self) -> Option<&ActivityRef> {
        self.selected_activity.as_ref()
    }
    pub fn outcome(&self) -> Option<&Outcome> {
        self.outcome.as_ref()
    }

    pub fn pending(&self) -> Option<&Request> {
        self.pending.as_ref()
    }

    pub fn is_pending(&self, request: &Request) -> bool {
        self.pending.as_ref().is_some_and(|pending| {
            Arc::ptr_eq(&pending.identity, &request.identity)
                && pending.reader == request.reader
                && pending.target == request.target
        })
    }

    pub fn select_resource(&mut self, reference: ResourceRef) -> Result<(), Refusal> {
        if !self
            .snapshot
            .resources
            .iter()
            .any(|item| item.reference == reference)
        {
            return Err(Refusal::UnknownResource);
        }
        if self.selected_resource != Some(reference) {
            self.selected_resource = Some(reference);
            self.invalidate();
        }
        Ok(())
    }

    pub fn select_activity(&mut self, reference: &ActivityRef) -> Result<(), Refusal> {
        if !self
            .snapshot
            .activities
            .iter()
            .any(|item| &item.reference == reference)
        {
            return Err(Refusal::UnknownActivity);
        }
        if self.selected_activity.as_ref() != Some(reference) {
            self.selected_activity = Some(reference.clone());
            self.invalidate();
        }
        Ok(())
    }

    /// A changed exact contribution is malformed; a newer, distinct contribution
    /// never silently retargets the selected old revision. Refusal preserves state.
    pub fn refresh(&mut self, snapshot: Snapshot) -> Result<(), Refusal> {
        snapshot.check()?;
        if snapshot.moot != self.snapshot.moot {
            return Err(Refusal::ForeignMoot);
        }
        if snapshot.reader != self.snapshot.reader {
            return Err(Refusal::DifferentReader);
        }
        for resource in &snapshot.resources {
            if self.snapshot.resources.iter().any(|old| {
                old.reference.contribution == resource.reference.contribution
                    && (old.reference.manifest != resource.reference.manifest
                        || old
                            .shared_by
                            .zip(resource.shared_by)
                            .is_some_and(|(old, new)| old != new))
            }) {
                return Err(Refusal::ChangedContribution);
            }
        }
        self.snapshot = snapshot;
        if self.selected_resource.is_some_and(|selected| {
            !self
                .snapshot
                .resources
                .iter()
                .any(|item| item.reference == selected)
        }) {
            self.selected_resource = None;
        }
        if self.selected_activity.as_ref().is_some_and(|selected| {
            !self
                .snapshot
                .activities
                .iter()
                .any(|item| &item.reference == selected)
        }) {
            self.selected_activity = None;
        }
        self.invalidate();
        Ok(())
    }

    pub fn prepare_resource(&mut self, action: ResourceAction) -> Result<Request, Refusal> {
        let reference = self.selected_resource.ok_or(Refusal::NoResourceSelected)?;
        let item = self
            .snapshot
            .resources
            .iter()
            .find(|item| item.reference == reference)
            .ok_or(Refusal::UnknownResource)?;
        require_offer(item.offers.get(&action))?;
        let observed_terms = item.terms.get(&action).cloned();
        Ok(self.prepare(Target::Resource {
            reference,
            action,
            observed_terms,
        }))
    }

    pub fn prepare_collection_fork(&mut self) -> Result<Request, Refusal> {
        let observed = self
            .snapshot
            .collection
            .as_ref()
            .ok_or(Refusal::NoCollection)?;
        require_offer(observed.fork.as_ref())?;
        if !observed
            .fork_destination
            .as_ref()
            .is_some_and(|destination| {
                destination.id != observed.id && !destination.name.trim().is_empty()
            })
        {
            return Err(Refusal::InvalidForkDestination);
        }
        Ok(self.prepare(Target::CollectionFork {
            moot: self.snapshot.moot,
            observed: observed.clone(),
        }))
    }

    pub fn prepare_activity(&mut self, action: ActivityAction) -> Result<Request, Refusal> {
        let reference = self
            .selected_activity
            .as_ref()
            .ok_or(Refusal::NoActivitySelected)?;
        let item = self
            .snapshot
            .activities
            .iter()
            .find(|item| &item.reference == reference)
            .ok_or(Refusal::UnknownActivity)?;
        require_offer(item.offers.get(&action))?;
        Ok(self.prepare(Target::Activity {
            reference: reference.clone(),
            action,
        }))
    }

    /// Stale or other-pane completions cannot attach. This does not cancel an
    /// owner command already dispatched; its authoritative receipt remains there.
    pub fn complete(&mut self, request: &Request, outcome: Outcome) -> bool {
        if !self.is_pending(request) {
            return false;
        }
        self.pending = None;
        self.outcome = Some(outcome);
        true
    }

    fn prepare(&mut self, target: Target) -> Request {
        let request = Request {
            reader: self.snapshot.reader,
            target,
            identity: Arc::new(()),
        };
        self.pending = Some(request.clone());
        self.outcome = None;
        request
    }

    fn invalidate(&mut self) {
        self.pending = None;
        self.outcome = None;
    }
}

fn require_offer(offer: Option<&Offer>) -> Result<(), Refusal> {
    match offer {
        Some(Offer::Offered) => Ok(()),
        Some(other) => Err(Refusal::OwnerOffer(other.clone())),
        None => Err(Refusal::UnknownOffer),
    }
}

/// Re-resolve an intention against a fresh owner read before dispatch. This
/// supplies no authority itself; the command owner still checks its own policy.
pub fn validate_request(request: &Request, current: &Snapshot) -> Result<(), Refusal> {
    current.check()?;
    if current.reader != request.reader {
        return Err(Refusal::DifferentReader);
    }
    match &request.target {
        Target::Refresh { moot } => {
            if *moot == current.moot {
                Ok(())
            } else {
                Err(Refusal::ForeignMoot)
            }
        },
        Target::Resource {
            reference,
            action,
            observed_terms,
        } => {
            if reference.moot != current.moot {
                return Err(Refusal::ForeignMoot);
            }
            let item = current
                .resources
                .iter()
                .find(|item| item.reference == *reference)
                .ok_or(Refusal::UnknownResource)?;
            require_offer(item.offers.get(action))?;
            if item.terms.get(action) != observed_terms.as_ref() {
                return Err(Refusal::ChangedOfferTerms);
            }
            Ok(())
        },
        Target::Activity { reference, action } => {
            if reference.moot != current.moot {
                return Err(Refusal::ForeignMoot);
            }
            let item = current
                .activities
                .iter()
                .find(|item| item.reference == *reference)
                .ok_or(Refusal::UnknownActivity)?;
            require_offer(item.offers.get(action))
        },
        Target::CollectionFork { moot, observed } => {
            if *moot != current.moot {
                return Err(Refusal::ForeignMoot);
            }
            let collection = current.collection.as_ref().ok_or(Refusal::NoCollection)?;
            require_offer(collection.fork.as_ref())?;
            if collection.id != observed.id
                || collection.heads != observed.heads
                || collection.membership_commitment != observed.membership_commitment
            {
                return Err(Refusal::ChangedCollection);
            }
            Ok(())
        },
    }
}

#[cfg(test)]
#[path = "C:/Users/mark_/Code/repos/mere/ports/moot/src/resource_coop/tests.rs"]
mod tests;
