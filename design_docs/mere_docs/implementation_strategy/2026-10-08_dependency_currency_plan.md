# Dependency Currency Plan

**Date:** 2026-10-08
**Status (2026-10-08):** surveyed, and set as the workspace upstream refresh (`mer3ly/docs/2026-10-08_workspace_upstream_refresh.md`)'s owner-specific stage for mere, genet and netrender (D8); rapier and parry moving on branch `rapier-036` (dynamics grammar plan F163 to F168), with mere's ten compatible updates folded in (D2). The breaking families wait on their order (§4).
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
- **Servo parsers** (genet): html5ever, markup5ever and xml5ever 0.39 to 0.40 (genet-static-dom, genet-scripted-dom, layout-dom-api, script-runtime-api, servo-xpath), and cssparser 0.37 to 0.38 (livery, cadency, genet-livery). *Corrected 2026-10-08:* the survey's first note said cssparser had to agree with a Stylo genet carries; genet has no Stylo dependency, so no such coupling holds.
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

**D5, the families the genet and netrender survey added (2026-10-08).** Options: Linebender text; Servo parsers; the small ones (sha2, tokio-tungstenite, windows-core, and genet's and netrender's compatible updates). Mark ticked **"Linebender text, Small ones"** and asked of the parsers: **"We kept the servo parsers?"** The coordinator's answer: yes, genet uses the crates.io html5ever, markup5ever and xml5ever 0.39 and cssparser 0.37, and has no Stylo coupling (§2, corrected). The parsers go back to Mark.

**D6, the order (2026-10-08).** Options: security first; upstream first (netrender, then genet, then mere's own groups, so each mere repin takes everything at once); accessibility first. Mark: **"Upstream first"**.

**D7, where this plan lands (2026-10-08).** Options: push the docs branch now; land it with rapier. Mark: **"Land with rapier"**.

**D8, how this plan fits the workspace upstream refresh (2026-10-08).** Question: Mark's workspace upstream refresh (mer3ly `docs/2026-10-08_workspace_upstream_refresh.md`, `f151b45`) already ran the compatible pass across 22 repositories (netrender's pollster 1.0, Turquet's sha2 0.11, Boa v0.22) and lists as deliberate boundaries the "registry API families" (accessibility, text shaping and glyph types, crypto and random, HTTP and storage, runtime) that "need owner-specific migrations and tests"; netrender's Vello lane (`netrender-notes/2026-10-08_vello_upstream_lane.md`) forbids bumping glyph-facing Skrifa across Classic's boundary independently. Options: this plan is the refresh's next stage, its owner-specific migrations for mere, genet and netrender, the text family going to the Vello lane; hand everything but rapier and parry to the refresh; keep both separate. Mark: **"Mine is the refresh's next stage (Recommended)"**. *Follows:* this plan carries the refresh's boundaries for mere, genet and netrender, citing them rather than repeating them. Linebender text (parley, parley_data, skrifa, read-fonts, harfrust) leaves this plan for the Vello lane, which owns glyph-facing types (D5's text tick is carried there, not here). The refresh's constraints hold here: provider before consumer, never a one-sided repin of the shared Netrender family, and toolchain ownership kept explicit (genet declares Rust 1.86).

**D9, coordinating with the refresh's owner (2026-10-08).** Mark: the refresh is **"The Codex agent"**'s. *Follows:* the coordinator writes the note and Mark relays it.

**D10, the Servo parsers (2026-10-08; from D5).** Options: include them in genet's stage, the WPT and DOM suites as their gate; leave them. Mark: **"Include them (Recommended)"**.

**D11, netrender's wgpu family (2026-10-08).** Question: `cargo update -p wgpu` left netrender on wgpu 30.0.1 over wgpu-core, wgpu-hal, wgpu-types and naga 30.0.0, a mix neither genet nor mere resolves; aligning moves five versions, downloads nothing, and passed netrender's suite (343 passed, 2 ignored, 66 executables) with the Classic Isocosm replay identical in all 18 fields. Options: align the family; keep wgpu alone at 30.0.1. Mark: **"Align the family (Recommended)"**.

**D12, landing netrender's lane (2026-10-08).** Question: sha2 0.11 (receipt hashes byte-identical) and the compatible updates reach neither consumer (sha2 is a dev-dependency; genet and mere already resolve every target version), so no repin follows. Options: push to netrender main once aligned; hold for genet's stage. Mark: **"Push after aligning (Recommended)"**.

**D13, when genet's stage opens (2026-10-08).** Question: with netrender landed (D12) and a lane slot free beside the mere lanes, genet's stage (accessibility, the small ones and compatible updates, the Servo parsers by D10) runs in genet's own repo and does not touch mere's lock. Options: open it now; wait for G4b1 to land, keeping the machine calm for its headed re-gate and rapier's receipts; spend the slot scoping G2c instead. Mark: **"Open genet's stage now (Recommended)"**. *Follows:* an Opus lane on its own genet worktree from `origin/main` (`cca45fc7a5c`), never genet's main checkout, which holds another session's uncommitted work. It carries D4, D8 and D10 and F138's gate rule, and stops at any fork or any WPT or DOM number that moves. Adopting the resulting genet into mere goes to Mark as its own question.

## 4. Order

Upstream first (D6): netrender (sha2 and its compatible updates; its text family is the Vello lane's, D8), then genet (accessibility, the small ones, the Servo parsers (D10); its text family is the Vello lane's), then mere's own groups (security-sensitive, accessibility, data and Meaning, other), one mere lane at a time, each taking the genet and netrender repins as they land. Constraints:
- every mere lane rewrites `Cargo.lock`, so mere lanes run one at a time;
- genet and netrender lanes can run beside a mere lane, but mere takes them only through a repin, netrender first (genet depends on it), then genet, then mere;
- at most three lanes at once (`~/.claude/CLAUDE.md`, Concurrency).

## 5. Done-conditions, per lane

- Each crate in the lane's family at its current release, or the release the family agrees on, with the lock diff logged.
- Gates under F138 (dynamics grammar plan): the crates whose inputs moved are compiled and tested; receipts that measure what moved are rerun, and any number that moves stops the lane for Mark (as F166).
- A family with a stored format (fjall) or a measured output (tokenizers, the text stack's shaping) comes to Mark as a fork before it lands.

## Progress

- 2026-10-08: surveyed (§1). rapier and parry moving on `rapier-036` with the compatible updates folded in (D2).
- 2026-10-08: netrender landed at `99d8d71ec` (D11, D12). genet's stage opened (D13).
