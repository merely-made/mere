#![allow(dead_code,private_interfaces)]
extern crate alloc;
extern crate self as cubecl_environment;
pub mod sync{pub use std::sync::atomic::{AtomicUsize,Ordering};}
mod dry {
//! Running a workload for the compilation and tuning it provokes, without
//! running the workload itself.
//!
//! Under a [`DryRun`] every launch is still expanded, compiled, validated and
//! cached, and is then dropped instead of reaching the device. A warm-up pass
//! then pays for compilation and tuning without also paying for the work that
//! provoked them, which is what makes producing a shippable environment
//! affordable.
//!
//! The launches autotune issues are the exception: they *are* the measurement,
//! so [`RealRun`] opts them back into executing.
//!
//! **Buffers are left as they were**, so anything read back during a dry run is
//! meaningless. It only suits a pass driven by the *shapes* it produces, which
//! is what keys the caches, and never one that branches on a computed value.
//!
//! The decision is made here, once, on the thread that issues the launch.
//! Servers receive the verdict as a [`LaunchMode`] argument rather than
//! deriving it: by the time a launch reaches a server thread, the context that
//! produced it is gone.

use cubecl_environment::sync::{AtomicUsize, Ordering};

/// What a server should do with a launch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchMode {
    /// Compile if needed, then run it. The normal case.
    Execute,
    /// Compile if needed, cache the artifact, and drop the launch.
    ///
    /// A server honoring this must still do everything a first launch does
    /// short of dispatching — expand, compile, validate, populate its caches —
    /// or the pass buys nothing.
    Skip,
}

impl LaunchMode {
    /// Whether the launch should be dropped rather than run.
    pub fn is_skipped(self) -> bool {
        matches!(self, LaunchMode::Skip)
    }
}

/// What to do with a launch issued on this thread, right now.
pub fn launch_mode() -> LaunchMode {
    if !dry_run() || real_run::depth() > 0 {
        return LaunchMode::Execute;
    }

    LaunchMode::Skip
}

/// How many dry runs are open in this process.
///
/// A depth rather than a flag so overlapping guards compose: a swap-and-restore
/// would let one thread's guard end a dry run another thread is still inside,
/// and leave the process dry-running forever once that one dropped in turn.
static DRY_RUN: AtomicUsize = AtomicUsize::new(0);

/// Whether launches are currently compiled and dropped rather than run.
pub fn dry_run() -> bool {
    DRY_RUN.load(Ordering::Relaxed) > 0
}

/// Makes every launch a dry run for as long as it lives, on every thread and
/// every device.
///
/// Overlapping guards compose, so a pass that opens one while another is still
/// open leaves the mode on until the last of them drops.
///
/// The flag is read on the thread issuing a launch, with relaxed ordering, so a
/// launch another thread had already begun issuing may still execute. What is
/// guaranteed is the launches issued by the thread that opened the guard, and
/// every launch issued after other threads observe it.
///
/// This is the only way in: there is deliberately no configuration file or
/// environment variable for it. A dry run left on by accident turns the rest of
/// the process into launches that quietly do nothing and read back
/// uninitialized memory, so its lifetime belongs to a scope in the code that
/// wants it, not to an ambient default nothing in the process can see.
///
/// ```no_run
/// # fn warm_up() {}
/// let _dry_run = cubecl_runtime::dry_run::DryRun::new();
/// warm_up();
/// ```
#[derive(Debug)]
pub struct DryRun {
    _private: (),
}

impl DryRun {
    /// Opens a dry run until the guard drops.
    #[allow(clippy::new_without_default, reason = "a guard is not a value")]
    pub fn new() -> Self {
        DRY_RUN.fetch_add(1, Ordering::Relaxed);
        Self { _private: () }
    }
}

impl Drop for DryRun {
    fn drop(&mut self) {
        DRY_RUN.fetch_sub(1, Ordering::Relaxed);
    }
}

