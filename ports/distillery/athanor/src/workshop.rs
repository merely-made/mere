// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Persistent, owner-scoped evidence briefs. The owner supplies retained bytes,
//! current authority and time. Servitor alone reduces the recorded run events;
//! this module supplies neither a scheduler nor destination-write authority.
//!
//! A returned dispatch is recorded intent, not dispatch permission. After each
//! await the embedding owner must recheck its source/custody gate before handing
//! the captured bytes to a worker. Destination adapters must atomically join
//! the acceptance operation ledger to their actual product mutation and return
//! a durable receipt. This module deliberately cannot turn a handoff into one.

use alembic::workshop::*;
use muniment::{Backend, WriteOp};
use serde::{Deserialize, Serialize};
use servitor::{AuthorityProvider, Cap, Mode, RunEvent, RunPhase, RunReducer};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

const SCHEMA: u32 = 1;
const MAX_RECORD: usize = 1024 * 1024;
const MAX_RUNS: usize = 32;
const DEADLINE: u64 = 30_000;
pub const LIMITS: Limits = Limits {
    decisions: 3,
    tool_calls: 2,
    tokens: 0,
    elapsed_ms: DEADLINE,
    consecutive_failures: 0,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Invalid(&'static str),
    Unauthorized,
    Inactive,
    ActiveRun(u64),
    Missing,
    Conflict,
    Budget,
    Review,
    Stale,
    Store(String),
    Run(servitor::RunError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "evidence brief: {self:?}")
    }
}
impl std::error::Error for Error {}
impl From<servitor::RunError> for Error {
    fn from(value: servitor::RunError) -> Self {
        Self::Run(value)
    }
}

