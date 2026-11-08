// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The domain registry: how a domain reaches the resident (cycle breaks P1b).
//!
//! One registry that every composition reads, the daemon and the embedded
//! resident alike, where each domain names its id, its owning application and
//! its authority (domain adoption plan M1). A product domain that is reserved
//! or, in strict mode, any domain that is not registered is refused when
//! served, with a typed `domain-not-registered` error (M2). The design is the
//! registration section of
//! `design_docs/mere_docs/technical_architecture/2026-11-06_domain_contract.md`;
//! cycle breaks D15 removed its lane and distribution parts, so there is no
//! `Lane` here.
//!
//! Registration is a value built before any route is served. It grants no
//! admission: the owner's grants and the application's declaration still
//! decide who may open `mere/<domain>`.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::Arc;

use ostiary::app_admission::AppId;
use personae::{PersonaId, ProfileId};

/// Whether a profile exists in the owner's vault: the resident-side fact that
/// an admission check may need (personal-facts' new profile subjects).
pub type ProfileLookup = Arc<dyn Fn(&ProfileId) -> Result<bool, String> + Send + Sync>;

/// Builds a domain's before/after admission check once the resident knows the
/// facts it needs, today only the profile lookup.
pub type AdmissionBuilder =
    Arc<dyn Fn(Option<ProfileLookup>) -> pandect::GraphAdmissionValidator + Send + Sync>;

/// What refuses an edit when a domain's authority is not yet installed.
pub const PENDING: &str = "pending: no authority installed";

/// A wasm component that supplies a graph domain's authority (cycle breaks P3). The pin is
/// always there; the running guest is there only when the component is installed and
/// verified. Without it the authority refuses every candidate as [`PENDING`].
#[derive(Clone)]
#[non_exhaustive]
pub struct GuestAuthority {
    /// Hash of the component, pinned across paired devices (D15 (h)).
    pub component: [u8; 32],
    runtime: Option<Arc<crate::authority_guest::GuestRuntime>>,
    /// Why the guest is absent (revoked, missing, altered), for the refusal text.
    reason: Option<String>,
}

impl fmt::Debug for GuestAuthority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GuestAuthority")
            .field("component", &self.component)
            .field("installed", &self.runtime.is_some())
            .field("reason", &self.reason)
            .finish()
    }
}

impl PartialEq for GuestAuthority {
    fn eq(&self, other: &Self) -> bool {
        self.component == other.component
            && self.runtime.is_some() == other.runtime.is_some()
            && self.reason == other.reason
    }
}

impl Eq for GuestAuthority {}

impl GuestAuthority {
    /// A pinned guest that is not installed: the domain refuses as pending.
    pub fn new(component: [u8; 32]) -> Self {
        Self {
            component,
            runtime: None,
            reason: None,
        }
    }

    /// A pinned guest that could not be loaded, with the reason the refusal carries.
    pub fn absent(component: [u8; 32], reason: impl Into<String>) -> Self {
        Self {
            component,
            runtime: None,
            reason: Some(reason.into()),
        }
    }

    /// A verified, running guest.
    pub fn installed(component: [u8; 32], runtime: Arc<crate::authority_guest::GuestRuntime>) -> Self {
        Self {
            component,
            runtime: Some(runtime),
            reason: None,
        }
    }

    /// The running guest, when one is installed.
    pub fn runtime(&self) -> Option<&Arc<crate::authority_guest::GuestRuntime>> {
        self.runtime.as_ref()
    }
}

impl DeclaredAuthority {
    pub fn new(policy: impl Into<String>) -> Self {
        Self { policy: policy.into() }
    }
}

/// Declared data a Stickleback `OperationPolicy`-shaped interpreter will run.
///
/// A typed pending slot until M4, its first resident-side consumer (Mark,
/// cycle breaks D24 (A), 2026-11-08): no interpreter is built, a domain that
/// registers it is refused as pending, and Knot's policy keeps running in
/// Knot's own sync host.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct DeclaredAuthority {
    /// Which declared policy this names.
    pub policy: String,
}

/// What decides whether a candidate is admissible. Required for every
/// registration.
#[derive(Clone)]
pub enum Authority {
    /// Linked code: a pure, deterministic check over the complete candidate
    /// graph, and an optional before/after check.
    Graph {
        complete: pandect::GraphValidator,
        admission: Option<AdmissionBuilder>,
    },
    /// A reviewed wasm guest. Without an installed guest, edits are refused as pending.
    Guest(GuestAuthority),
    /// Declared data. A pending slot until M4: edits are refused as pending.
    Declared(DeclaredAuthority),
}

