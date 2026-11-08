// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The `authority-guest` component host (cycle breaks P3, D15 (h)).
//!
//! A domain's authority ships as a pure component: its world imports only `log` and exports
//! `domain-authority` (`describe`, `validate`, `admit`). The candidate crosses the boundary
//! as copied bytes in the format the guest declares. This crate loads the component,
//! enforces the budget, and returns a verdict. It knows no domain and no Mere type.
//!
//! Containment, and why each choice:
//!
//! * **Fuel, not time.** Every call gets [`Budget::fuel`] and no more. Fuel is deterministic,
//!   so two devices running the same component on the same candidate are cut off at the same
//!   place; a wall-clock watchdog is not.
//! * **A fresh instance per call.** A trap (fuel cut-off, panic, memory cap) leaves a
//!   component instance unable to be entered again, and a call that leaves state behind
//!   would let one candidate influence the next verdict. Instantiating from the
//!   pre-linked, once-compiled component costs 0.4 ms (spike), so each call runs in a new
//!   store and a trap needs no recovery: the next call is simply a new instance.
//! * **A closed WASI.** The Rust standard library on `wasm32-wasip2` imports WASI. The host
//!   links it with no preopens, no environment, no arguments, no network and no stdio, a
//!   wall and monotonic clock fixed at zero and a fixed-seed random generator, so the guest
//!   can read neither the time nor entropy, and an iteration order it takes from a hashed
//!   collection is the same on every device.
//! * **A trap is a refusal.** A cut-off, panic or memory cap is reported as
//!   [`Verdict::Trapped`]; the caller refuses the candidate. An authority never fails open.
//!
//! Compilation (Cranelift, about a second for a 7.5 MiB component) happens once per
//! [`AuthorityComponent::load`]. Nothing is deserialised from a stored artifact:
//! `Component::deserialize` is `unsafe` and trusts the bytes, so a precompiled file on disk
//! would let anyone who can write the data directory bypass the pinned component hash.

use std::convert::Infallible;
use std::time::{Duration, Instant};

pub use wasmtime;
use wasmtime::component::{Component, HasSelf, Linker, ResourceTable};
use wasmtime::{Config, Engine, ResourceLimiter, Store};
use wasmtime_wasi::clocks::{HostMonotonicClock, HostWallClock};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

wasmtime::component::bindgen!({
    // The shared `mere:script` contract (one WIT, many worlds).
    path: "../wit",
    world: "authority-guest",
});

use exports::mere::script::domain_authority::Claim as WitClaim;

/// Most bytes a guest component may be (the capsule cap, D12).
pub const MAX_COMPONENT_BYTES: usize = 16 * 1024 * 1024;
/// The candidate format the resident encodes today.
pub const GRAPH_SNAPSHOT_AND_FACETS: &str = "mere.graph-snapshot-and-facets/json";

/// What one call may spend. The defaults come from the P3 spike over the corpus of the seven
/// moving tests plus 571 tampers (Findings, "P3 spike"), with headroom for graphs larger than
/// the corpus: the corpus peaked at 1.74e9 fuel (about 101 ms), 6.6 MiB of memory and a
/// 296 KB candidate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Budget {
    /// Fuel per call: 2e10, 11 times the corpus maximum, about 1 s of guest time at the
    /// measured rate (3.9e8 fuel in 20 ms).
    pub fuel: u64,
    /// Memory a call's instance may hold: 128 MiB, 19 times the measured peak.
    pub memory_bytes: usize,
    /// The largest candidate (or before and after together) a call accepts: 16 MiB, 54
    /// times the corpus maximum. The guest decodes it into a graph several times that size.
    pub candidate_bytes: usize,
}

impl Default for Budget {
    fn default() -> Self {
        Self {
            fuel: 20_000_000_000,
            memory_bytes: 128 * 1024 * 1024,
            candidate_bytes: 16 * 1024 * 1024,
        }
    }
}

/// What a guest claims (the WIT's `claim`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Claim {
    pub domain: String,
    pub version: String,
    pub graph_format: String,
    /// Whether `admit` is a real check; when false it is never called.
    pub admission: bool,
}

impl From<WitClaim> for Claim {
    fn from(claim: WitClaim) -> Self {
        Self {
            domain: claim.domain,
            version: claim.version,
            graph_format: claim.graph_format,
            admission: claim.admission,
        }
    }
}

