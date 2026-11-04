// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Stemma-backed scope visits with mutable, view-only bookmarks.

use super::{ScopeCamera, ScopeVisit};
use chartulary::stemma::{
    BindingSnapshot, EntryPrivacy, EntrySnapshot, OwnerSnapshot, Stemma, StemmaSnapshot,
    TransitionKind, VisitId, VisitSnapshot,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::DeserializeOwned};
use std::fmt::Debug;

const OWNER: &str = "contextual-navigation";
const WIRE_VERSION: u8 = 1;
const MAX_RESTORED_VISITS: usize = 100_000;
const MAX_REMEMBERED: usize = 64;
const LEGACY_MAX_ENTRIES: usize = 128;

/// Values retained by contextual navigation and its persisted snapshot.
pub trait ScopeValue: Clone + Debug + PartialEq + Serialize + DeserializeOwned {}

impl<T> ScopeValue for T where T: Clone + Debug + PartialEq + Serialize + DeserializeOwned {}

type Lineage<P, A> = Stemma<u64, (), String, ScopeVisit<P, A>>;
type Snapshot<P, A> = StemmaSnapshot<u64, (), String, ScopeVisit<P, A>>;

/// Immutable visit lineage plus the latest view state for each visit.
///
/// A Stemma visit is never rewritten by `capture`; its corresponding bookmark
/// can be refreshed as the camera or reading anchor changes. Navigating after
/// Back adds a branch and retains the older forward branch.
#[derive(Clone, Debug, PartialEq)]
pub struct ScopeNavigation<P: ScopeValue, A: ScopeValue> {
    lineage: Snapshot<P, A>,
    /// Ordinal-aligned with `lineage.visits`; these are mutable view bookmarks,
    /// not a second history list.
    bookmarks: Vec<ScopeVisit<P, A>>,
    remembered: Vec<ScopeVisit<P, A>>,
}

impl<P: ScopeValue, A: ScopeValue> Default for ScopeNavigation<P, A> {
    fn default() -> Self {
        let mut memory = Lineage::<P, A>::new();
        memory.ensure_owner(OWNER.to_owned(), None);
        Self {
            lineage: canonical_snapshot(memory.to_snapshot()),
            bookmarks: Vec::new(),
            remembered: Vec::new(),
        }
    }
}

impl<P: ScopeValue, A: ScopeValue> ScopeNavigation<P, A> {
    /// The bookmark for the owner's current visit.
    pub fn current(&self) -> Option<&ScopeVisit<P, A>> {
        self.bookmarks.get(self.lineage.owners.first()?.current?)
    }

    /// All retained visit bookmarks, including visits on inactive branches,
    /// followed by the separately bounded remembered-place bookmarks.
    pub fn visits(&self) -> impl Iterator<Item = &ScopeVisit<P, A>> {
        self.bookmarks.iter().chain(&self.remembered)
    }

    /// The owner's parent or selected forward child, if one exists.
    pub fn destination(&self, forward: bool) -> Option<&ScopeVisit<P, A>> {
        let current = self.lineage.owners.first()?.current?;
        let visit = self.lineage.visits.get(current)?;
        let target = if forward {
            visit
                .bindings
                .first()
                .and_then(|binding| binding.forward_child)?
        } else {
            visit.parent?
        };
        self.bookmarks.get(target)
    }

    /// Retain the latest view bookmark for this exact place, up to 64 places.
    pub fn remember(&mut self, visit: ScopeVisit<P, A>) {
        self.remembered.retain(|saved| saved.place != visit.place);
        self.remembered.push(visit);
        if self.remembered.len() > MAX_REMEMBERED {
            self.remembered.remove(0);
        }
    }

    /// Find the newest remembered view whose place matches a host-defined scope.
    pub fn saved_matching(&self, same_scope: impl Fn(&P) -> bool) -> Option<&ScopeVisit<P, A>> {
        self.remembered
            .iter()
            .rev()
            .find(|visit| same_scope(&visit.place))
    }

    /// Refresh the current visit's mutable view bookmark. The caller may supply
    /// a newly focused member while the immutable scope visit remains intact.
    pub fn capture(&mut self, visit: ScopeVisit<P, A>) {
        let Some(ordinal) = self.current_ordinal() else {
            self.append_visit(visit);
            return;
        };
        self.bookmarks[ordinal] = visit.clone();
        self.remember(visit);
    }

    /// Record a new visit, branching from the current visit when needed.
    pub fn visit(&mut self, visit: ScopeVisit<P, A>) {
        if self
            .current()
            .is_some_and(|current| current.place == visit.place)
        {
            self.capture(visit);
            return;
        }
        self.append_visit(visit);
    }

