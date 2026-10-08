# Dependency Currency Plan

**Date:** 2026-10-08
**Status (2026-10-08):** surveyed; rapier and parry moving on branch `rapier-036` (dynamics grammar plan F163 to F168), with mere's ten compatible updates folded in (D2). The breaking families wait on their order (§4).
**Scope:** bring mere, genet and netrender's crates.io dependencies to current releases; the rest of the workspace follows through repins (D1). Size and footprint are the [dependency footprint brief](../../2026-07-04_dependency_footprint_brief.md)'s; this plan is about currency.

## 1. How it was measured

On 2026-10-08, each repo's declared dependencies were read offline (`cargo metadata --no-deps`), and each registry crate's newest stable, unyanked release was read from the crates.io index (metadata only, nothing downloaded). Locked versions come from mere's `origin/main` lock, which also resolves genet's dependencies, since genet ignores its own lock. A crate is *current*, has a *compatible update* (same semver-compatible class), or has a *breaking newer* release. Git dependencies (genet, retinue, netrender, knot-editor and wgpu-weld at pinned revisions) are repins, not this survey.

| Repo | Registry deps | Current | Compatible update | Breaking newer | Not locked |
|---|---|---|---|---|---|
| mere | 164 | 125 | 10 | 20 | 9 (patched p2panda, burn-remote, vello) |
| genet | 126 | 76 | 13 | 15 | 22 (path and patched) |
| netrender | 15 | 8 | 4 | 3 | 0 |

## 2. The breaking releases, by family

A family moves together because its crates pin each other.

- **rapier and parry.** rapier2d and rapier3d 0.33 to 0.36, parry 0.28 to 0.31.1 (seiche, conatus). Moving now: dynamics grammar plan F163 to F168.
- **Security-sensitive** (mere and netrender):
  - argon2 0.5.3 to 0.6.0 (personae);
  - minisign 0.7.9 to 0.10.0 and minisign-verify 0.2.5 to 0.3.0 (luggage);
  - wasmtime and wasmtime-wasi 45 to 49 (app-host, document-host);
  - zip 0.6.6 and 2.4.2 to 8.6.0 (muniment);
  - sha2 0.10 to 0.11 (netrender's `paint_list_render`).
- **Linebender text** (netrender and genet): parley and parley_data 0.10 to 0.11, skrifa 0.42 and 0.44 to 0.48, read-fonts 0.39 and 0.41 to 0.45, harfrust 0.8 to 0.14.
- **Accessibility** (genet and mere): accesskit 0.24 to 0.25, accesskit_consumer (three versions locked) to 0.39, accesskit_winit 0.32 to 0.34, accesskit_macos 0.26 to 0.27, accesskit_unix 0.21 to 0.24, accesskit_windows 0.32 to 0.35.
- **Servo parsers** (genet): html5ever, markup5ever and xml5ever 0.39 to 0.40, and cssparser 0.37 to 0.38, which has to agree with the Stylo genet carries.
- **Data and Meaning** (mere):
  - fjall 2.11 to 3.1 (eidetic; its on-disk format changes);
  - jsonschema 0.18 to 0.58 (eidetic);
  - infer 0.19 to 0.22 (mere-kernel, mere-registry);
  - tokenizers 0.20 to 0.23 (esp; can shift the pinned e5-base-v2 embeddings and P7's receipt).
- **Other:** str0m 0.23 to 0.24, x11rb 0.13 to 0.14, dirs 5 and 6 to 7, pollster 0.4 to 1.0, tokio-tungstenite 0.29 to 0.30, windows-core 0.62 to 0.100.
- **Pinned on purpose, not in scope:** taffy at `0.11.0-experimental-cache-fix.3` (genet's layout work).

## 3. Rulings

**D1, which repos (2026-10-08).** Options: survey every repo under `Code/repos`; mere only. Mark: **"genet and netrender too, but that's all. the rest, the changes will propagate from those two."**

**D2, mere's compatible updates (2026-10-08).** Options: a small lane after rapier lands; fold them into the rapier lane; leave them. Mark: **"Fold into the rapier lane"**. *Follows:* encoding_rs, inventory, iroh-blobs, libc, scrying, thiserror, tokio, tokio-rustls, uuid and zeroize move in `rapier-036`'s lock change.

**D3, which breaking groups (2026-10-08).** Asked of mere's four groups (security-sensitive, accessibility, data and Meaning, other), Mark ticked all four: **"Security-sensitive, Accessibility, Data and Meaning, Other"**. *Reading, not ruled:* the families the genet and netrender survey added (Linebender text, Servo parsers, sha2, tokio-tungstenite, windows-core) go to Mark for their place in the order (§4).

**D4, downloads (2026-10-08).** Options: crates.io for this program only; ask per lane; the ten compatible updates only. Mark: **"crates.io, this program only (Recommended)"**. *Follows:* this program's lanes fetch from crates.io only, never git or other registries, and log every new crate and version in their lock diff.

## 4. Order

Open; goes to Mark. Constraints:
- every mere lane rewrites `Cargo.lock`, so mere lanes run one at a time;
- genet and netrender lanes can run beside a mere lane, but mere takes them only through a repin, netrender first (genet depends on it), then genet, then mere;
- at most three lanes at once (`~/.claude/CLAUDE.md`, Concurrency).

## 5. Done-conditions, per lane

- Each crate in the lane's family at its current release, or the release the family agrees on, with the lock diff logged.
- Gates under F138 (dynamics grammar plan): the crates whose inputs moved are compiled and tested; receipts that measure what moved are rerun, and any number that moves stops the lane for Mark (as F166).
- A family with a stored format (fjall) or a measured output (tokenizers, the text stack's shaping) comes to Mark as a fork before it lands.

## Progress

- 2026-10-08: surveyed (§1). rapier and parry moving on `rapier-036` with the compatible updates folded in (D2).
