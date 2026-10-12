# Release comparison fixtures

Research only. These fixtures do not wire an updater into any product, install
Djinn, open custody, or publish a feed. Their known-gap observations describe
the measured implementations, not acceptable product behavior.

Use Python 3.11+, Rust, and a .NET 8 runtime on Windows. Cargo output uses the
existing `C:/t/cargo-targets/mere`, one compiler job, and this independent
Cargo.lock. The product workspace lock and Hocket pins stay untouched.

```powershell
$env:LUGGAGE_COMPARISON_VERSION = '0.1.0'
cargo build --locked --manifest-path support/luggage-comparison/Cargo.toml --bin merely-release-comparison --target-dir C:/t/cargo-targets/mere --jobs 1
C:/t/cargo-targets/mere/debug/merely-release-comparison.exe luggage
```

`luggage` creates signed inert artifacts with ephemeral in-memory keys. It
checks real signed download, fresh-process stage reload, artifact-only versus
artifact-and-record edits, offer replay, and unsigned/edited manifest refusal.
It never calls `install` on those bytes.

For the Windows package probes, prepare the named generated-output directory
`C:/t/cargo-targets/mere/luggage-comparison`, containing `.generated-output`,
then download `vpk` 1.2.161 from NuGet and extract it beneath `vpk/`:

```powershell
python support/luggage-comparison/windows.py
```

The driver creates two portable packages, relocates v0.1 into a path with a
space, downloads v0.2, hides the feed and applies from a new process. It also
edits one inert asset inside the staged ZIP and injects a replacement failure
by opening the newly extracted directory without Windows delete-sharing.
The helper can still extract but cannot rename that directory into `current`.
Only this fixture's generated directories are affected. It does not simulate
power loss. Reruns need those disposable fixture directories removed first.
The random, unchanged 16 MiB payload makes the handle injection reliable;
delta size from this toy payload is not an estimate for a Genet application.

For TUF:

```powershell
cargo build --locked --manifest-path support/luggage-comparison/Cargo.toml --features tuf --bin tuf-comparison --target-dir C:/t/cargo-targets/mere --jobs 1
C:/t/cargo-targets/mere/debug/tuf-comparison.exe C:/Users/mark_/Code/repos/mere/support/luggage-comparison/fixtures
```

Public upstream fixtures are pinned and hashed in `fixtures/provenance.json`.
They come from `awslabs/tough` revision
`98d8eb8b2ce63515d9b4981c938ef6453c5b5771`, copyright Amazon.com, Inc. or its
affiliates, under MIT OR Apache-2.0. The MIT license is retained alongside
them. They contain public trust metadata, not private release keys.

The TUF probe uses tough 0.24.0, local filesystem transport and a temporary
datastore. Expiration is safe by default. One explicitly unsafe expiration
load is the negative control; it is not a product configuration. The probe
also checks valid root rotation, unsigned timestamp edits and replaying an
old signed root at the next root URL. These are library probes, not a complete
Merely TUF deployment or proof of persisted rollback protection across boots.

Results and limits are recorded in
[the family update brief](../../design_docs/2026-07-22_auto-update_brief.md#2026-10-12-pre-adoption-comparison)
and `support/receipts/luggage-comparison/`. Copy receipts before removing
marker-identified disposable tool/package output; never clear the shared
Cargo target or another lane's files.