/// A guest could not be loaded.
#[derive(Debug)]
pub enum LoadError {
    /// Over [`MAX_COMPONENT_BYTES`].
    TooLarge(usize),
    /// Not a component, or it does not fit the `authority-guest` world (an import beyond
    /// `log` and WASI, a missing export).
    Invalid(String),
    /// `describe` trapped or did not answer within the budget.
    Describe(String),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooLarge(bytes) => write!(f, "component is {bytes} bytes, over {MAX_COMPONENT_BYTES}"),
            Self::Invalid(message) => write!(f, "not an authority-guest component: {message}"),
            Self::Describe(message) => write!(f, "describe failed: {message}"),
        }
    }
}

impl std::error::Error for LoadError {}

/// The guest's answer to one call.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// The candidate is admissible.
    Admitted,
    /// The guest refused it, with its text.
    Refused(String),
    /// The guest trapped (fuel, panic, memory) or the candidate was over budget before it
    /// ran. The caller refuses the candidate.
    Trapped(String),
}

impl Verdict {
    /// `Ok` when admitted; otherwise the refusal text, a trap prefixed so a client can tell it.
    pub fn into_result(self) -> Result<(), String> {
        match self {
            Self::Admitted => Ok(()),
            Self::Refused(message) => Err(message),
            Self::Trapped(message) => Err(format!("authority guest trapped: {message}")),
        }
    }
}

/// What one call cost.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Metrics {
    pub fuel: u64,
    /// The most memory the instance asked to hold, in bytes.
    pub peak_memory: usize,
    pub elapsed: Duration,
}

struct State {
    wasi: WasiCtx,
    table: ResourceTable,
    memory_cap: usize,
    peak_memory: usize,
    logs: Vec<String>,
}

