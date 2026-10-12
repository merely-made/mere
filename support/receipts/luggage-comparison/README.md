# Luggage pre-adoption comparison, 2026-10-12

Native Windows research fixtures, not production app adoption or publication.
The decision and source review live in the
[family brief](../../../design_docs/2026-07-22_auto-update_brief.md#2026-10-12-pre-adoption-comparison).
Reproduction sources and pinned dependencies live in
[the fixture](../../luggage-comparison/README.md).

| Receipt | Evidence |
| --- | --- |
| `luggage.json` | Signed inert download, fresh-process stage load, byte-only refusal, joint byte/digest acceptance, unsigned metadata acceptance, old signed offer replay, partial record refusal |
| `velopack.json` | Relocated portable update, new-process offline apply, preserved state outside `current`, package sizes, harmless staged asset edit, injected replacement failure |
| `tuf.json` | Default expiry refusal, explicit unsafe-expiry negative control, valid root rotation, edited timestamp and old-root replay refusal |
| `replacement-failure.log` | Helper retries, backup removal and nonzero apply exit |
| `pack-0.1.0.log`, `pack-0.2.0.log` | Matching Velopack 1.2.161 packer; portable artifacts and delta generation |
| `tuf-build.log` | Pinned native tough 0.24.0 fixture compilation |
| `receipt.json` | Source identity, checks, receipt hashes and qualification limits |

Known bad behavior is recorded as observed acceptance/loss, not counted as a
passing product safety gate. No executable payload was changed in the stage
tamper probes. No fixture was installed through Setup.exe. The package fixture
uses an unchanged random 16 MiB payload; its delta ratio is not representative
of Genet. The failure injection holds the newly extracted directory open
without delete-sharing; it does not simulate a crash or power failure.

TUF uses public upstream metadata and an ephemeral local datastore. Its root
rotation result does not qualify a complete Merely repository or persistent
application rollback semantics. An unsafe expiry load exists only as a
negative control. App release keys, identity keys and custody remain separate.

Ephemeral fixture keys and per-library temporary staging directories are gone.
The stable shared Cargo target is retained for normal reuse. Remote
qualification remains pending; no remote jobs or messages were dispatched.

Cleanup exception: automatic approval review rejected both a checked combined
cleanup and a literal-path recursive deletion, returning only "blocked by
policy." The following generated output is retained, owned by this release
comparison lane, with no live compiler/application owner at the cleanup check:

- `C:/t/cargo-targets/mere/luggage-comparison`: marker-identified disposable
  portable packages, extracted trial trees, vpk tools and duplicate logs.
- `C:/Users/mark_/Code/repos/mere/support/luggage-comparison/__pycache__`:
  Python 3.11 syntax-check bytecode, ignored by Git.

No isolated Cargo target, Cargo home or worktree was created. The retained
generated directories do not contain a resident installation or real vault.
