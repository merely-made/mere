#![allow(dead_code,unused_mut,unused_variables,private_interfaces)]
extern crate alloc;
extern crate self as cubecl_environment;
extern crate self as cubecl_common;
extern crate self as futures_util;
extern crate self as log;
#[macro_export]macro_rules! warn {($($arg:tt)*)=>{{let _=format!($($arg)*);}}}
pub mod collections {pub use std::collections::HashMap;}
pub mod future {
 pub fn block_on<F:std::future::Future>(f:F)->F::Output {
  let w=std::task::Waker::noop();let mut cx=std::task::Context::from_waker(w);let mut f=std::pin::pin!(f);
  loop {if let std::task::Poll::Ready(v)=f.as_mut().poll(&mut cx){return v;}}
 }
 pub async fn join_all<I:IntoIterator>(i:I)->Vec<<I::Item as std::future::Future>::Output> where I::Item:std::future::Future {
  let mut out=Vec::new();for f in i {out.push(f.await);}out
 }
}
pub mod profile {pub use std::time::Instant;}
pub mod dry_run {pub struct RealRun;impl RealRun{pub fn new()->Self{Self}}}
use std::{time::Duration,marker::PhantomData,sync::{Arc,atomic::{AtomicUsize,Ordering}}};
struct Mutex<T>(std::sync::Mutex<T>);impl<T> Mutex<T>{fn new(v:T)->Self{Self(std::sync::Mutex::new(v))}fn lock(&self)->std::sync::MutexGuard<'_,T>{self.0.lock().unwrap()}}
trait AutotuneKey:Clone+std::fmt::Debug+Eq+std::hash::Hash{}impl<T:Clone+std::fmt::Debug+Eq+std::hash::Hash> AutotuneKey for T{}
#[derive(Debug,Clone)]enum AutotuneError {Unknown{name:String,err:String},InvalidSamples{name:String},NotMeasured{name:String},Skip{name:String}}
#[derive(Debug,Clone,PartialEq)]struct Score(u64);impl Score{fn score(&self)->u64{self.0}}
#[derive(Debug,Clone,PartialEq)]struct AutotuneOutcome{index:usize,computation:Score}
mod cache {
#[cfg(persistence)]
use alloc::vec::Vec;

#[cfg(persistence)]
use cubecl_environment::persistence::StoreError;
#[cfg(persistence)]
use cubecl_environment::persistence::{CacheOption, Namespace, Store, StoreOptions};
#[cfg(serializable)]
use serde::{Deserialize, Serialize};

use super::{AutotuneError, AutotuneKey, AutotuneOutcome};
use alloc::string::String;
use cubecl_environment::collections::HashMap;

#[derive(Debug)]
pub(crate) enum CacheEntry {
    Done {
        checksum: ChecksumState,
        fastest_index: usize,
    },
    Pending,
}

#[derive(Debug)]
#[allow(dead_code)] // Some variants are not created when the cache isn't saved.
pub(crate) enum ChecksumState {
    Match,
    NoMatch,
    ToBeVerified(String),
}

/// Persistent cache key
#[cfg(persistence)]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Hash)]
pub struct PersistentCacheKey<K> {
    /// The autotune key identifying the operation.
    pub key: K,
    /// The checksum of the candidate list the key was tuned under: an answer
    /// to another list is an answer to another question.
    pub checksum: String,
}

/// Persistent cache entry
///
/// Only [`fastest_index`](Self::fastest_index) is read back: hydration seeds the in-memory cache
/// from it and nothing else. Everything below it is stored so a cache entry can be inspected after
/// the fact — why a kernel won, against which measurements, and under which bounds — which is the
/// question that cannot be answered from a live process once tuning is over. That is also why the
/// type is `pub`: reading an entry back is the point.
#[cfg(persistence)]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct PersistentCacheValue {
    /// Index of the fastest candidate operation.
    pub fastest_index: usize,
    /// Benchmarking results for all autotune candidates.
    pub results: Vec<AutotuneResult>,
    /// Optional input size bounds for which the autotune result applies.
    ///
    /// Defaulted, so entries written before this field existed still decode. Without it every
    /// cached key on every existing installation would fail to read and re-tune from scratch.
    #[serde(default)]
    pub bounds: Option<crate::tune::Bounds>,
    /// Optional execution time limit for the autotune process.
    ///
    /// Defaulted for the same reason as [`bounds`](Self::bounds).
    #[serde(default)]
    pub limit: Option<core::time::Duration>,
}

#[cfg_attr(serializable, derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
/// The result of an autotune job.
pub struct AutotuneResult {
    /// The outcome of the benchmark.
    pub outcome: Result<AutotuneOutcome, AutotuneError>,
}

impl AutotuneResult {
    pub(crate) fn error(error: AutotuneError) -> Self {
        Self {
            outcome: Err(error),
        }
    }
    pub(crate) fn success(outcome: AutotuneOutcome) -> Self {
        Self {
            outcome: Ok(outcome),
        }
    }
}

impl Eq for AutotuneResult {}
impl PartialEq for AutotuneResult {
    fn eq(&self, other: &Self) -> bool {
        match (&self.outcome, &other.outcome) {
            (Ok(lhs), Ok(rhs)) => lhs == rhs,
            (Ok(_), Err(_)) => false,
            (Err(_), Ok(_)) => false,
            // We don't have to check the error
            (Err(_), Err(_)) => true,
        }
    }
}

/// Use to find and reuse the best kernel for some input
#[derive(Debug)]
pub(crate) struct TuneCache<K> {
    /// The single in-memory home of tuning state, keyed for the per-launch
    /// lookup: tuned picks, in-flight tunes and checksum verdicts. Hydrated
    /// from the store, which retains nothing itself, and rebuilt when the
    /// environment switches.
    in_memory_cache: HashMap<K, CacheEntry>,
    /// Write-through persistence, or `None` when the persistent cache is
    /// disabled, so no cache file is ever touched. Lazy: entries live in
    /// [`Self::in_memory_cache`] once hydrated, not here.
    #[cfg(persistence)]
    persistent_cache: Option<Store<PersistentCacheKey<K>, PersistentCacheValue>>,
    /// The namespace the table is stored under, whether or not it is
    /// persisted: what a [record](super::TuneRecord) names it by.
    #[cfg(persistence)]
    table: String,
    /// Whether everything the store holds has been ingested into
    /// [`Self::in_memory_cache`]. What makes an ordinary miss cost a bool
    /// check rather than a walk; `false` until the first sync, and again
    /// after an environment switch.
    #[cfg(persistence)]
    hydrated: bool,
    /// The environment generation [`Self::in_memory_cache`] was built under;
    /// see [`cubecl_environment::environment::generation`].
    #[cfg(persistence)]
    generation: u32,
}

/// Result of the cache try
#[derive(Debug)]
pub enum TuneCacheResult {
    /// An operation is found.
    Hit {
        /// The index of the fastest operation to execute.
        fastest_index: usize,
    },
    /// The operation might be cached, but we don't know yet whether the checksum is valid.
    Unchecked,
    /// A tuning job is in flight for this key — the worker hasn't published a result yet.
    /// Callers that see this fall through to running the operation rather than blocking on
    /// the in-flight job.
    Pending,
    /// No operation is found yet.
    Miss,
}

impl<K: AutotuneKey> TuneCache<K> {
    pub(crate) fn new(
        #[cfg_attr(not(persistence), allow(unused_variables))] name: &str,
        #[cfg_attr(not(persistence), allow(unused_variables))] device_id: &str,
    ) -> Self {
        #[cfg(persistence)]
        {
            use crate::config::RuntimeConfig;
            use alloc::format;

            let config = crate::config::CubeClRuntimeConfig::get();
            let namespace = Namespace::scoped("autotune", format!("{device_id}/{name}"));
            let table = namespace.as_str().into();

            if config.autotune.disable_cache {
                return TuneCache {
                    in_memory_cache: HashMap::new(),
                    persistent_cache: None,
                    table,
                    hydrated: true,
                    generation: cubecl_environment::environment::generation(),
                };
            }

            // Sampled before the store opens, so a switch landing in between
            // reads as "rebuild", never as "this state belongs to the new
            // environment".
            let generation = cubecl_environment::environment::generation();
            let mut cache = TuneCache {
                in_memory_cache: HashMap::new(),
                persistent_cache: Some(Store::new(
                    StoreOptions::new()
                        .storage(namespace)
                        .cache(CacheOption::Lazy),
                )),
                table,
                hydrated: false,
                generation,
            };
            log::info!("Load autotune cache ...");
            let loaded = cache.sync_persistent();
            log::info!("Loaded {loaded} autotune cached entries");

            cache
        }

        #[cfg(not(persistence))]
        {
            TuneCache {
                in_memory_cache: HashMap::new(),
            }
        }
    }