/// Makes the launches issued on this thread execute for real even inside a
/// [`DryRun`], for as long as it lives.
///
/// Autotune holds one: its launches are the measurement a dry run exists to
/// provoke, not the workload it exists to skip. Held across warm-up and samples
/// alike, since a candidate that was never warmed is a candidate measured on
/// its first, slowest run.
///
/// Thread-local, and the thread that matters is the one issuing the launches,
/// which is not always the one that asked for them: a task handed to
/// [`Client::exclusive`](crate::client::Client::exclusive) runs on
/// the device thread. The guard has to live inside that task, alongside the
/// launches it covers, not around the call that submits it.
#[derive(Debug)]
pub struct RealRun {
    _private: (),
}

impl RealRun {
    /// Opts this thread back into executing until the guard drops.
    #[allow(clippy::new_without_default, reason = "a guard is not a value")]
    pub fn new() -> Self {
        real_run::enter();
        Self { _private: () }
    }
}

impl Drop for RealRun {
    fn drop(&mut self) {
        real_run::exit();
    }
}

#[cfg(feature = "std")]
mod real_run {
    use core::cell::Cell;

    std::thread_local! {
        /// How many [`RealRun`](super::RealRun) guards are open on this thread.
        /// A depth rather than a flag: a tunable may itself dispatch through
        /// another tuner, and the inner one finishing must not un-mark the
        /// outer.
        static DEPTH: Cell<usize> = const { Cell::new(0) };
    }

    pub(super) fn depth() -> usize {
        DEPTH.with(|depth| depth.get())
    }

    pub(super) fn enter() {
        DEPTH.with(|depth| depth.set(depth.get() + 1));
    }

    pub(super) fn exit() {
        DEPTH.with(|depth| depth.set(depth.get().saturating_sub(1)));
    }
}

#[cfg(not(feature = "std"))]
mod real_run {
    // No threads to be local to; this keeps the call sites uniform.
    pub(super) fn depth() -> usize {
        0
    }
    pub(super) fn enter() {}
    pub(super) fn exit() {}
}

#[cfg(test)]
mod tests {
    use super::*;
    // `serial_test`'s macro expands to `vec!`, which a `no_std` crate has to
    // bring in itself.
    use alloc::vec;

    /// The guard nests: an inner measurement ending must not cancel the outer
    /// one, or a tunable that dispatches through another tuner would have the
    /// rest of its own measurement dropped.
    #[test]
    fn real_run_nests() {
        assert_eq!(real_run::depth(), 0);
        let outer = RealRun::new();
        {
            let _inner = RealRun::new();
            assert_eq!(real_run::depth(), 2);
        }
        assert_eq!(real_run::depth(), 1, "the outer guard is still open");
        drop(outer);
        assert_eq!(real_run::depth(), 0);
    }

    /// Nothing is skipped outside a dry run, whatever the depth.
    #[test]
    fn launches_execute_by_default() {
        assert_eq!(launch_mode(), LaunchMode::Execute);
        let _real_run = RealRun::new();
        assert_eq!(launch_mode(), LaunchMode::Execute);
    }

    /// The whole contract in one place: in a dry run every launch is dropped
    /// *except* the ones a measurement issues, which are the tuning the mode
    /// exists to keep.
    #[test]
    fn a_dry_run_spares_the_measurements() {
        let _dry_run = DryRun::new();

        assert_eq!(launch_mode(), LaunchMode::Skip);
        {
            let _real_run = RealRun::new();
            assert_eq!(launch_mode(), LaunchMode::Execute, "a measurement runs");
        }
        assert_eq!(launch_mode(), LaunchMode::Skip);
    }

    /// Overlapping guards compose, so neither an inner guard ending nor an
    /// outer one can leave the process in the wrong mode. This is what a
    /// swap-and-restore got wrong across threads.
    #[test]
    fn dry_runs_nest() {
        assert!(!dry_run());
        {
            let _outer = DryRun::new();
            {
                let _inner = DryRun::new();
                assert!(dry_run());
            }
            assert!(dry_run(), "the outer guard is still in force");
        }
        assert!(!dry_run(), "and the process is back to executing");
    }
}

