# Mere burn-remote patch

Source: crates.io `burn-remote 0.22.0`, upstream Burn commit
`b8d75ad07f426537594a5beb9309d62d916bcf05`.
License: MIT OR Apache-2.0, unchanged from upstream.

The released server authorizes an Iroh session only at admission. Removing a
session from its manager does not stop the live duplex pump, whose task sender
can still accept work. Mere requires targeted close before owner revocation.

Each session is reserved before application authorization and carries a close
signal observed by the pump. This fences the authorization-to-worker-binding
interval. `IrohRemoteProtocol::sessions` exposes reserved or active session IDs
and opaque credentials; `close_session` targets one pump. Client close uses the
same teardown. The public exports include `ServedSession` and `SessionId`.

Rebased on 2026-10-06 onto pristine stable source. Stable connection errors,
credential types, public server builder, hosted-device ID translation, FIFO
worker admission, response cancellation and global shutdown are preserved.
The patch adds a public `IrohRemoteProtocol::from_endpoint` constructor for
application-owned endpoints and keeps the protocol alive across owner clones.
Stable `HostSpec` and `IrohHost` handle client connections.

The session registry reserves admission before the application authorizer,
keeps failed teardown registered, and reports completion to every closer.
The duplex pump watches owner close, server shutdown and response-writer exit.
Worker completion waits for released device allocations and propagates errors.
The normalized manifest adds an empty workspace, self patch and exact stable
test dependencies; both manifests enable `tokio/macros` for pump selection.
Regression tests cover targeted close, in-flight authorization fencing and
failed cleanup acknowledgements alongside stable upstream's tests.

This rebase alone is not lifecycle acceptance. Full two-peer reclaim/recovery
and headed checks remain gates in the migration plan. Remove the patch when an
upstream release exposes equivalent session control and passes those receipts.

## Teardown completion (2026-10-03, Isometry ruling 508)

Upstream's worker teardown syncs, drops the session's interpreter and calls
`memory_cleanup`, logging sync failures and signalling completion either way.
On pre.4's WGPU backend the frees it issues complete only when their completion
callbacks run, so close was acknowledged while the session's allocations were
still live: the two-peer gate held 10 allocations against a zero baseline.

`server/worker.rs` now waits for the device (`B::sync`) after the release, then
runs one more `memory_cleanup` for the released pages. Every teardown sync
failure is kept, and the worker's completion carries `Result<(), String>`.
`server/session.rs` turns a teardown error into `SessionCompletion::Failed`.
The session stays registered and is never acknowledged clean, so
`close_session` returns the error, and Distillery's `close_run` propagates it.
The wait is device-wide; it may also wait on other sessions' queued work.

A `cfg(test)`-only fault hook (`worker::teardown_fault`) fails that wait for a
chosen session. The unit test
`server::session::teardown_tests::a_failed_teardown_sync_is_reported_and_never_acknowledged_clean`
uses it to check that a clean session closes `Ok` while a faulted one reports
the failure. It exists only in this crate's own test build; the public API is
unchanged. Remove this part with the rest of the patch, or earlier if an
upstream release waits for teardown completion and reports its failures.