    pub fn fastest(&self, key: &K) -> TuneCacheResult {
        let Some(val) = self.in_memory_cache.get(key) else {
            return TuneCacheResult::Miss;
        };

        let CacheEntry::Done {
            checksum,
            fastest_index,
        } = val
        else {
            // Pending: clone the receiver so the caller can subscribe to the in-flight tune.
            let CacheEntry::Pending = val else {
                unreachable!()
            };
            return TuneCacheResult::Pending;
        };

        if cfg!(persistence) {
            match checksum {
                ChecksumState::ToBeVerified(..) => TuneCacheResult::Unchecked, // Don't know yet.
                ChecksumState::NoMatch => TuneCacheResult::Miss,               // Can't use this.
                ChecksumState::Match => TuneCacheResult::Hit {
                    fastest_index: *fastest_index,
                },
            }
        } else {
            // Clippy;
            let _ = checksum;
            TuneCacheResult::Hit {
                fastest_index: *fastest_index,
            }
        }
    }

    #[cfg(persistence)]
    pub fn validate_checksum(&mut self, key: &K, checksum: &str) -> TuneCacheResult {
        let Some(val) = self.in_memory_cache.get_mut(key) else {
            return TuneCacheResult::Miss;
        };

        if let CacheEntry::Done {
            checksum: checksum_state,
            ..
        } = val
            && let ChecksumState::ToBeVerified(checksum_expected) = checksum_state
        {
            if checksum_expected == checksum {
                *checksum_state = ChecksumState::Match;
            } else {
                *checksum_state = ChecksumState::NoMatch;
            }
        }

        self.fastest(key)
    }

