//! The component side of C0: a Wasmtime 45 host that serves the projection
//! session to a guest from a native [`Provider`]. Wasmtime's `bindgen!` stays
//! here, on the component side (D12).

use c0_contract::{Failure as NativeFailure, Provider};
use wasmtime::component::{Component, HasSelf, Linker, Resource, ResourceTable};
use wasmtime::{Engine, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

/// The host's record of one opened session: the provider's handle.
pub struct SessionRep(u64);

wasmtime::component::bindgen!({
    path: "../wit",
    world: "projection-consumer",
    with: { "mere:script/projection-session.session": SessionRep },
});

// The world's `use` puts `failure` and `notice` at the crate root.
use mere::script::projection_session as ps;

fn to_wire(f: NativeFailure) -> Failure {
    match f {
        NativeFailure::Revoked => Failure::Revoked,
        NativeFailure::Expired => Failure::Expired,
        NativeFailure::TooLarge(n) => Failure::TooLarge(n),
        NativeFailure::Refused(s) => Failure::Refused(s),
        NativeFailure::Malformed(s) => Failure::Malformed(s),
    }
}

pub struct HostState<P> {
    wasi: WasiCtx,
    table: ResourceTable,
    pub provider: P,
}

impl<P: Provider + Send + 'static> WasiView for HostState<P> {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

impl<P: Provider> HostState<P> {
    fn handle(&self, s: &Resource<SessionRep>) -> u64 {
        self.table.get(s).map_or(u64::MAX, |r| r.0)
    }
}

impl<P: Provider + Send> ps::Host for HostState<P> {
    fn discover(&mut self) -> Result<Vec<u8>, Failure> {
        self.provider.discover().map_err(to_wire)
    }
    fn open(&mut self, request: Vec<u8>) -> Result<(Resource<SessionRep>, Vec<u8>), Failure> {
        let (h, opened) = self.provider.open(&request).map_err(to_wire)?;
        let res = self
            .table
            .push(SessionRep(h))
            .map_err(|e| Failure::Refused(e.to_string()))?;
        Ok((res, opened))
    }
}

impl<P: Provider + Send> ps::HostSession for HostState<P> {
    fn snapshot(&mut self, s: Resource<SessionRep>, ask: Vec<u8>) -> Result<Vec<u8>, Failure> {
        let h = self.handle(&s);
        self.provider.snapshot(h, &ask).map_err(to_wire)
    }
    fn fetch(&mut self, s: Resource<SessionRep>, address: Vec<u8>) -> Result<Vec<u8>, Failure> {
        let h = self.handle(&s);
        self.provider.fetch(h, &address).map_err(to_wire)
    }
    fn chunk(
        &mut self,
        s: Resource<SessionRep>,
        address: Vec<u8>,
        offset: u64,
        length: u32,
    ) -> Result<Vec<u8>, Failure> {
        let h = self.handle(&s);
        self.provider
            .chunk(h, &address, offset, length)
            .map_err(to_wire)
    }
    fn resume(
        &mut self,
        s: Resource<SessionRep>,
        epoch: u64,
        revision: u64,
    ) -> Result<Vec<u8>, Failure> {
        let h = self.handle(&s);
        self.provider.resume(h, epoch, revision).map_err(to_wire)
    }
    fn intent(&mut self, s: Resource<SessionRep>, call: Vec<u8>) -> Result<Vec<u8>, Failure> {
        let h = self.handle(&s);
        self.provider.intent(h, &call).map_err(to_wire)
    }
    fn suspend(&mut self, s: Resource<SessionRep>) -> Result<(), Failure> {
        let h = self.handle(&s);
        self.provider.suspend(h).map_err(to_wire)
    }
    fn take_notice(&mut self, s: Resource<SessionRep>) -> Option<Notice> {
        let h = self.handle(&s);
        self.provider.take_notice(h).map(|n| Notice {
            epoch: n.epoch,
            revision: n.revision,
        })
    }
    fn wait_notice(&mut self, s: Resource<SessionRep>) -> Result<Notice, Failure> {
        let h = self.handle(&s);
        self.provider
            .wait_notice(h)
            .map(|n| Notice {
                epoch: n.epoch,
                revision: n.revision,
            })
            .map_err(to_wire)
    }
    fn drop(&mut self, s: Resource<SessionRep>) -> wasmtime::Result<()> {
        let rep = self.table.delete(s)?;
        self.provider.close(rep.0);
        Ok(())
    }
}

/// A guest instance serving `provider`.
pub struct Guest<P: 'static> {
    pub store: Store<HostState<P>>,
    pub bindings: ProjectionConsumer,
}

impl<P: Provider + Send + 'static> Guest<P> {
    pub fn load(wasm: &std::path::Path, provider: P) -> wasmtime::Result<Self> {
        let engine = Engine::default();
        let component = Component::from_file(&engine, wasm)?;
        let mut linker = Linker::new(&engine);
        wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;
        ps::add_to_linker::<HostState<P>, HasSelf<HostState<P>>>(&mut linker, |s| s)?;
        let mut store = Store::new(
            &engine,
            HostState {
                wasi: WasiCtxBuilder::new().build(),
                table: ResourceTable::new(),
                provider,
            },
        );
        let bindings = ProjectionConsumer::instantiate(&mut store, &component, &linker)?;
        Ok(Self { store, bindings })
    }
}