impl WasiView for State {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

impl ResourceLimiter for State {
    fn memory_growing(
        &mut self,
        _current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> wasmtime::Result<bool> {
        self.peak_memory = self.peak_memory.max(desired);
        Ok(desired <= self.memory_cap)
    }

    fn table_growing(
        &mut self,
        _current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> wasmtime::Result<bool> {
        Ok(desired <= 1 << 20)
    }
}

impl mere::script::log::Host for State {
    fn log(&mut self, message: String) {
        // Bounded: a guest cannot grow the host by logging.
        if self.logs.len() < 64 {
            self.logs.push(message.chars().take(1024).collect());
        }
    }
}

struct ZeroWallClock;
impl HostWallClock for ZeroWallClock {
    fn resolution(&self) -> Duration {
        Duration::from_secs(1)
    }
    fn now(&self) -> Duration {
        Duration::ZERO
    }
}

struct ZeroMonotonicClock;
impl HostMonotonicClock for ZeroMonotonicClock {
    fn resolution(&self) -> u64 {
        1_000_000_000
    }
    fn now(&self) -> u64 {
        0
    }
}

/// A fixed-seed generator (splitmix64): the same bytes on every device and every call.
struct FixedRandom(u64);
impl rand::TryRng for FixedRandom {
    type Error = Infallible;
    fn try_next_u32(&mut self) -> Result<u32, Infallible> {
        Ok(self.try_next_u64()? as u32)
    }
    fn try_next_u64(&mut self) -> Result<u64, Infallible> {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        Ok(z ^ (z >> 31))
    }
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Infallible> {
        for chunk in dest.chunks_mut(8) {
            let word = self.try_next_u64()?.to_le_bytes();
            chunk.copy_from_slice(&word[..chunk.len()]);
        }
        Ok(())
    }
}

fn closed_wasi() -> WasiCtx {
    let mut builder = WasiCtxBuilder::new();
    builder
        .wall_clock(ZeroWallClock)
        .monotonic_clock(ZeroMonotonicClock)
        .secure_random(FixedRandom(0x6d65_7265_0001))
        .insecure_random(FixedRandom(0x6d65_7265_0002))
        .insecure_random_seed(0x6d65_7265_0003);
    builder.build()
}

/// One compiled, linked authority component: cheap to call, safe to share between threads.
pub struct AuthorityComponent {
    engine: Engine,
    pre: AuthorityGuestPre<State>,
    claim: Claim,
    budget: Budget,
    component_bytes: usize,
}

impl AuthorityComponent {
    /// An engine with fuel metering on, for [`Self::load`]. Share one between components.
    pub fn engine() -> wasmtime::Result<Engine> {
        let mut config = Config::new();
        config.consume_fuel(true);
        Engine::new(&config)
    }

    /// Compile `bytes` (the exact bytes a host verified), link the world, and call
    /// `describe` once under the budget. Compiles on the calling thread (about a second
    /// for a component of the divination validator's size).
    pub fn load(engine: &Engine, bytes: &[u8], budget: Budget) -> Result<Self, LoadError> {
        if bytes.len() > MAX_COMPONENT_BYTES {
            return Err(LoadError::TooLarge(bytes.len()));
        }
        let invalid = |error: wasmtime::Error| LoadError::Invalid(format!("{error:#}"));
        let component = Component::from_binary(engine, bytes).map_err(invalid)?;
        let mut linker: Linker<State> = Linker::new(engine);
        wasmtime_wasi::p2::add_to_linker_sync(&mut linker).map_err(invalid)?;
        mere::script::log::add_to_linker::<_, HasSelf<State>>(&mut linker, |state| state).map_err(invalid)?;
        let pre = AuthorityGuestPre::new(linker.instantiate_pre(&component).map_err(invalid)?).map_err(invalid)?;
        let mut loaded = Self {
            engine: engine.clone(),
            pre,
            claim: Claim {
                domain: String::new(),
                version: String::new(),
                graph_format: String::new(),
                admission: false,
            },
            budget,
            component_bytes: bytes.len(),
        };
        let (claim, _) = loaded
            .call(|guest, store| guest.mere_script_domain_authority().call_describe(store))
            .map_err(|error| LoadError::Describe(error.to_string()))?;
        loaded.claim = claim.into();
        Ok(loaded)
    }

    /// What the guest claimed at load.
    pub fn claim(&self) -> &Claim {
        &self.claim
    }

    pub fn budget(&self) -> Budget {
        self.budget
    }

    /// The size of the component this was loaded from.
    pub fn component_bytes(&self) -> usize {
        self.component_bytes
    }

    /// Run one call in a fresh, fuel-limited, memory-capped instance.
    fn call<T>(
        &self,
        run: impl FnOnce(&AuthorityGuest, &mut Store<State>) -> wasmtime::Result<T>,
    ) -> Result<(T, Metrics), wasmtime::Error> {
        let mut store = Store::new(
            &self.engine,
            State {
                wasi: closed_wasi(),
                table: ResourceTable::new(),
                memory_cap: self.budget.memory_bytes,
                peak_memory: 0,
                logs: Vec::new(),
            },
        );
        store.limiter(|state| state);
        store.set_fuel(self.budget.fuel)?;
        let started = Instant::now();
        let guest = self.pre.instantiate(&mut store)?;
        let outcome = run(&guest, &mut store);
        let metrics = Metrics {
            fuel: self.budget.fuel - store.get_fuel().unwrap_or(0),
            peak_memory: store.data().peak_memory,
            elapsed: started.elapsed(),
        };
        outcome.map(|value| (value, metrics))
    }

    fn verdict(&self, result: Result<(Result<(), String>, Metrics), wasmtime::Error>) -> (Verdict, Metrics) {
        match result {
            Ok((Ok(()), metrics)) => (Verdict::Admitted, metrics),
            Ok((Err(message), metrics)) => (Verdict::Refused(message), metrics),
            Err(trap) => (Verdict::Trapped(format!("{trap:#}")), Metrics::default()),
        }
    }

    /// Whether the complete `candidate` is admissible, and what the call cost.
    pub fn validate_measured(&self, candidate: &[u8]) -> (Verdict, Metrics) {
        if candidate.len() > self.budget.candidate_bytes {
            return (
                Verdict::Trapped(format!(
                    "candidate is {} bytes, over the {} byte budget",
                    candidate.len(),
                    self.budget.candidate_bytes
                )),
                Metrics::default(),
            );
        }
        let result = self.call(|guest, store| guest.mere_script_domain_authority().call_validate(store, candidate));
        self.verdict(result)
    }

    pub fn validate(&self, candidate: &[u8]) -> Verdict {
        self.validate_measured(candidate).0
    }

    /// The before/after check, when the claim says there is one. A guest without one admits.
    pub fn admit_measured(&self, before: &[u8], after: &[u8]) -> (Verdict, Metrics) {
        if !self.claim.admission {
            return (Verdict::Admitted, Metrics::default());
        }
        if before.len().saturating_add(after.len()) > self.budget.candidate_bytes {
            return (
                Verdict::Trapped("transition is over the candidate budget".into()),
                Metrics::default(),
            );
        }
        let result = self.call(|guest, store| guest.mere_script_domain_authority().call_admit(store, before, after));
        self.verdict(result)
    }

    pub fn admit(&self, before: &[u8], after: &[u8]) -> Verdict {
        self.admit_measured(before, after).0
    }

    /// The guest's `log` output for one validation, for tests and diagnostics.
    pub fn validate_logged(&self, candidate: &[u8]) -> (Verdict, Vec<String>) {
        let mut logs = Vec::new();
        let result = self.call(|guest, store| {
            let verdict = guest.mere_script_domain_authority().call_validate(&mut *store, candidate);
            logs = std::mem::take(&mut store.data_mut().logs);
            verdict
        });
        (self.verdict(result).0, logs)
    }
}