    /// The index settled for `key`, reading only: `None` for anything but a verified result, and
    /// for every key once the environment has switched away from the one this cache was built
    /// under — the reset that switch owes is left to the next lookup that tunes.
    pub(crate) fn settled(&self, key: &K) -> Option<usize> {
        #[cfg(persistence)]
        if self.environment_switched() {
            return None;
        }

        match self.fastest(key) {
            TuneCacheResult::Hit { fastest_index } => Some(fastest_index),
            TuneCacheResult::Unchecked | TuneCacheResult::Pending | TuneCacheResult::Miss => None,
        }
    }

    /// Mark a key as being tuned. Used by [`Tuner::check_tune`] under the cache mutex so that
    /// concurrent callers see [`TuneCacheResult::Pending`] instead of starting a second job
    /// for the same key.
    pub(crate) fn mark_pending(&mut self, key: K) {
        self.in_memory_cache.insert(key, CacheEntry::Pending);
    }

    pub(crate) fn cache_insert(&mut self, key: K, fastest_index: usize) {
        self.in_memory_cache.insert(
            key,
            CacheEntry::Done {
                checksum: ChecksumState::Match,
                fastest_index,
            },
        );
    }
}

#[cfg(persistence)]
impl<K: AutotuneKey> TuneCache<K> {
    /// Drops tuning state belonging to a previous environment, so a switch
    /// re-hydrates and re-tunes rather than serving the old environment's
    /// picks. One relaxed atomic load when nothing switched.
    ///
    /// In-flight tunes are dropped with everything else: their completion
    /// still records a hardware-valid result, so the whole cost of the race
    /// is one duplicate tune per switch.
    pub(crate) fn reset_if_environment_switched(&mut self) {
        if !self.environment_switched() {
            return;
        }

        log::debug!("Environment switched, resetting the autotune cache");
        self.generation = cubecl_environment::environment::generation();
        self.in_memory_cache.clear();
        self.hydrated = false;
    }