    /// Move the Stemma cursor only after the host successfully restores the
    /// destination returned by [`Self::destination`].
    pub fn step(&mut self, forward: bool) -> bool {
        if self.destination(forward).is_none() {
            return false;
        }
        let mut memory = self.hydrate();
        let Some(owner) = memory.owner_id_by_identity(&OWNER.to_owned()) else {
            return false;
        };
        let moved = if forward {
            memory.forward(owner, 1, 0)
        } else {
            memory.back(owner, 1, 0)
        };
        if !matches!(moved, Ok(Some(_))) {
            return false;
        }
        self.lineage = canonical_snapshot(memory.to_snapshot());
        true
    }

    /// Validate route contents against host identity and anchor rules.
    pub fn validate(
        &self,
        valid_place: impl Fn(&P) -> bool,
        valid_anchor: impl Fn(&A) -> bool,
    ) -> Result<(), String> {
        validate_wire_state(&self.lineage, &self.bookmarks, &self.remembered)?;
        if self
            .bookmarks
            .iter()
            .chain(&self.remembered)
            .chain(self.lineage.visits.iter().map(|visit| &visit.context))
            .any(|visit| {
                !valid_place(&visit.place)
                    || visit
                        .anchor
                        .as_ref()
                        .is_some_and(|anchor| !valid_anchor(anchor))
                    || visit.camera.is_some_and(|camera| !camera.valid())
            })
        {
            return Err("Invalid scope navigation visit".into());
        }
        Ok(())
    }

    /// Refresh mutable visit and remembered-place bookmarks while retaining the
    /// immutable encounter lineage, branches, and owner cursor verbatim.
    ///
    /// Use this for live presentation changes. [`Self::try_map`] also maps the
    /// encounter contexts and is intended for admission or format migration.
    /// An error leaves this navigation unchanged.
    pub fn try_map_bookmarks(
        &self,
        mut map: impl FnMut(&ScopeVisit<P, A>) -> Result<ScopeVisit<P, A>, String>,
    ) -> Result<Self, String> {
        let bookmarks = self
            .bookmarks
            .iter()
            .map(&mut map)
            .collect::<Result<Vec<_>, _>>()?;
        let remembered = self
            .remembered
            .iter()
            .map(&mut map)
            .collect::<Result<Vec<_>, _>>()?;
        validate_wire_state(&self.lineage, &bookmarks, &remembered)?;
        Ok(Self {
            lineage: self.lineage.clone(),
            bookmarks,
            remembered,
        })
    }

    /// Map every retained visit and bookmark while keeping the Stemma's exact
    /// entry, parent, child, owner, and cursor structure. Mapping is staged in
    /// fresh values; an error leaves this navigation untouched.
    pub fn try_map<Q: ScopeValue, B: ScopeValue>(
        &self,
        mut map: impl FnMut(&ScopeVisit<P, A>) -> Result<ScopeVisit<Q, B>, String>,
    ) -> Result<ScopeNavigation<Q, B>, String> {
        let lineage = StemmaSnapshot {
            entries: self
                .lineage
                .entries
                .iter()
                .map(|entry| EntrySnapshot {
                    key: entry.key,
                    payload: (),
                    first_seen_at_ms: entry.first_seen_at_ms,
                    last_seen_at_ms: entry.last_seen_at_ms,
                    visit_count: entry.visit_count,
                    privacy: entry.privacy,
                })
                .collect(),
            visits: self
                .lineage
                .visits
                .iter()
                .map(|visit| {
                    Ok(VisitSnapshot {
                        entry: visit.entry,
                        parent: visit.parent,
                        children: visit.children.clone(),
                        created_at_ms: visit.created_at_ms,
                        context: map(&visit.context)?,
                        inbound: visit.inbound,
                        bindings: visit
                            .bindings
                            .iter()
                            .map(|binding| BindingSnapshot {
                                owner: binding.owner,
                                forward_child: binding.forward_child,
                                last_accessed_at_ms: binding.last_accessed_at_ms,
                            })
                            .collect(),
                    })
                })
                .collect::<Result<Vec<_>, String>>()?,
            owners: self
                .lineage
                .owners
                .iter()
                .map(|owner| OwnerSnapshot {
                    identity: owner.identity.clone(),
                    origin: owner.origin,
                    current: owner.current,
                    creator: owner.creator,
                    pending_origin_parent: owner.pending_origin_parent,
                    owned_visits: owner.owned_visits.clone(),
                })
                .collect(),
        };
        let bookmarks = self
            .bookmarks
            .iter()
            .map(&mut map)
            .collect::<Result<Vec<_>, _>>()?;
        let remembered = self
            .remembered
            .iter()
            .map(&mut map)
            .collect::<Result<Vec<_>, _>>()?;
        let lineage = canonical_snapshot(lineage);
        validate_wire_state(&lineage, &bookmarks, &remembered)?;
        Ok(ScopeNavigation {
            lineage,
            bookmarks,
            remembered,
        })
    }

