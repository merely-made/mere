# Crypto Generation Unification

**Date:** 2026-08-10
**Status (2026-10-06):** landed. The unification landed 2026-08-10
(`1468e1b1`); ruling S44 of the [stack seams plan](2026-10-04_stack_seams_plan.md)
reopened it for three first-party `sha2 = "0.10"` pins, and that follow-on
landed the same day: djinn and the distillery session fixture take the 0.11
row (`d8291708`, `3032a629`), and Pelt desktop's pin, an orphan since L5,
was removed rather than repinned (ruling C1, `d8291708`). Genet's own 0.10 row
(fleece, genet-scripted) is genet's move.
**Anchors:** [crypto stack decision](../technical_architecture/2026-08-10_crypto_stack_decision.md),
[dependency footprint brief](../../2026-07-04_dependency_footprint_brief.md)
(which named this migration unit on 2026-07-04 and did not execute it)

## What was wrong

Both RustCrypto generations were resolved in the workspace at once. Retinue had
moved to the `digest` 0.11 row unilaterally; mere's own manifests were still on
0.10. Two implementations of the same primitives were compiling into one graph,
and "which SHA-256 is this?" had an answer that depended on the call site.

## What changed

Seven pins across six manifests, all in one commit because the row moves
together:

| Manifest | From | To |
|---|---|---|
| root `[workspace.dependencies]` | `sha2 = "0.10"` | `"0.11"` |
| root `[workspace.dependencies]` | `hkdf = "0.12"` | `"0.13"` |
| `personae` | `chacha20poly1305 = "0.10"` | `"0.11"` |
| `eidetic-core` | `chacha20poly1305 = "0.10"` | `"0.11"` |
| `session-runtime` | `chacha20poly1305 = "0.10"` | `"0.11"` |
| `castellan` | `hmac = "0.12"` | `"0.13"` |
| `castellan` | `sha1 = "0.10"` | `"0.11"` |

`mere-transport` and `graphshell` took the workspace pins and needed no edit.

**One source change, in castellan.** `new_from_slice` moved from `Mac` to
`KeyInit`. Worth checking rather than mechanically fixing: `KeyInit`'s *default*
`new_from_slice` demands an exact key size, which for HMAC would be the hash's
block size, and OTP secrets are 20, 32, or 64 bytes. `HmacCore` overrides it and
still accepts any length (`hmac-0.13/src/block_api.rs:53`), so the change is
genuinely just the trait name. The RFC vectors confirm it.

## The invariant that mattered

Sealed records are real data: vaults, wallets, and woodshed sessions on disk
right now were written by `personae::seal::seal_bytes`. A round-trip test cannot
catch a format change, because it writes and reads the *new* format happily.

So `the_sealed_format_is_pinned_across_crate_generations` asserts the exact
bytes of a fixed-nonce seal, and it was **checked under both generations**: the
manifest was temporarily reverted to `chacha20poly1305 = "0.10"`, the test run
again, and the bytes were identical. XChaCha20-Poly1305 is standardized, so this
is the algorithm speaking rather than the crate, but that is now a fact on
record instead of a reasonable assumption.

The pin stays as a guard on the next bump.

## Verified

`personae` 91 + 88 + doctests, `session-runtime` 246, `mere-eidetic` 81 + 4 + 4
+ 2 + 3, `castellan` 26, `knot` 5, all green. `mere-transport --all-features`
and `graphshell` compile. Castellan's 26 include every published RFC 4226 /
6238 / 4648 vector, which is the strongest available statement that the HMAC
change altered nothing.

## Residue, deliberately not chased

Every remaining `digest` 0.10 consumer is **third-party transitive**:
p2panda-encryption, snow, ssh-key, sqlx, wasmtime, rsa, p256/384/521,
elliptic-curve, bcrypt-pbkdf, oxrdf. We do not own those manifests, and the
decision doc's rule is that a transitive dep on the other row is tolerable
while a first-party one is not.

**Corrected 2026-10-06 (S14 pass):** no longer true. At mere `535bca11` three
first-party manifests pin `sha2 = "0.10"`: `ports/djinn/Cargo.toml` (line 78,
added in `37706afb`, 2026-09-13), `ports/pelt/desktop/Cargo.toml` (line 186,
commented "this carries genet's 0.10 line") and
`ports/distillery/probe/session-fixture/Cargo.toml` (line 21, added in
`aa121f03`, 2026-08-24). The base Cargo.lock lists `djinn` 0.0.2 and
`pelt-desktop` 0.2.0 among sha2 0.10.9's dependents. The `session-runtime` row
in the table above is now pandect.

**Annotated 2026-10-06 (S44 follow-on):** the three pins are gone; see
"Reopened 2026-10-06" below.

Two items are ours but out of this repo's reach:

- **`misfin` 0.0.4** pulls sha2 0.10 through its own published manifest. It
  lives in the smolweb workspace and is held in stewardship, so bumping it is a
  separate repo's commit and a republish.