    /// Whether the environment switched since this cache was built. Never with persistence
    /// disabled: the tuning state is then process-local and unbound, like a store without a
    /// storage, and survives switches.
    fn environment_switched(&self) -> bool {
        self.persistent_cache.is_some()
            && cubecl_environment::environment::generation() != self.generation
    }

    /// Ingest everything the persistent store holds into the in-memory cache,
    /// as unverified entries.
    ///
    /// Runs at construction, and again whenever `hydrated` fell back to
    /// `false` after an environment switch. Once hydrated, a miss costs one
    /// bool check here — never a walk, and never a rescan of the database
    /// under the tuner mutex.
    ///
    /// Returns how many entries the store delivered.
    pub(crate) fn sync_persistent(&mut self) -> usize {
        if self.hydrated {
            return 0;
        }

        let Some(persistent_cache) = self.persistent_cache.as_mut() else {
            return 0;
        };

        let mut delivered = 0;
        persistent_cache.scan(|key, value| {
            delivered += 1;
            self.in_memory_cache
                .entry(key.key)
                .or_insert(CacheEntry::Done {
                    checksum: ChecksumState::ToBeVerified(key.checksum),
                    fastest_index: value.fastest_index,
                });
        });
        self.hydrated = true;

        delivered
    }

    /// The namespace the table is stored under.
    pub(crate) fn table(&self) -> &str {
        &self.table
    }