    fn current_ordinal(&self) -> Option<usize> {
        let ordinal = self.lineage.owners.first()?.current?;
        self.bookmarks.get(ordinal)?;
        Some(ordinal)
    }

    fn append_visit(&mut self, visit: ScopeVisit<P, A>) {
        let mut memory = self.hydrate();
        let owner = memory.ensure_owner(OWNER.to_owned(), None);
        // Reuse the Stemma entry for this place if any retained visit names it.
        // This avoids adding a Hash requirement to the generic place type.
        let existing_entry = memory
            .visits()
            .find_map(|(_, old)| (old.context.place == visit.place).then_some(old.entry));
        let entry = existing_entry.unwrap_or_else(|| {
            let key = memory.entry_count() as u64;
            memory.resolve_or_create_entry(key, (), 0, EntryPrivacy::LocalOnly)
        });
        let Ok(visit_id) =
            memory.visit_entry(owner, entry, visit.clone(), TransitionKind::Unknown, 0)
        else {
            return;
        };
        let Some(ordinal) = visit_ordinal(&memory, visit_id) else {
            return;
        };
        if ordinal != self.bookmarks.len() {
            return;
        }
        self.bookmarks.push(visit.clone());
        self.remember(visit);
        self.lineage = canonical_snapshot(memory.to_snapshot());
    }

    fn hydrate(&self) -> Lineage<P, A> {
        // All snapshots reaching this private path are created here or have
        // already passed `validate_wire_state` during Deserialize.
        Lineage::from_snapshot(self.lineage.clone())
    }
}

fn visit_ordinal<P: ScopeValue, A: ScopeValue>(
    memory: &Lineage<P, A>,
    wanted: VisitId,
) -> Option<usize> {
    memory.visits().position(|(visit_id, _)| visit_id == wanted)
}

fn canonical_snapshot<P: ScopeValue, A: ScopeValue>(
    mut snapshot: Snapshot<P, A>,
) -> Snapshot<P, A> {
    for visit in &mut snapshot.visits {
        visit.bindings.sort_by_key(|binding| binding.owner);
    }
    for owner in &mut snapshot.owners {
        owner.owned_visits.sort_unstable();
    }
    snapshot
}

#[derive(Serialize)]
struct WireRef<'a, P: ScopeValue, A: ScopeValue> {
    version: u8,
    lineage: &'a Snapshot<P, A>,
    bookmarks: Vec<IndexedBookmarkRef<'a, P, A>>,
    remembered: &'a [ScopeVisit<P, A>],
}

#[derive(Serialize)]
struct IndexedBookmarkRef<'a, P: ScopeValue, A: ScopeValue> {
    visit: usize,
    value: &'a ScopeVisit<P, A>,
}

#[derive(Deserialize)]
#[serde(bound = "")]
struct Wire<P: ScopeValue, A: ScopeValue> {
    version: u8,
    lineage: Snapshot<P, A>,
    bookmarks: Vec<IndexedBookmark<P, A>>,
    remembered: Vec<ScopeVisit<P, A>>,
}

#[derive(Deserialize)]
#[serde(bound = "")]
struct IndexedBookmark<P: ScopeValue, A: ScopeValue> {
    visit: usize,
    value: ScopeVisit<P, A>,
}

#[derive(Deserialize)]
#[serde(bound = "")]
struct LegacyWire<P: ScopeValue, A: ScopeValue> {
    entries: Vec<ScopeVisit<P, A>>,
    cursor: usize,
    #[serde(default)]
    remembered: Vec<ScopeVisit<P, A>>,
}

fn validate_scope_visit<P: ScopeValue, A: ScopeValue>(
    visit: &ScopeVisit<P, A>,
) -> Result<(), String> {
    if visit.camera.is_some_and(|camera| !camera.valid()) {
        return Err("Invalid scope navigation camera".into());
    }
    Ok(())
}