impl Authority {
    /// A graph authority with only the complete-candidate check.
    pub fn graph(complete: pandect::GraphValidator) -> Self {
        Self::Graph {
            complete,
            admission: None,
        }
    }

    /// The short kind name shown in status.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Graph { .. } => "graph",
            Self::Guest(_) => "guest",
            Self::Declared(_) => "declared",
        }
    }

    /// The complete-candidate check the resident runs on edits, archive
    /// save, open and compose, and loaded sessions. A guest authority without an
    /// installed guest, and a declared one until its interpreter lands, refuse every
    /// candidate as pending: an operation waits, it is never applied unchecked (D15 (h)).
    pub(crate) fn validator(&self, domain: &str) -> pandect::GraphValidator {
        match self {
            Self::Graph { complete, .. } => complete.clone(),
            Self::Guest(guest) => match &guest.runtime {
                Some(runtime) => runtime.validator(),
                None => pending(domain, guest.reason.as_deref()),
            },
            Self::Declared(_) => pending(domain, None),
        }
    }

    pub(crate) fn admission(&self) -> Option<AdmissionBuilder> {
        match self {
            Self::Graph { admission, .. } => admission.clone(),
            Self::Guest(guest) => guest.runtime.as_ref().and_then(|runtime| runtime.admission()),
            Self::Declared(_) => None,
        }
    }
}

fn pending(domain: &str, reason: Option<&str>) -> pandect::GraphValidator {
    let message = match reason {
        Some(reason) => format!("{PENDING} for {domain} ({reason})"),
        None => format!("{PENDING} for {domain}"),
    };
    Arc::new(move |_| Err(message.clone()))
}

/// One domain's registration.
#[derive(Clone)]
pub struct DomainRegistration {
    /// The domain id, named for the data (`"divination"`, `"notes"`).
    pub domain: String,
    /// The owning application label.
    pub owner: AppId,
    /// The grammar version, shown in status.
    pub version: &'static str,
    pub authority: Authority,
}

impl DomainRegistration {
    pub fn new(
        domain: impl Into<String>,
        owner: AppId,
        version: &'static str,
        authority: Authority,
    ) -> Self {
        Self {
            domain: domain.into(),
            owner,
            version,
            authority,
        }
    }
}

impl DomainRegistration {
    /// Run this registration's complete-candidate check over `graph`, as a route would
    /// on an edit or a load. A guest authority without an installed guest refuses with
    /// [`PENDING`].
    pub fn validate(&self, graph: &kernel::graph::Graph) -> Result<(), String> {
        (self.authority.validator(&self.domain))(graph)
    }
}

/// What a served-but-unregistered domain gets.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Unregistered {
    /// Serve it under generic graph authority, as before M2. Reserved
    /// domains are still refused. The default until every application
    /// registers.
    #[default]
    ServeGeneric,
    /// Refuse it (M2 in full).
    Refuse,
}

/// A registry error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RegistryError {
    /// The domain id is not a valid domain id.
    InvalidDomain(String),
    /// The domain is registered twice.
    Duplicate(String),
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDomain(domain) => write!(f, "invalid domain id {domain:?}"),
            Self::Duplicate(domain) => write!(f, "domain {domain} is registered twice"),
        }
    }
}

impl std::error::Error for RegistryError {}

/// The typed refusal for a domain the resident does not carry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DomainNotRegistered {
    pub domain: String,
    /// The domains this resident does carry, so a client can say so.
    pub registered: Vec<String>,
}

/// The stable prefix of the refusal's text, for clients to match.
pub const DOMAIN_NOT_REGISTERED: &str = "domain-not-registered";

impl fmt::Display for DomainNotRegistered {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{DOMAIN_NOT_REGISTERED}: {} (registered: {})",
            self.domain,
            self.registered.join(", ")
        )
    }
}

impl std::error::Error for DomainNotRegistered {}

/// The registrations a composition hands to its routes.
#[derive(Clone, Default)]
pub struct DomainRegistry {
    entries: BTreeMap<String, DomainRegistration>,
    reserved: BTreeSet<String>,
    unregistered: Unregistered,
}