    /// Store an answer in the table, and say whether it took it: never when
    /// the cache is disabled.
    pub(crate) fn persistent_cache_insert(
        &mut self,
        key: K,
        checksum: String,
        value: PersistentCacheValue,
    ) -> bool {
        let Some(persistent_cache) = self.persistent_cache.as_mut() else {
            return false;
        };

        let Err(err) = persistent_cache.insert(PersistentCacheKey { key, checksum }, value) else {
            return true;
        };
        match err {
            StoreError::DuplicatedKey {
                key,
                value_previous,
                value_updated,
            } => log::warn!(
                "Autotune the same function multiple times for key {key:?} => old {value_previous:?}, new {value_updated:?}"
            ),
            // Another process sharing the cache root tuned this key first.
            // Routine with N training processes on a cold cache, and both
            // results are valid, so it stays quiet: warning here would
            // print a full result payload per key on every cold start.
            StoreError::KeyOutOfSync { key, .. } => {
                log::debug!("Autotune result for key {key:?} was already stored concurrently")
            },
            StoreError::Backend { key, error } => log::warn!(
                "Autotune result for key {key:?} could not be stored, it will be retuned: {error}"
            ),
        }
        false
    }
}

}
use cache::{AutotuneResult,TuneCache,TuneCacheResult};
trait TuneInputs {type At<'a>:Clone+Send;}
impl TuneInputs for (){type At<'a>=();}
trait AutotuneOutput{}impl AutotuneOutput for (){}
struct TuneFn<F: TuneInputs,O>{name:String,marker:PhantomData<(F,O)>}
#[derive(Clone)]struct Client{entered:Arc<AtomicUsize>}
impl Client {fn exclusive<R,F:FnOnce()->R>(&self,_f:F)->Result<R,&'static str>{Err("exclusive acquisition refused before task execution")}}
#[derive(Debug)]struct Batch{entries:Vec<usize>}
impl Batch{fn indices(&self)->Vec<usize>{self.entries.clone()}fn is_empty(&self)->bool{self.entries.is_empty()}fn unconditional(entries:Vec<usize>)->Self{Self{entries}}}
#[derive(Debug)]struct TunePlan{batches:std::collections::VecDeque<Batch>}
impl TunePlan{fn next(&mut self)->Batch{self.batches.pop_front().unwrap_or(Batch{entries:vec![]})}}
#[derive(Debug)]struct BatchOutcome{results:Vec<(usize,AutotuneResult)>,steps:Vec<(String,Duration)>,short_circuit:Option<String>,any_success:bool,decided:Option<usize>,unmeasured:Vec<usize>}
#[derive(Debug)]struct PlanOutcome{steps:Vec<(String,Duration)>,short_circuit:Option<String>,decided:Option<usize>}
struct Schedule;
impl Schedule{
 async fn drive<'a,F:TuneInputs,O:AutotuneOutput>(&mut self,_b:Batch,_a:&[&TuneFn<F,O>],_i:F::At<'a>,c:&Client)->BatchOutcome{c.entered.fetch_add(1,Ordering::SeqCst);panic!("refusing client must not enter drive")}
pub(crate) fn run_batch<'a, F: TuneInputs, Out: AutotuneOutput>(
        &mut self,
        batch: Batch,
        autotunables: &[&TuneFn<F, Out>],
        inputs: <F as TuneInputs>::At<'a>,
        client: &Client,
    ) -> BatchOutcome
    where
        <F as TuneInputs>::At<'a>: Clone + Send,
    {
        let fallback = batch.indices();
        let run = || {
            let _real_run = crate::dry_run::RealRun::new();

            cubecl_environment::future::block_on(self.drive(batch, autotunables, inputs, client))
        };

        match client.clone().exclusive(run) {
            Ok(outcome) => outcome,
            Err(err) => BatchOutcome {
                results: fallback
                    .into_iter()
                    .map(|index| {
                        let error = AutotuneError::Unknown {
                            name: autotunables[index].name.to_string(),
                            err: err.to_string(),
                        };
                        (index, AutotuneResult::error(error))
                    })
                    .collect(),
                steps: Vec::new(),
                short_circuit: None,
                any_success: false,
                decided: None,
                unmeasured: Vec::new(),
            },
        }
    }
pub(crate) fn run_plan<'a, K, F, Out>(
        &mut self,
        key: &K,
        plan: &mut TunePlan,
        autotunables: &[&TuneFn<F, Out>],
        inputs: &<F as TuneInputs>::At<'a>,
        client: &Client,
        results: &mut [AutotuneResult],
    ) -> PlanOutcome
    where
        K: core::fmt::Debug,
        F: TuneInputs,
        Out: AutotuneOutput,
        <F as TuneInputs>::At<'a>: Clone + Send,
    {
        let mut steps = Vec::new();
        let mut retry = None;

        loop {
            let batch = retry.take().unwrap_or_else(|| plan.next());

            if batch.is_empty() {
                // Every candidate failed. A candidate that *executed* but
                // could not be *measured* — `Unknown` wraps benchmark-harness
                // failures like a profiling hiccup (timestamp query sets on a
                // busy stream), `InvalidSamples` collected timings it cannot
                // trust — is still a usable kernel: decide the first such,
                // unmeasured, rather than panicking the device thread. A
                // dead tune poisons every stream sharing it, and the decided
                // kernel executes for real right after — if it truly cannot
                // run, that failure surfaces there, exactly as a measured
                // winner's would. Only launch failures and manual skips say
                // the kernel itself is unusable; when nothing else remains,
                // there is genuinely no kernel to run.
                let executed_unmeasured = results.iter().position(|result| {
                    matches!(
                        &result.outcome,
                        Err(AutotuneError::Unknown { .. })
                            | Err(AutotuneError::InvalidSamples { .. })
                    )
                });
                if let Some(index) = executed_unmeasured {
                    log::warn!(
                        "Autotune measured no candidate for key {key:?}; \
                         deciding candidate {index} unmeasured.\n - results: {results:?}"
                    );
                    return PlanOutcome {
                        steps,
                        short_circuit: None,
                        decided: Some(index),
                    };
                }

                panic!(
                    "Can't execute the autotune plan for key: {key:?}\n - plan: {plan:?}\n - results: {results:?}"
                );
            }

            let outcome = self.run_batch(batch, autotunables, inputs.clone(), client);

            for (index, result) in outcome.results {
                results[index] = result;
            }
            steps.extend(outcome.steps);

            if outcome.any_success {
                return PlanOutcome {
                    steps,
                    short_circuit: outcome.short_circuit,
                    decided: outcome.decided,
                };
            }

            // Every candidate measured failed, so the ones their groups' patience skipped never
            // had their chance: measure them before the plan moves on.
            if !outcome.unmeasured.is_empty() {
                retry = Some(Batch::unconditional(outcome.unmeasured));
            }
        }
    }
}
struct Logger;
struct LogContext;
trait LogExt{fn push_tuning_step(&mut self,n:String,d:Duration);fn log_result<K:AutotuneKey>(&self,l:&mut Logger,k:&K,r:&[AutotuneResult]);}
impl LogExt for Option<LogContext>{fn push_tuning_step(&mut self,_n:String,_d:Duration){}fn log_result<K:AutotuneKey>(&self,_l:&mut Logger,_k:&K,_r:&[AutotuneResult]){}}
struct PendingBench{index:usize,name:String,launch:Option<Duration>}
async fn resolve_bench(_:PendingBench)->AutotuneResult{panic!("no pending samples in probe")}
struct TuneRequest<K:AutotuneKey>{key:K,results:Vec<AutotuneResult>,log_context:Option<LogContext>,pending:Vec<PendingBench>,decided:Option<usize>}
async fn process_request<K: AutotuneKey>(
    request: TuneRequest<K>,
    cache: &Mutex<TuneCache<K>>,
    logger: &Mutex<Logger>,
) -> TuneCacheResult {
    let TuneRequest {
        key,
        mut results,
        #[cfg(persistence)]
        checksum,
        mut log_context,
        pending,
        decided,
        #[cfg(persistence)]
        limit,
        #[cfg(persistence)]
        bounds,
        #[cfg(persistence)]
        recording,
    } = request;

    // Resolved concurrently, and each benchmark timed individually rather than timing the loop:
    // the profiles were all queued before any of them was polled, so awaiting them in turn would
    // charge the first benchmark for draining the whole device queue and report the rest as
    // free.
    let resolved = futures_util::future::join_all(pending.into_iter().map(|bench| {
        let index = bench.index;
        let name = bench.name.clone();
        let launch = bench.launch;

        async move {
            let started = cubecl_common::profile::Instant::now();
            let result = resolve_bench(bench).await;
            let step = launch.map(|launch| (name, launch + started.elapsed()));

            (index, step, result)
        }
    }))
    .await;

    for (index, step, result) in resolved {
        if let Some((name, duration)) = step {
            log_context.push_tuning_step(name, duration);
        }

        results[index] = result;
    }

    // Read before the sort, which reorders `results` out of tunable order. A
    // decided candidate whose own outcome is an error is one `Schedule::run_plan`
    // picked with nothing measured — the tune executed but could not be timed.
    #[cfg(persistence)]
    let unmeasured = decided.is_some_and(|index| results[index].outcome.is_err());

    results.sort_by(|a, b| {
        let a = a
            .outcome
            .as_ref()
            .map(|r| r.computation.score())
            .unwrap_or(u64::MAX);
        let b = b
            .outcome
            .as_ref()
            .map(|r| r.computation.score())
            .unwrap_or(u64::MAX);
        a.cmp(&b)
    });

    // The sort above orders what gets logged and persisted. It does not pick the winner when the
    // strategy already did: a scheduler that eliminates candidates leaves results built from
    // different sample counts behind, and `score` reads a short sample set as a stable one.
    let fastest_index = match decided {
        Some(index) => index,
        None => {
            results
                .first()
                .expect("At least one kernel needed.")
                .outcome
                .as_ref()
                .expect("At least one kernel has to succeed.")
                .index
        },
    };

    {
        log_context.log_result(&mut logger.lock(), &key, &results);
        // In-memory regardless: without it this key re-tunes on every call, and
        // a tune that measured nothing would keep failing the same way.
        cache.lock().cache_insert(key.clone(), fastest_index);

        // Not on disk, though. An unmeasured decision is a guess made to keep
        // the device thread alive, and the failures that produce one — a
        // profiling hiccup, timestamp query sets on a busy stream — are
        // transient. Persisting it would freeze the guess into every later
        // process and never measure the key again; letting it expire with this
        // one costs a re-tune and buys a real measurement.
        #[cfg(persistence)]
        let stored = !unmeasured
            && cache.lock().persistent_cache_insert(
                key,
                checksum,
                crate::tune::PersistentCacheValue {
                    fastest_index,
                    results,
                    bounds,
                    limit,
                },
            );

        #[cfg(persistence)]
        recording.finish(
            crate::tune::record::Answer {
                winner: fastest_index,
                stored,
            },
            log_context.as_ref(),
        );
    }

    TuneCacheResult::Hit { fastest_index }
}
fn op()->TuneFn<(),()>{TuneFn{name:"candidate0".into(),marker:PhantomData}}
#[test]fn refusal_before_execution_becomes_a_settled_unmeasured_winner(){
 let client=Client{entered:Arc::new(AtomicUsize::new(0))};let op=op();let mut results=vec![AutotuneResult::error(AutotuneError::Skip{name:op.name.clone()})];
 let mut plan=TunePlan{batches:[Batch{entries:vec![0]}].into()};
 let outcome=Schedule.run_plan(&17u32,&mut plan,&[&op],&(),&client,&mut results);
 assert_eq!(client.entered.load(Ordering::SeqCst),0);assert_eq!(outcome.decided,Some(0));
 assert!(matches!(results[0].outcome,Err(AutotuneError::Unknown{..})));
 let cache=Mutex::new(TuneCache::<u32>::new("audit","cpu-double"));cache.lock().mark_pending(17);
 let request=TuneRequest{key:17,results,log_context:None,pending:vec![],decided:outcome.decided};
 assert!(matches!(future::block_on(process_request(request,&cache,&Mutex::new(Logger))),TuneCacheResult::Hit{fastest_index:0}));
 assert_eq!(cache.lock().settled(&17),Some(0));
}
#[test]fn no_profile_measurement_is_a_different_all_failed_policy(){
 let client=Client{entered:Arc::new(AtomicUsize::new(0))};let op=op();let mut results=vec![AutotuneResult::error(AutotuneError::NotMeasured{name:op.name.clone()})];
 let mut plan=TunePlan{batches:[].into()};
 assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||Schedule.run_plan(&17u32,&mut plan,&[&op],&(),&client,&mut results))).is_err());
}
#[test]fn measured_success_control_is_committed_by_result_index(){
 let cache=Mutex::new(TuneCache::<u32>::new("audit","cpu-double"));
 let results=vec![AutotuneResult::success(AutotuneOutcome{index:4,computation:Score(10)}),AutotuneResult::success(AutotuneOutcome{index:7,computation:Score(2)})];
 let request=TuneRequest{key:17,results,log_context:None,pending:vec![],decided:None};
 assert!(matches!(future::block_on(process_request(request,&cache,&Mutex::new(Logger))),TuneCacheResult::Hit{fastest_index:7}));assert_eq!(cache.lock().settled(&17),Some(7));
}
