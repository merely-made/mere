# Crypto Generation Unification

**Date:** 2026-08-10
**Status (2026-10-06):** reopened by ruling S44 of the
[stack seams plan](2026-10-04_stack_seams_plan.md). The unification landed
2026-08-10 (`1468e1b1`) for every first-party manifest then in mere; three
first-party manifests have since pinned `sha2 = "0.10"` against the ruled 0.11
row: djinn (`37706afb`), the distillery session fixture (`aa121f03`) and Pelt
desktop (on genet's 0.10 line). Open: repin all three to 0.11, a follow-on
task (below); Pelt's repin may wait on genet's own move to 0.11.
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

Two items are ours but out of this repo's reach:

- **`misfin` 0.0.4** pulls sha2 0.10 through its own published manifest. It
  lives in the smolweb workspace and is held in stewardship, so bumping it is a
  separate repo's commit and a republish.
- **`ed25519-dalek` is still in the graph at both 2.2.0 and 3.0.0**, via
  transitive pins (p2panda's fork line, misfin). The archived 2026-07-15
  iroh/p2panda bump plan is the record of how much work unifying that is.

`argon2` stays at 0.5.3: 0.6 is release-candidate only, and a password-hash
change is a stored-format change that wants its own migration note.
`signature` 2 rides with `ssh-key` 0.6 rather than moving alone.

## Reopened 2026-10-06: the three `sha2` 0.10 pins

Ruling S44 of the [stack seams plan](2026-10-04_stack_seams_plan.md): repin
all three, as a follow-on task outside the S14 documentation pass. Done when:

- [ ] `ports/djinn/Cargo.toml` takes the 0.11 row;
- [ ] `ports/distillery/probe/session-fixture/Cargo.toml` takes the 0.11 row;
- [ ] `ports/pelt/desktop/Cargo.toml` takes the 0.11 row, which may wait on
      genet's own move to 0.11.

## Progress

- **2026-10-06 (S14 pass).** Status and claims corrected against the tree at
  mere 535bca11, from the D2 record in
  support/doc-audit/d2/batch_43_s14_phase_b5.md: the status reopened under
  ruling S44, the Residue claim corrected with the three first-party
  `sha2` 0.10 pins, and the repins added as open done-conditions.
