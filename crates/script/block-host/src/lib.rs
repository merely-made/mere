// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The `block-guest` component host.
//!
//! A guest block adapter is a pure component: it imports only `log` and `caps` and exports
//! `block-adapter` (`describe`, `choices`, `apply`, `run`, `view`). Everything crosses the
//! boundary as copied values. This crate instantiates the world and exposes the five calls
//! with the WIT's own generated types; the pane's `BlockAdapter` wraps it. It knows no
//! adapter and no document.
//!
//! Containment mirrors app-host and document-host: [`guarded_engine`] turns on epoch
//! interruption, [`Watchdog`] bumps the epoch so a runaway turn hits its deadline, and
//! `StoreLimits` caps allocation. A misbehaving component traps; the host survives, and the
//! caller drops the instance.

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use wasmtime::component::{Component, HasSelf, Linker, ResourceTable};
use wasmtime::{Config, Engine, Store, StoreLimits};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

wasmtime::component::bindgen!({
    // The shared `mere:script` contract (one WIT, many worlds).
    path: "../wit",
    world: "block-guest",
    exports: { default: async },
    imports: { default: async },
});

/// The generated value types of `block-adapter`, for callers to build and read.
pub use exports::mere::script::block_adapter as wit;

/// Most bytes a guest component may be (the capsule cap).
pub const MAX_COMPONENT_BYTES: usize = 16 * 1024 * 1024;
/// Most bytes of one view payload (the projection session's 1 MiB cap).
pub const MAX_PAYLOAD_BYTES: usize = 1024 * 1024;

pub struct BlockHostState {
    logs: Vec<String>,
    granted: Vec<String>,
    wasi: WasiCtx,
    table: ResourceTable,
    limits: StoreLimits,
}

impl crate::mere::script::log::Host for BlockHostState {
    async fn log(&mut self, message: String) {
        self.logs.push(message);
    }
}

impl crate::mere::script::caps::Host for BlockHostState {
    async fn granted(&mut self) -> Vec<String> {
        self.granted.clone()
    }
}

impl WasiView for BlockHostState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

/// An engine with epoch interruption on, so a runaway turn can be trapped.
pub fn guarded_engine() -> wasmtime::Result<Engine> {
    let mut config = Config::new();
    config.epoch_interruption(true);
    Engine::new(&config)
}

/// Bumps an engine's epoch on a fixed tick so a deadline is actually reached.
/// Stops and joins on drop, so a run can never leak the thread.
pub struct Watchdog {
    stop: Arc<AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl Watchdog {
    pub fn start(engine: Engine, tick: Duration) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let watch_stop = stop.clone();
        let handle = std::thread::spawn(move || {
            while !watch_stop.load(Ordering::Relaxed) {
                std::thread::sleep(tick);
                engine.increment_epoch();
            }
        });
        Self {
            stop,
            handle: Some(handle),
        }
    }
}

impl Drop for Watchdog {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

/// One live `block-guest` instance. Every call arms a fresh epoch deadline of `deadline`
/// ticks when one was given, so a turn that spins traps instead of hanging the pane.
pub struct BlockGuestHost {
    store: Store<BlockHostState>,
    bindings: BlockGuest,
    deadline: Option<u64>,
}

impl BlockGuestHost {
    /// Instantiate a component from the exact bytes supplied: a host reads and verifies a
    /// body once, then passes those same bytes here. `granted` is what `caps.granted()`
    /// reports to the guest.
    pub async fn attach_bytes(
        engine: &Engine,
        bytes: &[u8],
        granted: Vec<String>,
        limits: StoreLimits,
        deadline: Option<u64>,
    ) -> wasmtime::Result<Self> {
        if bytes.len() > MAX_COMPONENT_BYTES {
            wasmtime::bail!("component exceeds {MAX_COMPONENT_BYTES} bytes");
        }
        let component = Component::from_binary(engine, bytes)?;
        let mut linker: Linker<BlockHostState> = Linker::new(engine);
        wasmtime_wasi::p2::add_to_linker_async(&mut linker)?;
        crate::mere::script::log::add_to_linker::<_, HasSelf<BlockHostState>>(&mut linker, |s| s)?;
        crate::mere::script::caps::add_to_linker::<_, HasSelf<BlockHostState>>(&mut linker, |s| s)?;
        let mut store = Store::new(
            engine,
            BlockHostState {
                logs: Vec::new(),
                granted,
                wasi: WasiCtxBuilder::new().build(),
                table: ResourceTable::new(),
                limits,
            },
        );
        store.limiter(|s| &mut s.limits);
        if let Some(ticks) = deadline {
            store.set_epoch_deadline(ticks);
        }
        let bindings = BlockGuest::instantiate_async(&mut store, &component, &linker).await?;
        Ok(Self {
            store,
            bindings,
            deadline,
        })
    }