- **`ed25519-dalek` is still in the graph at both 2.2.0 and 3.0.0**, via
  transitive pins (p2panda's fork line, misfin). The archived 2026-07-15
  iroh/p2panda bump plan is the record of how much work unifying that is.

**Annotated 2026-10-06:** a third. Genet's workspace row is still
`sha2 = "0.10"` (genet `Cargo.toml:282` at `90c5ef50`), and fleece 0.5.0 and
genet-scripted 0.2.0 take it into mere's graph at the pinned genet rev
`d851a9db`. Moving it is genet's commit, and by the decision doc's rule 1 it
moves the whole row.

`argon2` stays at 0.5.3: 0.6 is release-candidate only, and a password-hash
change is a stored-format change that wants its own migration note.
`signature` 2 rides with `ssh-key` 0.6 rather than moving alone.

## Reopened 2026-10-06: the three `sha2` 0.10 pins

Ruling S44 of the [stack seams plan](2026-10-04_stack_seams_plan.md): repin
all three, as a follow-on task outside the S14 documentation pass. Done when:

- [x] `ports/djinn/Cargo.toml` takes the 0.11 row; *done 2026-10-06
      (`d8291708`): `sha2.workspace = true`, no source change.*
- [x] `ports/distillery/probe/session-fixture/Cargo.toml` takes the 0.11 row;
      *done 2026-10-06 (`3032a629`): `sha2 = "0.11"` stated inline, because
      the fixture is a standalone workspace and cannot inherit the row.*
- [x] `ports/pelt/desktop/Cargo.toml` takes the 0.11 row, which may wait on
      genet's own move to 0.11. *Closed 2026-10-06 by ruling C1 (`d8291708`):
      the line was an orphan and was removed; genet did not need to move
      first.*

**Ruling C1.** *S44 said repin Pelt desktop's sha2 0.10 pin to 0.11, possibly
after genet moves. It turns out to be an orphan: L5 (`f014052a`, 2026-08-18)
moved the `<script integrity>` code into genet-scripted and dropped
`dep:sha2` from the `scripted` feature, but left the declaration; nothing
enables it (0 enablers in mere's manifests) and no Pelt source names sha2.
Genet need not move first. What happens to the line?* Options: remove sha2
only; repin to the workspace row; remove all three L5 orphans (sha2,
encoding_rs, base64). Mark: **"Remove sha2 only (Recommended)"**. Follows: the
`sha2` line and its "genet's 0.10 line" comment go, the integrity comment is
trimmed to the two deps left, and S44's third done-condition closes as removed
rather than repinned. `encoding_rs` and `base64` stay. *Reading, not ruled*:
the trimmed comment says what the two were for and that no feature enables
them now, rather than restating a verification they no longer perform.

**Ruling C2.** *Where should this answer be recorded as a ruling? The stack
seams plan holds S44 but another session is committing rounds to it about
every 30 minutes (S76-S77 at 22:03 today), so a new S-number could collide.*
Options: in the crypto plan; as S78 in the stack seams plan. Mark: **"In the
crypto plan (Recommended)"**. Follows: this plan carries C1 and C2, and S44's
follow-on leaves the stack seams plan unedited; S44's "Follows" already points
here. The stack seams plan reached S82 the same evening, so S78 would have
collided.

### Findings (2026-10-06)

- **djinn.** The one call site, `identity_fingerprint`
  (`ports/djinn/src/resident_site.rs:964`), converts `Sha256::digest(..)` into
  `[u8; 32]` with `.into()`; 0.11's `hybrid_array::Array<u8, U32>` keeps that
  conversion, so the repin compiled unchanged.
- **Session fixture.** It declares an empty `[workspace]` and commits its own
  `Cargo.lock`, so rule 2's workspace pin cannot reach it; the row is stated
  inline with a one-line comment. Its lock already carried sha2 0.11.0, for
  ed25519-dalek 3; the repin added only `const-oid` 0.10.2 under digest
  0.11.3 (sha2 0.11's default `oid` feature), the same shape the root lock
  has. Its run script (`run-model-session.ps1`) builds it from a directory
  outside the checkout, so that gitignored local patch redirects cannot leak
  into the committed lock; the gates here did the same.
- **Pelt desktop.** `encoding_rs` and `base64` are orphans of the same L5
  move: no `dep:` reference and no source use. They stay under C1.
- **Lock.** At `d8291708` the root lock moved djinn to sha2 0.11.0 and dropped
  pelt-desktop's sha2 edge; nothing else changed. After the merge at
  `1d87808a`, sha2 0.10.9's twenty dependents are all third-party apart from
  misfin and genet's fleece and genet-scripted.

## Progress

- **2026-10-06 (S14 pass).** Status and claims corrected against the tree at
  mere 535bca11, from the D2 record in
  support/doc-audit/d2/batch_43_s14_phase_b5.md: the status reopened under
  ruling S44, the Residue claim corrected with the three first-party
  `sha2` 0.10 pins, and the repins added as open done-conditions.
- **2026-10-06 (S44 follow-on).** Rulings C1 and C2. djinn repinned and Pelt
  desktop's orphan removed (`d8291708`); the session fixture repinned
  (`3032a629`). Gates, all green: `cargo check -p djinn`; `cargo check -p
  pelt-desktop` with default features and with `scripted`; the fixture's
  `cargo check` and `cargo check --locked`, run from outside the checkout;
  `cargo check --workspace --locked` at `1d87808a`, after the gopher-protocol
  0.2.0 merge landed on the same lock; `cargo test -p pelt-desktop` (63 + 0);
  `cargo test -p djinn` (116 passed, 9 ignored, `knot_residue` clean), its
  `published_site` tests exercising the repinned fingerprint. The fixture has
  no tests. djinn's test build failed twice at default parallelism with
  missing-artifact errors (`E0463`, rlibs "not found in this form", one rustc
  panic) while about twenty other cargo jobs ran on the machine, and built and
  passed at `-j 4`; the cause was not isolated, and no error named a type.