fn hash(bytes: &[u8]) -> Digest {
    *blake3::hash(bytes).as_bytes()
}
fn encoded<T: Serialize>(value: &T) -> Result<Vec<u8>, Error> {
    serde_json::to_vec(value).map_err(|_| Error::Invalid("record encoding"))
}
fn digest<T: Serialize>(value: &T) -> Digest {
    hash(&serde_json::to_vec(value).expect("bounded concrete record serializes"))
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn definition() -> Digest {
    hash(b"mere.alembic.evidence-brief.v1/retained-djot/three-steps")
}

/// The adapter must first authorize this exact retained version and classify
/// its bytes. This function checks UTF-8 selectors, not the owner's authority.
pub fn capture(mut source: Source, retained: &str, public: bool) -> Result<Passage, Error> {
    if !public {
        return Err(Error::Invalid("private storage is not qualified"));
    }
    let start = usize::try_from(source.start).map_err(|_| Error::Invalid("selector"))?;
    let end = usize::try_from(source.end).map_err(|_| Error::Invalid("selector"))?;
    let text = retained
        .get(start..end)
        .ok_or(Error::Invalid("UTF-8 selector"))?;
    if text.is_empty() || text.len() > 2048 {
        return Err(Error::Invalid("passage size"));
    }
    source.content_digest = hash(retained.as_bytes());
    Ok(Passage {
        source,
        text: text.into(),
        text_digest: hash(text.as_bytes()),
        public,
    })
}

fn owner_valid(owner: &Owner) -> bool {
    [&owner.host, &owner.workspace, &owner.session]
        .iter()
        .all(|part| !part.is_empty() && part.len() <= 256)
}

fn validate(owner: &Owner, passages: &[Passage]) -> Result<(), Error> {
    if !owner_valid(owner) || !(2..=4).contains(&passages.len()) {
        return Err(Error::Invalid("owner or selection count"));
    }
    let mut metadata = 0;
    let mut bytes = 0;
    for passage in passages {
        let source = &passage.source;
        if !passage.public {
            return Err(Error::Invalid("private storage is not qualified"));
        }
        if &source.owner != owner {
            return Err(Error::Invalid("source owner/session"));
        }
        if source.resource.is_empty()
            || source.version.is_empty()
            || source.title.is_empty()
            || source.end.checked_sub(source.start) != Some(passage.text.len() as u64)
            || passage.text.is_empty()
            || passage.text.len() > 2048
            || hash(passage.text.as_bytes()) != passage.text_digest
        {
            return Err(Error::Invalid("passage identity, selector or digest"));
        }
        metadata += encoded(source)?.len();
        bytes += passage.text.len();
    }
    if metadata > 4096 || bytes > 8192 {
        return Err(Error::Invalid("selection bounds"));
    }
    Ok(())
}

/// Versioned semantic operation scopes. Their hashes identify exact arguments;
/// they do not grant themselves. The host supplies the AuthorityProvider.
pub fn read_cap(source: &Source) -> Cap {
    Cap::scope(&format!(
        "alembic/v1/{}/sources/{}",
        hex(&digest(&source.owner)),
        hex(&digest(source))
    ))
    .expect("hex scope is valid")
}
pub fn proposal_cap(owner: &Owner, installation: [u8; 16]) -> Cap {
    Cap::scope(&format!(
        "alembic/v1/{}/installations/{}/proposals",
        hex(&digest(owner)),
        hex(&installation)
    ))
    .expect("hex scope is valid")
}
pub fn destination_cap(destination: &Destination) -> Cap {
    Cap::scope(&format!(
        "alembic/v1/{}/graphs/{}",
        hex(&digest(&destination.owner)),
        hex(&digest(&destination.graph))
    ))
    .expect("hex scope is valid")
}

/// Necessary checks immediately before the owner dispatches acceptance,
/// including after persistence awaits. The destination still checks its live
/// graph revision and custody policy and owns the atomic operation ledger.
/// Expiry never hides an already committed receipt; reconciliation may query
/// that ledger after the dispatch deadline.
pub fn revalidate_acceptance(
    acceptance: &Acceptance,
    human: [u8; 32],
    authority: &impl AuthorityProvider,
    now: u64,
) -> Result<(), Error> {
    if human != acceptance.human
        || !authority.covers(
            servitor::Subject(human),
            &destination_cap(&acceptance.destination),
            Mode::Write,
        )
    {
        return Err(Error::Unauthorized);
    }
    if now < acceptance.requested_at_ms || now >= acceptance.deadline_at_ms {
        return Err(Error::Budget);
    }
    Ok(())
}
fn requirements(owner: &Owner, id: [u8; 16], sources: &[Passage]) -> Vec<(Cap, Mode)> {
    let mut required: Vec<_> = sources
        .iter()
        .map(|p| (read_cap(&p.source), Mode::Read))
        .collect();
    required.push((proposal_cap(owner, id), Mode::Write));
    required
}

fn title_text(text: &str) -> String {
    // Preserve labels as literal inline text, including newlines and punctuation.
    text.chars()
        .flat_map(|c| {
            if c == '\n' || c == '\r' {
                vec![' ']
            } else if c.is_ascii_punctuation() {
                vec!['\\', c]
            } else {
                vec![c]
            }
        })
        .collect()
}

fn proposal(id: [u8; 16], run: u64, sources: &[Passage]) -> Result<Proposal, Error> {
    let mut djot = String::from("# Evidence brief\n\n");
    for (index, passage) in sources.iter().enumerate() {
        djot.push_str(&format!(
            "## {}. {}\n\n",
            index + 1,
            title_text(&passage.source.title)
        ));
        // A literal code block inside the quotation prevents retained Djot from
        // injecting links/structure; a longer fence handles embedded backticks.
        let fence = "`".repeat(
            3.max(
                passage
                    .text
                    .split(|c| c != '`')
                    .map(str::len)
                    .max()
                    .unwrap_or(0)
                    + 1,
            ),
        );
        djot.push_str(&format!("> {fence}\n"));
        for line in passage.text.split('\n') {
            djot.push_str(&format!("> {line}\n"));
        }
        djot.push_str(&format!(
            "> {fence}\n\n[Open retained source {}](alembic-source:{})\n\n",
            index + 1,
            hex(&digest(&passage.source))
        ));
    }
    if djot.len() > 16_384 {
        return Err(Error::Invalid("brief size"));
    }
    let mut value = Proposal {
        schema: SCHEMA,
        definition: definition(),
        installation: id,
        run,
        sources: sources.to_vec(),
        djot,
        digest: [0; 32],
    };
    value.digest = digest(&value);
    Ok(value)
}

/// Resolve only references contained in this immutable proposal. The source
/// owner must authorize and open this exact version; there is no latest-version
/// fallback and this function never acquires content.
pub fn citation_source(proposed: &Proposal, reference: &str) -> Result<Source, Error> {
    if proposed != &proposal(proposed.installation, proposed.run, &proposed.sources)? {
        return Err(Error::Invalid("proposal integrity"));
    }
    let token = reference
        .strip_prefix("alembic-source:")
        .ok_or(Error::Invalid("citation route"))?;
    proposed
        .sources
        .iter()
        .find(|p| hex(&digest(&p.source)) == token)
        .map(|p| p.source.clone())
        .ok_or(Error::Invalid("citation route"))
}

/// Validate the exact immutable public proposal before a product owner admits
/// its acceptance. Integrity does not supply read or destination authority.
pub fn validate_proposal(proposed: &Proposal) -> Result<(), Error> {
    let owner = &proposed
        .sources
        .first()
        .ok_or(Error::Invalid("selection"))?
        .source
        .owner;
    validate(owner, &proposed.sources)?;
    if proposed != &proposal(proposed.installation, proposed.run, &proposed.sources)? {
        return Err(Error::Invalid("proposal integrity"));
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum Lifecycle {
    Active,
    Paused,
    Revoked,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Binding {
    id: [u8; 16],
    subject: [u8; 32],
    revision: Digest,
    generation: u64,
    lifecycle: Lifecycle,
}
impl Binding {
    fn servitor(&self) -> servitor::ResidentBinding {
        servitor::ResidentBinding {
            id: servitor::ResidentId(self.id),
            subject: servitor::Subject(self.subject),
            revision: servitor::BodyRevision(self.revision),
            generation: self.generation,
            lifecycle: match self.lifecycle {
                Lifecycle::Active => servitor::Lifecycle::Active,
                Lifecycle::Paused => servitor::Lifecycle::Paused,
                Lifecycle::Revoked => servitor::Lifecycle::Revoked,
            },
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum Entry {
    Intent {
        correlation: Correlation,
        at: u64,
    },
    Result {
        correlation: Correlation,
        at: u64,
        completed: bool,
    },
    Interrupted(u64),
    Paused(u64),
    Resumed(u64),
    Cancelled(u64),
    Budget(u64),
}
fn correlation(c: Correlation) -> servitor::Correlation {
    servitor::Correlation {
        run_id: servitor::RunId(c.run),
        step: c.step,
        attempt: c.attempt,
        generation: c.generation,
    }
}
fn wire(c: servitor::Correlation) -> Correlation {
    Correlation {
        run: c.run_id.0,
        step: c.step,
        attempt: c.attempt,
        generation: c.generation,
    }
}
fn tool_calls(step: u64) -> u64 {
    u64::from(step != 1)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Run {
    id: u64,
    admitted: Binding,
    started: u64,
    limits: Limits,
    inputs: Vec<Passage>,
    entries: Vec<Entry>,
    candidate: Option<Proposal>,
    proposal: Option<Proposal>,
    review: Review,
}
impl Run {
    fn replay(&self, owner: &Owner) -> Result<RunReducer, Error> {
        let mut reducer = RunReducer::new(servitor::RunHeader {
            id: servitor::RunId(self.id),
            started_at_ms: self.started,
            limits: servitor::RunLimits {
                decisions: self.limits.decisions,
                tool_calls: self.limits.tool_calls,
                tokens: self.limits.tokens,
                elapsed_ms: self.limits.elapsed_ms,
                consecutive_failures: self.limits.consecutive_failures,
            },
            ticket: servitor::RunTicket {
                binding: self.admitted.servitor(),
                trigger: servitor::Trigger::Manual,
                required: requirements(owner, self.admitted.id, &self.inputs),
            },
        })?;
        for entry in &self.entries {
            let event = match *entry {
                Entry::Intent { correlation: c, at } => RunEvent::IntentRecorded {
                    correlation: correlation(c),
                    effect: servitor::Effect {
                        kind: servitor::EffectKind::ReadOnly,
                        operation: c.step,
                    },
                    usage: servitor::Usage {
                        decisions: 1,
                        tool_calls: tool_calls(c.step),
                        elapsed_ms: at.checked_sub(reducer.last_at_ms()).ok_or(Error::Stale)?,
                        ..Default::default()
                    },
                    at_ms: at,
                },
                Entry::Result {
                    correlation: c,
                    at,
                    completed,
                } => RunEvent::ResultRecorded {
                    correlation: correlation(c),
                    result: if completed {
                        servitor::ResultKind::Completed
                    } else {
                        servitor::ResultKind::Progress
                    },
                    usage: servitor::Usage {
                        elapsed_ms: at.checked_sub(reducer.last_at_ms()).ok_or(Error::Stale)?,
                        ..Default::default()
                    },
                    at_ms: at,
                },
                Entry::Interrupted(at) => RunEvent::Interrupted { at_ms: at },
                Entry::Paused(at) => RunEvent::Paused { at_ms: at },
                Entry::Resumed(at) => RunEvent::Resumed { at_ms: at },
                Entry::Cancelled(at) => RunEvent::CancellationRequested { at_ms: at },
                Entry::Budget(at) => RunEvent::BudgetExhausted { at_ms: at },
            };
            reducer.apply(event)?;
        }
        Ok(reducer)
    }
    fn push(&mut self, owner: &Owner, entry: Entry) -> Result<(), Error> {
        self.entries.push(entry);
        self.replay(owner)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    schema: u32,
    owner: Owner,
    binding: Binding,
    dissolved: bool,
    runs: Vec<Run>,
}

impl Record {
    fn projected_size(&self) -> Result<usize, Error> {
        let mut size = encoded(self)?.len();
        for run in &self.runs {
            let active = !matches!(run.replay(&self.owner)?.phase(), RunPhase::Terminal(_));
            if active {
                let proposal_size =
                    encoded(&proposal(self.binding.id, run.id, &run.inputs)?)?.len();
                if run.candidate.is_none() {
                    size += proposal_size;
                }
                if run.proposal.is_none() {
                    size += proposal_size;
                }
            }
            // Reserve the bounded acceptance/receipt envelope while review is
            // unresolved. Filling the store with new work cannot consume the
            // space needed to record an existing operation's resolution.
            if active
                || matches!(
                    run.review,
                    Review::AwaitingReview | Review::Applying(_) | Review::Reconciling(_)
                )
            {
                size += 16_384usize.saturating_sub(encoded(&run.review)?.len());
            }
        }
        Ok(size)
    }
}

/// One installation in one owner's transactional store. Reopening this handle
/// reads persisted state; no in-memory cache is an authority or a durability ack.
pub struct Workshop<B> {
    backend: B,
    owner: Owner,
    id: [u8; 16],
    key: String,
}

/// Native transactional futures require a shared backend; browser backends
/// retain Muniment's `?Send` contract and stay on their owning event loop.
#[cfg(not(target_arch = "wasm32"))]
pub trait WorkshopBackend: Backend + Sync {}
#[cfg(not(target_arch = "wasm32"))]
impl<B: Backend + Sync> WorkshopBackend for B {}
#[cfg(target_arch = "wasm32")]
pub trait WorkshopBackend: Backend {}
#[cfg(target_arch = "wasm32")]
impl<B: Backend> WorkshopBackend for B {}

impl<B: WorkshopBackend> Workshop<B> {
    /// Refuse a caller that names a different subject for this installation.
    pub async fn validate_installed_subject(&self, subject: [u8; 32]) -> Result<(), Error> {
        let (_, record) = self.load().await?;
        if record.binding.subject != subject {
            return Err(Error::Unauthorized);
        }
        Ok(())
    }

    /// Return the installed run's bounded captured inputs to its owning driver.
    /// This does not authorize source disclosure; the caller rechecks its gate.
    pub async fn run_inputs(&self, id: u64) -> Result<Vec<Passage>, Error> {
        let (_, record) = self.load().await?;
        Ok(record
            .runs
            .iter()
            .find(|r| r.id == id)
            .ok_or(Error::Missing)?
            .inputs
            .clone())
    }

    /// Recheck the binding and authority after intent persistence and before
    /// handing captured bytes to the worker. This observes no new effect.
    pub async fn dispatch_authorized(
        &self,
        dispatch: &Dispatch,
        authority: &impl AuthorityProvider,
    ) -> Result<bool, Error> {
        let (_, record) = self.load().await?;
        if record.dissolved {
            return Ok(false);
        }
        let run = record
            .runs
            .iter()
            .find(|r| r.id == dispatch.correlation.run)
            .ok_or(Error::Missing)?;
        let reducer = run.replay(&self.owner)?;
        let issued = run.entries.iter().rev().find_map(|entry| match entry {
            Entry::Intent { correlation, at } if *correlation == dispatch.correlation => Some(*at),
            _ => None,
        });
        Ok(
            matches!(reducer.phase(), RunPhase::Awaiting { correlation: c, .. } if c == correlation(dispatch.correlation))
                && dispatch.installation == self.id
                && dispatch.inputs == run.inputs
                && dispatch.candidate == run.candidate
                && Some(dispatch.issued_at_ms) == issued
                && run.started.checked_add(run.limits.elapsed_ms) == Some(dispatch.deadline_at_ms)
                && servitor::revalidate(
                    reducer.ticket(),
                    &record.binding.servitor(),
                    authority,
                    &[],
                )
                .is_ok(),
        )
    }
    pub fn new(backend: B, owner: Owner, id: [u8; 16]) -> Self {
        let key = format!("alembic/workshop/v1/{}/{}", hex(&digest(&owner)), hex(&id));
        Self {
            backend,
            owner,
            id,
            key,
        }
    }

    async fn load(&self) -> Result<(Vec<u8>, Record), Error> {
        let bytes = self
            .backend
            .get(&self.key)
            .await
            .map_err(|e| Error::Store(e.to_string()))?
            .ok_or(Error::Missing)?;
        if bytes.len() > MAX_RECORD {
            return Err(Error::Invalid("record size"));
        }
        let record: Record =
            serde_json::from_slice(&bytes).map_err(|_| Error::Invalid("record schema"))?;
        if record.schema != SCHEMA
            || record.owner != self.owner
            || record.binding.id != self.id
            || record.binding.revision != definition()
            || record.runs.len() > MAX_RUNS
        {
            return Err(Error::Invalid("installation schema or identity"));
        }
        for run in &record.runs {
            if run.admitted.id != self.id || run.admitted.revision != definition() {
                return Err(Error::Invalid("selected definition"));
            }
            validate(&self.owner, &run.inputs)?;
            if run.limits != LIMITS {
                return Err(Error::Invalid("run limits"));
            }
            let reducer = run.replay(&self.owner)?;
            let expected = proposal(self.id, run.id, &run.inputs)?;
            if run.candidate.as_ref().is_some_and(|p| p != &expected)
                || run.proposal.as_ref().is_some_and(|p| p != &expected)
            {
                return Err(Error::Invalid("persisted proposal integrity"));
            }
            let completed =
                reducer.phase() == RunPhase::Terminal(servitor::TerminalOutcome::Completed);
            if completed != run.proposal.is_some()
                || completed == (run.review == Review::Generating)
            {
                return Err(Error::Invalid("proposal disposition"));
            }
            let (acceptance, receipt) = match &run.review {
                Review::Applying(a) | Review::Reconciling(a) => (Some(a), None),
                Review::Added {
                    acceptance,
                    receipt,
                } => (Some(acceptance), Some(receipt)),
                _ => (None, None),
            };
            if let Some(a) = acceptance {
                if a.operation
                    != digest(&(
                        "alembic-acceptance-v1",
                        &self.owner,
                        self.id,
                        run.id,
                        expected.digest,
                    ))
                    || a.installation != self.id
                    || a.run != run.id
                    || a.proposal != expected.digest
                    || a.proposing_subject != run.admitted.subject
                    || a.destination.owner != self.owner
                    || a.destination.graph.is_empty()
                    || a.destination.revision.is_empty()
                    || a.deadline_at_ms.checked_sub(a.requested_at_ms) != Some(DEADLINE)
                {
                    return Err(Error::Invalid("acceptance integrity"));
                }
                if let Some(r) = receipt {
                    if r.operation != a.operation
                        || r.proposal != a.proposal
                        || r.destination != a.destination
                        || r.resource.is_empty()
                        || r.storage_witness.is_empty()
                    {
                        return Err(Error::Invalid("receipt integrity"));
                    }
                }
            }
        }
        Ok((bytes, record))
    }

    async fn save(&self, before: Option<Vec<u8>>, record: Record) -> Result<(), Error> {
        let value = encoded(&record)?;
        if value.len() > MAX_RECORD || record.projected_size()? > MAX_RECORD {
            return Err(Error::Invalid("record retention bound"));
        }
        let key = self.key.clone();
        let committed = Arc::new(AtomicBool::new(false));
        let flag = committed.clone();
        self.backend
            .transact(Box::new(move |reader| {
                // Read errors refuse the comparison. They never select an absent
                // record or fall back to a nontransactional write.
                if reader.get(&key).ok().as_ref() != Some(&before) {
                    return vec![];
                }
                flag.store(true, Ordering::Release);
                vec![WriteOp::Put { key, value }]
            }))
            .await
            .map_err(|e| Error::Store(e.to_string()))?;
        if committed.load(Ordering::Acquire) {
            Ok(())
        } else {
            Err(Error::Conflict)
        }
    }

    pub async fn install(&self, subject: [u8; 32]) -> Result<(), Error> {
        if !owner_valid(&self.owner) {
            return Err(Error::Invalid("owner"));
        }
        self.save(
            None,
            Record {
                schema: SCHEMA,
                owner: self.owner.clone(),
                binding: Binding {
                    id: self.id,
                    subject,
                    revision: definition(),
                    generation: 1,
                    lifecycle: Lifecycle::Active,
                },
                dissolved: false,
                runs: vec![],
            },
        )
        .await
    }

    pub async fn begin(
        &self,
        id: u64,
        inputs: Vec<Passage>,
        authority: &impl AuthorityProvider,
        now: u64,
    ) -> Result<(), Error> {
        validate(&self.owner, &inputs)?;
        // Check the prospective artifact before installing an actionable run;
        // an unrenderable selection must not poison its persisted record.
        proposal(self.id, id, &inputs)?;
        let (before, mut record) = self.load().await?;
        if record.dissolved {
            return Err(Error::Inactive);
        }
        if record.runs.len() == MAX_RUNS {
            return Err(Error::Invalid("run retention bound"));
        }
        for run in &record.runs {
            if run.id == id {
                return Err(Error::Invalid("run ID reuse"));
            }
            if !matches!(run.replay(&self.owner)?.phase(), RunPhase::Terminal(_)) {
                return Err(Error::ActiveRun(run.id));
            }
        }
        servitor::admit(
            &record.binding.servitor(),
            servitor::Trigger::Manual,
            authority,
            &requirements(&self.owner, self.id, &inputs),
        )
        .map_err(|_| Error::Unauthorized)?;
        record.runs.push(Run {
            id,
            admitted: record.binding.clone(),
            started: now,
            limits: LIMITS,
            inputs,
            entries: vec![],
            candidate: None,
            proposal: None,
            review: Review::Generating,
        });
        self.save(Some(before), record).await
    }

    /// Persist one accepted intent before the owner can dispatch its bytes.
    /// Source availability/custody still belongs to the owner's action gate.
    pub async fn dispatch(
        &self,
        id: u64,
        authority: &impl AuthorityProvider,
        now: u64,
    ) -> Result<Dispatch, Error> {
        let (before, mut record) = self.load().await?;
        let binding = record.binding.servitor();
        if record.dissolved {
            return Err(Error::Inactive);
        }
        let run = record
            .runs
            .iter_mut()
            .find(|r| r.id == id)
            .ok_or(Error::Missing)?;
        let reducer = run.replay(&self.owner)?;
        servitor::revalidate(reducer.ticket(), &binding, authority, &[])
            .map_err(|_| Error::Unauthorized)?;
        let RunPhase::Ready { correlation: c } = reducer.phase() else {
            return Err(Error::Stale);
        };
        if now.checked_sub(run.started).ok_or(Error::Stale)? >= DEADLINE
            || reducer.usage().decisions >= 3
            || reducer.usage().tool_calls + tool_calls(c.step) > 2
        {
            run.push(&self.owner, Entry::Budget(now))?;
            self.save(Some(before), record).await?;
            return Err(Error::Budget);
        }
        if c.step > 2 {
            return Err(Error::Invalid("workflow step"));
        }
        run.push(
            &self.owner,
            Entry::Intent {
                correlation: wire(c),
                at: now,
            },
        )?;
        let dispatch = Dispatch {
            correlation: wire(c),
            installation: self.id,
            issued_at_ms: now,
            deadline_at_ms: run
                .started
                .checked_add(run.limits.elapsed_ms)
                .ok_or(Error::Invalid("run deadline overflow"))?,
            inputs: run.inputs.clone(),
            candidate: run.candidate.clone(),
        };
        self.save(Some(before), record).await?;
        Ok(dispatch)
    }

    /// Revalidate after worker awaits, then atomically persist the proposal and
    /// Completed event. Wrong-generation, duplicate and delayed observations
    /// cannot change a terminal run or a rejected proposal.
    pub async fn observe(
        &self,
        observation: Observation,
        authority: &impl AuthorityProvider,
        now: u64,
    ) -> Result<(), Error> {
        self.observe_guarded(observation, authority, &|| now, &|_| true)
            .await
    }

    /// Recheck the exact stored source selection and sample time after loading
    /// the run. The owner serializes policy changes with the subsequent write.
    pub async fn observe_guarded(
        &self,
        observation: Observation,
        authority: &impl AuthorityProvider,
        clock: &impl Fn() -> u64,
        source_gate: &impl Fn(&Passage) -> bool,
    ) -> Result<(), Error> {
        let (before, mut record) = self.load().await?;
        let binding = record.binding.servitor();
        if record.dissolved {
            return Err(Error::Inactive);
        }
        let c = observation.correlation;
        let run = record
            .runs
            .iter_mut()
            .find(|r| r.id == c.run)
            .ok_or(Error::Missing)?;
        if !run.inputs.iter().all(source_gate) {
            return Err(Error::Unauthorized);
        }
        let now = clock();
        let reducer = run.replay(&self.owner)?;
        servitor::revalidate(reducer.ticket(), &binding, authority, &[])
            .map_err(|_| Error::Unauthorized)?;
        let RunPhase::Awaiting {
            correlation: expected,
            ..
        } = reducer.phase()
        else {
            return Err(Error::Stale);
        };
        if correlation(c) != expected {
            return Err(Error::Stale);
        }
        if now.checked_sub(run.started).ok_or(Error::Stale)? >= DEADLINE {
            run.push(
                &self.owner,
                Entry::Result {
                    correlation: c,
                    at: now,
                    completed: false,
                },
            )?;
            if !matches!(run.replay(&self.owner)?.phase(), RunPhase::Terminal(_)) {
                run.push(&self.owner, Entry::Budget(now))?;
            }
            self.save(Some(before), record).await?;
            return Err(Error::Budget);
        }
        let wanted = if c.step == 0 {
            None
        } else {
            Some(proposal(self.id, run.id, &run.inputs)?)
        };
        if observation.proposal != wanted {
            return Err(Error::Invalid("worker proposal"));
        }
        if c.step == 1 {
            run.candidate = wanted;
        } else if c.step == 2 {
            run.proposal = wanted;
            run.review = Review::AwaitingReview;
        }
        run.push(
            &self.owner,
            Entry::Result {
                correlation: c,
                at: now,
                completed: c.step == 2,
            },
        )?;
        self.save(Some(before), record).await
    }

    /// Explicit recovery records lost read-only work; it never executes it.
    /// A resumed run is still constrained by the original decision/time budget.
    pub async fn interrupt(&self, id: u64, now: u64) -> Result<(), Error> {
        let (before, mut record) = self.load().await?;
        record
            .runs
            .iter_mut()
            .find(|r| r.id == id)
            .ok_or(Error::Missing)?
            .push(&self.owner, Entry::Interrupted(now))?;
        self.save(Some(before), record).await
    }

    /// Pause/revoke/dissolve are owner lifecycle decisions, persisted before
    /// cancelling/joining the separate worker. They never roll back a brief.
    pub async fn lifecycle(
        &self,
        control: Control,
        authority: &impl AuthorityProvider,
        now: u64,
    ) -> Result<(), Error> {
        let (before, mut record) = self.load().await?;
        if record.dissolved {
            return Err(Error::Inactive);
        }
        if record.binding.lifecycle == Lifecycle::Revoked {
            if control != Control::Dissolve {
                return Err(Error::Inactive);
            }
            record.dissolved = true;
            return self.save(Some(before), record).await;
        }
        for run in &mut record.runs {
            let mut phase = run.replay(&self.owner)?.phase();
            if matches!(phase, RunPhase::Terminal(_)) {
                continue;
            }
            match control {
                Control::Pause => {
                    if matches!(phase, RunPhase::Awaiting { .. }) {
                        run.push(&self.owner, Entry::Interrupted(now))?;
                        phase = run.replay(&self.owner)?.phase();
                    }
                    if matches!(phase, RunPhase::Ready { .. }) {
                        run.push(&self.owner, Entry::Paused(now))?;
                    }
                },
                Control::Resume => {
                    let mut active = record.binding.clone();
                    active.lifecycle = Lifecycle::Active;
                    servitor::revalidate(
                        run.replay(&self.owner)?.ticket(),
                        &active.servitor(),
                        authority,
                        &[],
                    )
                    .map_err(|_| Error::Unauthorized)?;
                    run.push(&self.owner, Entry::Resumed(now))?;
                },
                Control::Revoke | Control::Dissolve => {
                    run.push(&self.owner, Entry::Cancelled(now))?
                },
                _ => return Err(Error::Invalid("lifecycle control")),
            }
        }
        record.binding.lifecycle = match control {
            Control::Pause => Lifecycle::Paused,
            Control::Resume => Lifecycle::Active,
            Control::Revoke | Control::Dissolve => Lifecycle::Revoked,
            _ => return Err(Error::Invalid("lifecycle control")),
        };
        if matches!(control, Control::Revoke | Control::Dissolve) {
            record.binding.generation = record
                .binding
                .generation
                .checked_add(1)
                .ok_or(Error::Invalid("generation overflow"))?;
        }
        record.dissolved = control == Control::Dissolve;
        self.save(Some(before), record).await
    }

    pub async fn reject(&self, id: u64) -> Result<(), Error> {
        let (before, mut record) = self.load().await?;
        let run = record
            .runs
            .iter_mut()
            .find(|r| r.id == id)
            .ok_or(Error::Missing)?;
        if run.review != Review::AwaitingReview {
            return Err(Error::Review);
        }
        run.review = Review::Rejected;
        self.save(Some(before), record).await
    }

    /// Assign one operation/destination. A concurrent duplicate may report
    /// Conflict; reloading/repeating returns the winner's identical assignment.
    /// This method performs no destination effect.
    pub async fn accept(
        &self,
        id: u64,
        destination: Destination,
        human: [u8; 32],
        authority: &impl AuthorityProvider,
        now: u64,
    ) -> Result<Acceptance, Error> {
        let (before, mut record) = self.load().await?;
        if record.dissolved || record.binding.lifecycle != Lifecycle::Active {
            return Err(Error::Inactive);
        }
        if destination.owner != self.owner
            || destination.graph.is_empty()
            || destination.revision.is_empty()
            || encoded(&destination)?.len() > 4096
        {
            return Err(Error::Invalid("destination"));
        }
        if !authority.covers(
            servitor::Subject(human),
            &destination_cap(&destination),
            Mode::Write,
        ) {
            return Err(Error::Unauthorized);
        }
        let run = record
            .runs
            .iter_mut()
            .find(|r| r.id == id)
            .ok_or(Error::Missing)?;
        let proposal = run.proposal.as_ref().ok_or(Error::Review)?;
        match &run.review {
            Review::Applying(old)
            | Review::Reconciling(old)
            | Review::Added {
                acceptance: old, ..
            } => {
                if old.destination != destination || old.human != human {
                    return Err(Error::Review);
                }
                return Ok(old.clone());
            },
            Review::AwaitingReview => {},
            _ => return Err(Error::Review),
        }
        if now < run.replay(&self.owner)?.last_at_ms() {
            return Err(Error::Stale);
        }
        let acceptance = Acceptance {
            operation: digest(&(
                "alembic-acceptance-v1",
                &self.owner,
                self.id,
                id,
                proposal.digest,
            )),
            proposal: proposal.digest,
            destination,
            human,
            installation: self.id,
            run: id,
            proposing_subject: run.admitted.subject,
            requested_at_ms: now,
            deadline_at_ms: now
                .checked_add(DEADLINE)
                .ok_or(Error::Invalid("acceptance deadline overflow"))?,
        };
        run.review = Review::Applying(acceptance.clone());
        self.save(Some(before), record).await?;
        Ok(acceptance)
    }

    /// An uncertain target write stays uncertain until that owner's ledger is
    /// queried. A new operation ID or a regenerated note is never allocated.
    pub async fn uncertain(&self, id: u64) -> Result<(), Error> {
        let (before, mut record) = self.load().await?;
        let run = record
            .runs
            .iter_mut()
            .find(|r| r.id == id)
            .ok_or(Error::Missing)?;
        let acceptance = match &run.review {
            Review::Applying(a) | Review::Reconciling(a) => a.clone(),
            _ => return Err(Error::Review),
        };
        run.review = Review::Reconciling(acceptance);
        self.save(Some(before), record).await
    }

    /// Call only after the authenticated destination reports its durable
    /// ledger receipt. Acceptance never appends to the terminal run reducer.
    pub async fn acknowledge(&self, id: u64, receipt: Receipt) -> Result<(), Error> {
        let (before, mut record) = self.load().await?;
        let run = record
            .runs
            .iter_mut()
            .find(|r| r.id == id)
            .ok_or(Error::Missing)?;
        let acceptance = match &run.review {
            Review::Applying(a) | Review::Reconciling(a) | Review::Added { acceptance: a, .. } => {
                a.clone()
            },
            _ => return Err(Error::Review),
        };
        if receipt.operation != acceptance.operation
            || receipt.proposal != acceptance.proposal
            || receipt.destination != acceptance.destination
            || receipt.resource.is_empty()
            || receipt.storage_witness.is_empty()
            || encoded(&receipt)?.len() > 4096
        {
            return Err(Error::Invalid("destination receipt"));
        }
        if let Review::Added { receipt: old, .. } = &run.review {
            return if *old == receipt {
                Ok(())
            } else {
                Err(Error::Invalid("receipt changed"))
            };
        }
        run.review = Review::Added {
            acceptance,
            receipt,
        };
        self.save(Some(before), record).await
    }

    pub async fn view(&self, id: Option<u64>) -> Result<WorkshopView, Error> {
        let (_, record) = self.load().await?;
        let run = match id {
            Some(id) => Some(
                record
                    .runs
                    .iter()
                    .find(|r| r.id == id)
                    .ok_or(Error::Missing)?,
            ),
            None => record.runs.last(),
        };
        let mut controls = vec![];
        if !record.dissolved && record.binding.lifecycle == Lifecycle::Revoked {
            controls.push(Control::Dissolve);
        }
        if !record.dissolved && record.binding.lifecycle != Lifecycle::Revoked {
            controls.extend([Control::Revoke, Control::Dissolve]);
            controls.push(if record.binding.lifecycle == Lifecycle::Paused {
                Control::Resume
            } else {
                Control::Pause
            });
        }
        let mut view = WorkshopView {
            installation: self.id,
            run: run.map(|r| r.id),
            generation: record.binding.generation,
            lifecycle: if record.dissolved {
                InstallationStatus::Dissolved
            } else {
                match record.binding.lifecycle {
                    Lifecycle::Active => InstallationStatus::Active,
                    Lifecycle::Paused => InstallationStatus::Paused,
                    Lifecycle::Revoked => InstallationStatus::Revoked,
                }
            },
            phase: RunStatus::Installed,
            review: run.map(|r| r.review.clone()),
            proposal: run.and_then(|r| r.proposal.clone()),
            usage: Usage::default(),
            limits: run.map(|r| r.limits),
            controls,
        };
        let active = record.binding.lifecycle == Lifecycle::Active && !record.dissolved;
        if let Some(run) = run {
            let reducer = run.replay(&self.owner)?;
            view.phase = match reducer.phase() {
                RunPhase::Ready { correlation } => RunStatus::Ready(wire(correlation)),
                RunPhase::Awaiting { correlation, .. } => RunStatus::Awaiting(wire(correlation)),
                RunPhase::Paused { correlation } => RunStatus::Paused(wire(correlation)),
                RunPhase::Reconciling { correlation, .. } => {
                    RunStatus::Reconciling(wire(correlation))
                },
                RunPhase::Terminal(outcome) => match outcome {
                    servitor::TerminalOutcome::Completed => RunStatus::Completed,
                    servitor::TerminalOutcome::Refused => RunStatus::Refused,
                    servitor::TerminalOutcome::Cancelled => RunStatus::Cancelled,
                    servitor::TerminalOutcome::Failed => RunStatus::Failed,
                    servitor::TerminalOutcome::BudgetExhausted => RunStatus::BudgetExhausted,
                },
            };
            let usage = reducer.usage();
            view.usage = Usage {
                decisions: usage.decisions,
                tool_calls: usage.tool_calls,
                tokens: usage.tokens,
                elapsed_ms: reducer.last_at_ms() - run.started,
                consecutive_failures: usage.consecutive_failures,
            };
            if active && matches!(run.review, Review::AwaitingReview) {
                view.controls.extend([Control::Accept, Control::Reject]);
            }
            if matches!(run.review, Review::Applying(_) | Review::Reconciling(_)) {
                view.controls.push(Control::Reconcile);
            }
        }
        if active
            && !record.runs.iter().any(|r| {
                r.replay(&self.owner)
                    .is_ok_and(|s| !matches!(s.phase(), RunPhase::Terminal(_)))
            })
        {
            view.controls.push(Control::Run);
        }
        Ok(view)
    }
}

#[derive(Clone, Debug)]
pub struct Dispatch {
    pub correlation: Correlation,
    installation: [u8; 16],
    issued_at_ms: u64,
    deadline_at_ms: u64,
    inputs: Vec<Passage>,
    candidate: Option<Proposal>,
}
impl Dispatch {
    /// Fixed original run deadline; persistence waits do not extend it.
    pub fn deadline_at_ms(&self) -> u64 {
        self.deadline_at_ms
    }
    pub fn issued_at_ms(&self) -> u64 {
        self.issued_at_ms
    }
}

#[derive(Clone, Debug)]
pub struct Observation {
    pub correlation: Correlation,
    pub proposal: Option<Proposal>,
}

/// The same deterministic worker body for every host. It sees captured public
/// bytes and correlation, with no authority provider, store or destination.
pub fn evaluate(dispatch: Dispatch) -> Result<Observation, Error> {
    let c = dispatch.correlation;
    let owner = &dispatch
        .inputs
        .first()
        .ok_or(Error::Invalid("selection"))?
        .source
        .owner;
    validate(owner, &dispatch.inputs)?;
    let proposed = match c.step {
        0 => None,
        1 => Some(proposal(dispatch.installation, c.run, &dispatch.inputs)?),
        2 => {
            let wanted = proposal(dispatch.installation, c.run, &dispatch.inputs)?;
            if dispatch.candidate.as_ref() != Some(&wanted) {
                return Err(Error::Invalid("candidate"));
            }
            Some(wanted)
        },
        _ => return Err(Error::Invalid("workflow step")),
    };
    Ok(Observation {
        correlation: c,
        proposal: proposed,
    })
}

#[cfg(all(feature = "workshop-worker", not(target_arch = "wasm32")))]
pub mod worker {
    //! Execution transport only. The owner persists lifecycle changes, requests
    //! cancellation, and observes `join`; none of those proves an external effect.
    use super::*;
    use std::sync::{atomic::AtomicBool, mpsc::Receiver};

    pub struct Worker {
        actor: armillary::ActorHandle<Dispatch>,
        cancelled: Arc<AtomicBool>,
        pub observations: Receiver<Result<Observation, Error>>,
    }
    impl Worker {
        pub fn spawn(wake: armillary::Wake) -> Self {
            let cancelled = Arc::new(AtomicBool::new(false));
            let stop = cancelled.clone();
            let (actor, observations) =
                armillary::spawn_named("alembic-evidence-brief", wake, move |commands, updates| {
                    while let Ok(dispatch) = commands.recv() {
                        if stop.load(Ordering::Acquire) {
                            break;
                        }
                        let result = evaluate(dispatch);
                        if stop.load(Ordering::Acquire) {
                            break;
                        }
                        updates.emit(result);
                    }
                });
            Self {
                actor,
                cancelled,
                observations,
            }
        }
        pub fn dispatch(&self, dispatch: Dispatch) -> bool {
            !self.cancelled.load(Ordering::Acquire) && self.actor.command(dispatch)
        }
        pub fn cancel(&self) {
            self.cancelled.store(true, Ordering::Release);
        }
        pub fn join(self) -> Result<(), armillary::ActorError> {
            self.actor.join()
        }
    }
}

#[cfg(test)]
mod tests;
