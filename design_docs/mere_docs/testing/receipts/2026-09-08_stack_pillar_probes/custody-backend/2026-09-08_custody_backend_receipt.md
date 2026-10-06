# S10 custody backend experiment, 2026-09-08

Three assertions pass against actual Muniment with its redb backend and BLAKE3
blob identities, native Windows x86_64, Rust 1.97.1, default json plus redb
features, dev test profile. Engine, renderer and server: none. `src/lib.rs` is
the named regression manifest; `run.json` records exact command, lock, runner,
source and compiler identity. The source is an unchanged Muniment subtree from
Mere's revision in `source-manifest.json`, placed under a minimal scratch
workspace whose manifest is supplied as `workspace.toml`.

- Equal bytes yield one blob while two same-class owners retain separate
  references. Transferring one owner to recycle, releasing the other, reopening
  the real database and restoring ownership preserves bytes. Releasing the last
  owner then permits collection. Self-transfer preserves the reference.
- A collection proposal made before a new claim is refused when the single-owner
  adapter rechecks references at application time.
- The negative control applies a precomputed delete batch after the claim. The
  real backend deletes the blob and leaves the reference dangling. Atomic write
  batches do not make an earlier read current.

This is a research adapter, not a production custody service or Turnstone
capture receipt. The commands are deterministically serialized by one adapter
in these tests; another writer bypassing it can still race its read/delete.
Product implementation therefore needs one custody writer covering claims,
transfers and collection, or a transactional conditional read/write API.
The current generic Backend::apply is insufficient to establish that boundary
on its own. Turnstone's existing deposit-and-download writer cannot simply be
called for capture because it also creates a user-visible file.

Reopen is verified. Process-kill recovery, injected I/O failures, competing
writers, portable owner-key encoding, disclosure policy, envelope persistence
and headed capture remain untested. Probe keys use controlled fixed labels;
this string format is not a proposed public identity encoding.

For replay, extract the pinned `crates/eidetic/muniment` subtree under `source`,
copy `workspace.toml` to `source/Cargo.toml`, and run Cargo against the supplied
manifest and lock. `run.py` expects the scratch sibling private Cargo home;
change only cache/target paths for another host. Exact source hashes allow the
extracted tree to be checked independently. Initial private-cache dependency
fetch succeeded; the final receipt is offline and locked.