fn validate_wire_state<P: ScopeValue, A: ScopeValue>(
    snapshot: &Snapshot<P, A>,
    bookmarks: &[ScopeVisit<P, A>],
    remembered: &[ScopeVisit<P, A>],
) -> Result<(), String> {
    let fail = |reason: &str| Err(format!("Invalid scope navigation snapshot: {reason}"));
    let entries = snapshot.entries.len();
    let visits = snapshot.visits.len();
    if visits > MAX_RESTORED_VISITS || entries > visits || remembered.len() > MAX_REMEMBERED {
        return fail("retained item limit exceeded");
    }
    if bookmarks.len() != visits {
        return fail("bookmark count mismatch");
    }
    if snapshot.owners.len() != 1 {
        return fail("expected one contextual-navigation owner");
    }
    for (index, visit) in remembered.iter().enumerate() {
        if remembered[..index]
            .iter()
            .any(|earlier| earlier.place == visit.place)
        {
            return fail("duplicate remembered place");
        }
    }
    let owner = &snapshot.owners[0];
    if owner.identity != OWNER
        || owner.creator.is_some()
        || owner.pending_origin_parent.is_some()
        || (visits == 0 && (owner.origin.is_some() || owner.current.is_some()))
        || (visits > 0 && owner.origin != Some(0))
        || (visits > 0 && owner.current.is_none())
        || owner.current.is_some_and(|current| current >= visits)
    {
        return fail("invalid owner identity or cursor");
    }
    let mut owned = owner.owned_visits.clone();
    owned.sort_unstable();
    if owned != (0..visits).collect::<Vec<_>>() {
        return fail("owner visit membership mismatch");
    }
    let mut entry_counts = vec![0u64; entries];
    for (index, entry) in snapshot.entries.iter().enumerate() {
        if entry.key != index as u64 {
            return fail("noncanonical entry key");
        }
    }
    let mut parents = vec![None; visits];
    for (index, visit) in snapshot.visits.iter().enumerate() {
        if visit.entry >= entries
            || visit.parent.is_some_and(|parent| parent >= index)
            || visit
                .children
                .iter()
                .any(|&child| child >= visits || child <= index)
            || visit.bindings.len() != 1
            || visit.bindings[0].owner != 0
            || visit.bindings.iter().any(|binding| {
                binding
                    .forward_child
                    .is_some_and(|child| !visit.children.contains(&child))
            })
        {
            return fail("visit reference or owner binding invalid");
        }
        if let Some(parent) = visit.parent {
            if parents[index].replace(parent).is_some() {
                return fail("visit has multiple parents");
            }
        }
        entry_counts[visit.entry] += 1;
        validate_scope_visit(&visit.context)?;
        validate_scope_visit(&bookmarks[index])?;
    }
    let mut child_seen = vec![false; visits];
    for (index, visit) in snapshot.visits.iter().enumerate() {
        let mut local = std::collections::HashSet::new();
        for &child in &visit.children {
            if !local.insert(child) || child_seen[child] || parents[child] != Some(index) {
                return fail("inconsistent parent-child links");
            }
            child_seen[child] = true;
        }
        if visit.parent.is_some() && !child_seen[index] {
            // A child can appear later in the serialized visit order, so this
            // check is completed below after the full parent pass.
        }
    }
    for (index, visit) in snapshot.visits.iter().enumerate() {
        if visit.parent.is_some() && !child_seen[index] {
            return fail("parent-child links are not reciprocal");
        }
    }
    for (entry, count) in snapshot.entries.iter().zip(entry_counts) {
        if entry.visit_count != count {
            return fail("entry visit count mismatch");
        }
    }
    if visits == 0 && !bookmarks.is_empty() {
        return fail("bookmarks without visits");
    }
    for visit in remembered {
        validate_scope_visit(visit)?;
    }

    // Parent indices must decrease, which already excludes visit cycles. Still
    // verify cursor ownership and each entry's immutable place identity.
    // Mutable bookmarks may carry a later focus in the same host-owned route.
    let mut entry_places: Vec<Option<&P>> = vec![None; entries];
    for visit in &snapshot.visits {
        match entry_places[visit.entry] {
            Some(place) if place != &visit.context.place => {
                return fail("one entry refers to multiple places");
            },
            None => entry_places[visit.entry] = Some(&visit.context.place),
            _ => {},
        }
    }
    if entry_places.iter().any(Option::is_none) {
        return fail("unvisited entry present");
    }
    for (index, visit) in snapshot.visits.iter().enumerate() {
        if visit.parent.is_none() != (index == 0) {
            return fail("lineage root mismatch");
        }
    }
    if owner.current.is_some_and(|current| current >= visits) {
        return fail("current visit out of bounds");
    }
    Ok(())
}

#[cfg(test)]
#[path = "history/tests.rs"]
mod tests;

#[path = "history/serialization.rs"]
mod serialization;
