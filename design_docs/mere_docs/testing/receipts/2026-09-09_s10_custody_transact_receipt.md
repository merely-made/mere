# S10 custody lifecycle: `Backend::transact` landed, 2026-09-09

Mark's ruling (2026-09-09, pricing in `scratch/s10-custody-20260909/REPORT.md`):
option B, a transactional read-then-write method on Muniment's `Backend`
trait, not a serialized-writer actor. This receipt covers that method
landing in the actual `muniment` crate — not a scratch research adapter like
the 2026-09-08 custody-backend probe this one supersedes for the
production-boundary question; that probe's `src/lib.rs` scenarios are ported
here as real tests against the real crate.

Work happened in the worktree `C:/Users/mark_/Code/worktrees/mere-s10-custody-20260909`,
branch `slice/s10-custody-20260909`, based on mere main `725b0f35`. Landing
commit: `9ba9f790ad856aefb6d603f28c471d177504b79f`. Muniment crate tree hash at
that commit (`git rev-parse HEAD:crates/eidetic/muniment`):
`48896a86c79d50ff235351244f8743637ca10c55`.

## Trait signature landed

```rust
pub trait TransactionReader {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StoreError>;
    fn list(&self, prefix: &str) -> Result<Vec<String>, StoreError>;
}

#[cfg(not(target_arch = "wasm32"))]
pub type TransactFn = Box<dyn FnOnce(&dyn TransactionReader) -> Vec<WriteOp> + Send>;
#[cfg(target_arch = "wasm32")]
pub type TransactFn = Box<dyn FnOnce(&dyn TransactionReader) -> Vec<WriteOp>>;

// on Backend:
async fn transact(&self, f: TransactFn) -> Result<(), StoreError> {
    Err(StoreError::NotTransactional) // default
}
```

Boxed `dyn FnOnce`, not a generic `impl FnOnce`, because `Backend` is used as
`dyn Backend + Send + Sync` elsewhere in the crate (`backend.rs` tests) and a
generic method parameter is not object-safe.

## Per-backend outcome

| Backend | `transact` | Notes |
|---|---|---|
| `RedbBackend` | Honest | One `begin_write`; a `Reader` wraps `&Table` for `get`/`range` during the closure call, then the same `table` is borrowed mutably for insert/remove (sequential borrows, not overlapping); one `commit`. |
| `MemoryBackend` | Honest | The single `Mutex` stays locked from the closure's reads through the writes it returns. |
| `IndexedDbBackend` | Honest (wasm32 only) | One `Readwrite` transaction; `get_all`/`get_all_keys` snapshot the store into memory before the closure runs (synchronous reads, no `.await` inside the closure), writes issue against the same still-open transaction before `oncomplete`. |
| `ZipBackend` | Typed refusal | `StoreError::NotTransactional`, documented: the in-process `Mutex` cannot promise anything across processes, and two processes rewriting the same archive is last-writer-wins per the module's own doc comment. |
| Default (unimplemented) | Typed refusal | `StoreError::NotTransactional`, so an external `Backend` implementor keeps compiling without adding this method. |

## Cross-process finding (the one thing the pricing left unverified)

`REPORT.md` §4/§5 left open whether redb's write lock is cross-process on
Windows. Verified two ways:

1. **Source**: redb 2.6.3,
   `src/tree_store/page_store/file_backend/windows.rs::FileBackend::new` calls
   Win32 `LockFile` (whole-file, exclusive) the moment a `Database` is
   created or opened, mapping a failed lock to
   `DatabaseError::DatabaseAlreadyOpen`; `Drop for FileBackend` calls
   `UnlockFile`. The lock is held for the handle's whole lifetime, not
   acquired and released per `begin_write` — stronger than "concurrent
   `begin_write` blocks."
2. **Runtime** (`examples/redb_cross_process_lock.rs`, real `std::process::Command`
   child, not a thread): built and run directly, native, this machine.

   ```
   PARENT: holding an open Database handle on "...\muniment_cross_process_lock_21108.redb"
   CHILD: Database::open FAILED: Database already open. Cannot acquire lock.
   PARENT: first child (spawned while the parent's handle is open) exited exit code: 0
   PARENT: dropped its handle (releases the Windows LockFile)
   CHILD: Database::open SUCCEEDED (lock was not held, or not cross-process)
   PARENT: second child (spawned after release) exited exit code: 0
   ```

   Confirmed: a second OS process cannot even open the database file while
   the first process holds it open, and can once the first drops its handle.
   `transact`'s single-writer guarantee holds across processes for
   `RedbBackend` on Windows.

## Commands, exit codes, environment

`CARGO_TARGET_DIR=C:/Users/mark_/Code/.targets/s10-custody-20260909`, all
`cargo` invocations `--offline`, run from the worktree above.
`rustc 1.97.1 (8bab26f4f 2026-07-14)`, `cargo 1.97.1 (c980f4866 2026-06-30)`.

| Command | Exit |
|---|---|
| `cargo check -p muniment --all-features` | 0 |
| `cargo test -p muniment --all-features -- --nocapture` | 0 (59 passed, 0 failed) |
| `cargo check -p muniment --no-default-features` | 0 |
| `cargo check -p muniment --no-default-features --features redb` | 0 |
| `cargo check -p muniment --no-default-features --features zip` | 0 |
| `cargo check -p muniment --no-default-features --features json,redb,zip` | 0 |
| `cargo check -p muniment --no-default-features --features postcard` | 0 |
| `cargo check -p muniment --no-default-features --features json,indexeddb --target wasm32-unknown-unknown` | 0 |
| `cargo build -p muniment --example redb_cross_process_lock --features redb` | 0 |
| `.../debug/examples/redb_cross_process_lock.exe` (direct, not via `cargo run`) | 0 |

Test breakdown (59 total, all passing): the crate's pre-existing 56 tests
unchanged in behavior, plus 3 new `custody_transact_tests` groups —
`memory_backend`/`redb_backend` (3 scenarios each: pre-transaction competing
claim refused, post-commit late claim sees no dangling reference, the
apply-alone negative control retained as a documented regression) and the two
typed-refusal tests (default, zip).

## Environment residual (out of muniment's scope, recorded not silently fixed)

The worktree's local, gitignored `.cargo/config.toml` had a stale
`[patch]` entry, `"servo-paint" = { path = ".../genet/components/paint" }`,
pointing at a directory that no longer exists in the current genet checkout
(it looks renamed to `webgl-essl`). This blocked every `cargo check`/`test` in
this worktree before any muniment work could run. Genet is out of this
slice's scope, so the entry was commented out in this worktree's local config
only (not committed; does not touch the genet repo or the other active mere
session) rather than repaired. Whoever next touches genet's paint rename
should also update `mere/.cargo/config.toml` on any machine that still has the
old entry.

## Residuals

- Turnstone (or any consumer) is not wired to `transact`; this slice ends at
  the store boundary per instruction.
- Process-kill recovery (an orphaned reference from a killed writer) remains
  open, as it was before this slice — `transact` prevents a *torn* write, not
  a *crash between two separate* `transact` calls (e.g. a claim commits, then
  the process dies before the paired transfer).
- Portable owner-key encoding and disclosure policy remain open, as noted in
  `REPORT.md`.
- `IndexedDbBackend::transact` is check-compiled only (wasm32, offline); it
  has no browser-driven test in this crate, consistent with the rest of that
  backend's test coverage (none of `IndexedDbBackend`'s other methods have
  in-crate browser tests either — `ports/muniment-opfs-probe` is where browser
  runs happen for this backend family).