    pub fn attach_bytes_blocking(
        engine: &Engine,
        bytes: &[u8],
        granted: Vec<String>,
        limits: StoreLimits,
        deadline: Option<u64>,
    ) -> wasmtime::Result<Self> {
        pollster::block_on(Self::attach_bytes(engine, bytes, granted, limits, deadline))
    }

    pub fn attach_path_blocking(
        engine: &Engine,
        path: &Path,
        granted: Vec<String>,
        limits: StoreLimits,
        deadline: Option<u64>,
    ) -> wasmtime::Result<Self> {
        let bytes = std::fs::read(path)?;
        Self::attach_bytes_blocking(engine, &bytes, granted, limits, deadline)
    }

    fn arm(&mut self) {
        if let Some(ticks) = self.deadline {
            self.store.set_epoch_deadline(ticks);
        }
    }

    /// The guest's captured `log` output, oldest first.
    pub fn logs(&self) -> &[String] {
        &self.store.data().logs
    }

    pub fn describe(&mut self) -> wasmtime::Result<wit::Claim> {
        self.arm();
        pollster::block_on(self.bindings.mere_script_block_adapter().call_describe(&mut self.store))
    }

    pub fn choices(
        &mut self,
        block: &wit::Block,
        command: &str,
    ) -> wasmtime::Result<Result<Vec<wit::Change>, String>> {
        self.arm();
        pollster::block_on(self.bindings.mere_script_block_adapter().call_choices(
            &mut self.store,
            block,
            command,
        ))
    }

    pub fn apply(
        &mut self,
        block: &wit::Block,
        command: &str,
        changes: &[wit::Change],
    ) -> wasmtime::Result<Result<Vec<wit::Attr>, String>> {
        self.arm();
        pollster::block_on(self.bindings.mere_script_block_adapter().call_apply(
            &mut self.store,
            block,
            command,
            changes,
        ))
    }

    pub fn run(
        &mut self,
        block: &wit::Block,
        command: &str,
        arg: &str,
    ) -> wasmtime::Result<Result<wit::RunResult, String>> {
        self.arm();
        pollster::block_on(self.bindings.mere_script_block_adapter().call_run(
            &mut self.store,
            block,
            command,
            arg,
        ))
    }

    /// A view. The two payloads are bounded here before the caller decodes them.
    pub fn view(
        &mut self,
        block: &wit::Block,
        ask: &wit::ViewAsk,
    ) -> wasmtime::Result<Result<wit::ViewResult, String>> {
        self.arm();
        let result = pollster::block_on(self.bindings.mere_script_block_adapter().call_view(
            &mut self.store,
            block,
            ask,
        ))?;
        Ok(result.and_then(|view| {
            if view.scene.len() > MAX_PAYLOAD_BYTES || view.sheet.len() > MAX_PAYLOAD_BYTES {
                Err("The block adapter returned a view over the 1 MiB payload cap".to_string())
            } else {
                Ok(view)
            }
        }))
    }
}