#[cfg(test)]mod audit {
 use super::*;
 #[test]fn moving_real_run_then_dropping_it_leaves_origin_enabled(){
  let _dry=DryRun::new();let real=RealRun::new();
  std::thread::spawn(move||drop(real)).join().unwrap();
  assert_eq!(real_run::depth(),1);assert_eq!(launch_mode(),LaunchMode::Execute);
  real_run::exit();assert_eq!(launch_mode(),LaunchMode::Skip);
 }
 #[test]fn foreign_real_run_drop_disables_another_threads_live_guard(){
  let _dry=DryRun::new();let foreign=RealRun::new();
  std::thread::spawn(move||{
   let own=RealRun::new();assert_eq!(launch_mode(),LaunchMode::Execute);
   drop(foreign);assert_eq!(real_run::depth(),0);assert_eq!(launch_mode(),LaunchMode::Skip);
   drop(own);
  }).join().unwrap();
  assert_eq!(real_run::depth(),1);real_run::exit();
 }
}
}
mod allocation {
use std::{collections::HashMap,sync::{Arc,Mutex}};
#[derive(Default, Clone, Copy, Debug)]
/// The mode of allocation used.
pub enum MemoryAllocationMode {
    /// Use the automatic memory management strategy for allocation.
    #[default]
    Auto,
    /// Use a persistent memory management strategy, meaning that all allocations are for data that is
    /// likely never going to be freed.
    Persistent,
    /// Give every allocation its own device allocation, returned to the driver
    /// once it is freed, outside every pool: for a buffer that exists only for
    /// one measurement (the memory-bandwidth probe) and must neither stay
    /// reserved nor shape the pools' sizing.
    Dedicated,
}


#[derive(Clone,Copy,Hash,PartialEq,Eq,Debug)]struct StreamId(u64);
impl StreamId{fn current()->Self{Self(1)}}
#[derive(Default)]struct Server{modes:HashMap<StreamId,MemoryAllocationMode>,events:Vec<(StreamId,MemoryAllocationMode)>}
impl Server{fn allocation_mode(&mut self,mode:MemoryAllocationMode,stream:StreamId){self.modes.insert(stream,mode);self.events.push((stream,mode));}}
#[derive(Default,Clone)]struct Device(Arc<Mutex<Server>>);
impl Device{fn submit(&self,call:impl FnOnce(&mut Server)){call(&mut self.0.lock().unwrap());}}
#[derive(Clone)]struct Client{device:Device,pinned:Option<StreamId>}
impl Client{
 fn stream_id(&self)->StreamId{self.pinned.unwrap_or_else(StreamId::current)}
 fn observed_mode(&self)->MemoryAllocationMode{self.device.0.lock().unwrap().modes.get(&self.stream_id()).copied().unwrap_or_default()}
fn allocation_window<'a, Re: Send, Input: Send, F: FnOnce(Input) -> Re + Send + 'a>(
        &'a self,
        mode: MemoryAllocationMode,
        input: Input,
        task: F,
    ) -> Re {
        let stream_id = StreamId::current();

        self.device.submit(move |server| {
            server.allocation_mode(mode, stream_id);
        });

        // All tasks created on the same stream allocate in this mode.
        let output = task(input);

        self.device.submit(move |server| {
            server.allocation_mode(MemoryAllocationMode::Auto, stream_id);
        });

        output
    }
}
#[cfg(test)]mod cases{
use super::*;
#[test]fn ordinary_stream_window_control_routes_and_resets(){
 let client=Client{device:Device::default(),pinned:None};let observed=client.clone();
 client.allocation_window(MemoryAllocationMode::Persistent,(),move|()|assert!(matches!(observed.observed_mode(),MemoryAllocationMode::Persistent)));
 assert!(matches!(client.observed_mode(),MemoryAllocationMode::Auto));assert_eq!(client.device.0.lock().unwrap().events.len(),2);
}
#[test]fn pinned_client_window_changes_other_stream_not_its_allocation_stream(){
 let client=Client{device:Device::default(),pinned:Some(StreamId(2))};let observed=client.clone();
 client.allocation_window(MemoryAllocationMode::Persistent,(),move|()|assert!(matches!(observed.observed_mode(),MemoryAllocationMode::Auto)));
 let server=client.device.0.lock().unwrap();assert_eq!(server.events.len(),2);assert!(server.events.iter().all(|(stream,_)|*stream==StreamId(1)));
}
#[test]fn unwinding_callback_skips_reset_command(){
 let client=Client{device:Device::default(),pinned:None};
 let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||client.allocation_window(MemoryAllocationMode::Dedicated,(),|()|panic!("synthetic caller unwind"))));
 assert!(result.is_err());assert!(matches!(client.observed_mode(),MemoryAllocationMode::Dedicated));assert_eq!(client.device.0.lock().unwrap().events.len(),1);
}
}
}
mod observation{
use std::{sync::{Arc,Mutex},sync::atomic::{AtomicBool,Ordering}};
struct RwLock<T>(std::sync::RwLock<T>);
impl<T>RwLock<T>{const fn new(v:T)->Self{Self(std::sync::RwLock::new(v))}fn read(&self)->std::sync::RwLockReadGuard<'_,T>{self.0.read().unwrap()}fn write(&self)->std::sync::RwLockWriteGuard<'_,T>{self.0.write().unwrap()}}
trait LaunchObserver:Send+Sync{fn launched(&self,kernel:&'static str);}
static OBSERVING:AtomicBool=AtomicBool::new(false);
static OBSERVER:RwLock<Option<Arc<dyn LaunchObserver>>>=RwLock::new(None);
pub struct LaunchObservation {
    previous: Option<Arc<dyn LaunchObserver>>,
}

impl LaunchObservation {
    /// Installs `observer` until the guard drops.
    pub fn new(observer: Arc<dyn LaunchObserver>) -> Self {
        let previous = OBSERVER.write().replace(observer);
        // Last, so the flag is never set over an empty slot.
        OBSERVING.store(true, Ordering::Relaxed);
        Self { previous }
    }
}

impl Drop for LaunchObservation {
    fn drop(&mut self) {
        let previous = self.previous.take();
        let still_observed = previous.is_some();
        *OBSERVER.write() = previous;
        // Last, so the flag is never cleared while an observer is still
        // installed: the launches this guard covers are the ones it must not
        // miss, and a notify that reads the flag in between finds an empty
        // slot and does nothing.
        OBSERVING.store(still_observed, Ordering::Relaxed);
    }
}

pub(crate) fn is_observing() -> bool {
    OBSERVING.load(Ordering::Relaxed)
}
pub(crate) fn notify_launch(kernel: &'static str) {
    if !OBSERVING.load(Ordering::Relaxed) {
        return;
    }
    if let Some(observer) = OBSERVER.read().as_ref() {
        observer.launched(kernel);
    }
}
#[cfg(test)]mod cases{
use super::*;
#[derive(Default)]struct Recorder(Mutex<Vec<&'static str>>);
impl LaunchObserver for Recorder{fn launched(&self,k:&'static str){self.0.lock().unwrap().push(k);}}
#[test]fn reverse_order_control_restores_and_ends_observers(){
 let a=Arc::new(Recorder::default());let b=Arc::new(Recorder::default());
 let outer=LaunchObservation::new(a.clone());let inner=LaunchObservation::new(b.clone());
 notify_launch("inner");drop(inner);notify_launch("outer");drop(outer);notify_launch("ended");
 assert_eq!(*a.0.lock().unwrap(),vec!["outer"]);assert_eq!(*b.0.lock().unwrap(),vec!["inner"]);assert!(!is_observing());
}
#[test]fn documented_taken_order_loses_active_then_resurrects_ended_observer(){
 let a=Arc::new(Recorder::default());let b=Arc::new(Recorder::default());
 let first=LaunchObservation::new(a.clone());let second=LaunchObservation::new(b.clone());
 drop(first);notify_launch("second_still_alive");assert!(b.0.lock().unwrap().is_empty());
 drop(second);notify_launch("both_ended");assert_eq!(*a.0.lock().unwrap(),vec!["both_ended"]);assert!(is_observing());
 *OBSERVER.write()=None;OBSERVERING_CLEANUP();
}
#[allow(non_snake_case)]fn OBSERVERING_CLEANUP(){OBSERVING.store(false,Ordering::Relaxed);}
}
}