impl DomainRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Mere's own domains for `persona`: `personal-facts`. `divination` is a
    /// reserved domain id (a name for data, no dependency), so a resident
    /// with no authority registered for it refuses it rather than serving it
    /// unchecked.
    pub fn mere_defaults(persona: PersonaId) -> Self {
        Self::new()
            .reserve("divination")
            .register(personal_facts_registration(persona))
            .expect("a fresh registry takes personal-facts")
    }

    /// [`Self::mere_defaults`] plus every authority guest the owner has approved for this
    /// persona (cycle breaks P3): one `Authority::Guest` registration per pin in `store`,
    /// pending when the guest cannot be loaded. A guest cannot take a domain Mere already
    /// registers itself; that is refused as a duplicate.
    pub fn resident_defaults(persona: PersonaId, store: &crate::authority_guest::GuestStore) -> Result<Self, RegistryError> {
        store
            .registrations()
            .into_iter()
            .try_fold(Self::mere_defaults(persona), |registry, registration| registry.register(registration))
    }

    /// Add a registration. A duplicate domain is refused.
    pub fn register(mut self, registration: DomainRegistration) -> Result<Self, RegistryError> {
        if pandect::DomainId::new(registration.domain.as_str()).is_err() {
            return Err(RegistryError::InvalidDomain(registration.domain));
        }
        if self.entries.contains_key(&registration.domain) {
            return Err(RegistryError::Duplicate(registration.domain));
        }
        self.entries
            .insert(registration.domain.clone(), registration);
        Ok(self)
    }

    /// Require this domain to be registered before it is served, even while
    /// unregistered domains are otherwise served generically.
    pub fn reserve(mut self, domain: impl Into<String>) -> Self {
        self.reserved.insert(domain.into());
        self
    }

    /// Choose what an unregistered, unreserved domain gets.
    pub fn unregistered(mut self, policy: Unregistered) -> Self {
        self.unregistered = policy;
        self
    }

    pub fn get(&self, domain: &str) -> Option<&DomainRegistration> {
        self.entries.get(domain)
    }

    /// The registered domain ids, in order.
    pub fn domains(&self) -> Vec<String> {
        self.entries.keys().cloned().collect()
    }

    pub(crate) fn entries(&self) -> impl Iterator<Item = &DomainRegistration> {
        self.entries.values()
    }

    pub(crate) fn reserved(&self) -> &BTreeSet<String> {
        &self.reserved
    }

    pub(crate) fn policy(&self) -> Unregistered {
        self.unregistered
    }
}

/// What the resident knows about one installed registration.
#[derive(Clone)]
pub(crate) struct Installed {
    pub owner: Option<AppId>,
    pub version: &'static str,
    pub kind: &'static str,
    pub validator: pandect::GraphValidator,
    pub admission: Option<AdmissionBuilder>,
}

/// A registered domain as status shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisteredDomain {
    pub domain: String,
    pub owner: Option<String>,
    pub version: String,
    pub authority: String,
}

/// Mere's own personal-facts domain: the biographical-fact rules, over
/// `mere-facts`, with the new-profile-subject check against the owner vault.
pub fn personal_facts_registration(persona: PersonaId) -> DomainRegistration {
    let persona = persona.as_uuid().to_string();
    let admission: AdmissionBuilder = Arc::new(|lookup| {
        Arc::new(move |before, after| {
            personal_facts::validate_new_profile_subjects(before, after, |id| match &lookup {
                Some(resolve) => resolve(id),
                None => Err("profile subjects require the resident vault authority".into()),
            })
        })
    });
    DomainRegistration::new(
        personal_facts::DOMAIN,
        AppId::new("mere"),
        "v1",
        Authority::Graph {
            complete: Arc::new(move |graph| personal_facts::validate_for_persona(graph, &persona)),
            admission: Some(admission),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guest_and_declared_validators_refuse_every_candidate_as_pending() {
        let graph = kernel::graph::Graph::new();
        for authority in [
            Authority::Guest(GuestAuthority::new([1; 32])),
            Authority::Declared(DeclaredAuthority::new("p")),
        ] {
            let refusal = authority.validator("fixture")(&graph).unwrap_err();
            assert_eq!(refusal, format!("{PENDING} for fixture"));
            assert!(authority.admission().is_none());
        }
    }

    #[test]
    fn a_graph_authority_runs_its_own_check() {
        let authority = Authority::graph(Arc::new(|_| Err("no".into())));
        assert_eq!(authority.validator("d")(&kernel::graph::Graph::new()).unwrap_err(), "no");
    }
}
