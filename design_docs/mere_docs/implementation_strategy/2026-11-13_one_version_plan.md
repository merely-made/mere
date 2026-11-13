# One-version plan: every crate at one version, the latest, across the family

**Status:** 2026-11-13: plan. P1 has started outside Mere (base64 0.23 and rcgen 0.14 in Graphshell,
Armillary, Gemot and Errand). No fork exists yet; the fork list goes to Mark before any is created.

**For:** carrying out engine-split D144. Mark, 2026-11-13, asked "Why skips?" about the
`[[bans.skip]]` rows added for duplicates held by third-party crates at their latest release, then
ruled: "If we have to patch and fork the snot out of these crates to get them in line with a unified
stack, then we'll do it like we did wgpu. One version, latest version, no duplicates, unafraid to
patch or fork if needed to complete standards compliance or ensure the latest, unified version." And:
"Pure rust, data orientation. If no path exists with existing crates, we'll make our own. If only
inefficient paths exist, we'll make our own." On stale holders in other family repositories:
"Update freely."

This plan lives in Mere because the ruling is an engine-split decision and Mere is the crate most
repositories consume; the work is in every repository that carries a `deny.toml`.

## Where it stands

Measured on each repository's `origin/main:deny.toml`, 2026-11-13. The classification, holder by
holder, is `Code/testing/engine-split/skips_by_holder.md`; holder names there come from skip reason
text and existing locks, never from a fresh resolve, so each phase re-measures before it acts.

| Measure | Count |
|---|---|
| Repositories with a `deny.toml` | 11 |
| Skipped crate versions (distinct) | 263, over 208 crates |
| Skips per repository | Turnstone 177, Cleromancy 139, Isocosm 90, Esp 87, Knot 87, Woodshed 55, Genet 42, Mere 34, Eidetic 34, Murm 28, Dramatis 27 |
| Root holders (a crate that names an old version and is not itself skipped) | 405 |
| A: our own crate holds it | 83 holders; 21 skips need nothing else |
| B: a third-party holder has a newer release that drops it | 116 holders; 65 skips need at most this |
| C: a third-party holder still names it in its latest release | 206 holders; 177 skips need at least one |
| Crates in the cryptography domain among the 208 | 44 |
| Genet skips with no version and no reason | 62 |

A skip clears only when every one of its holders has moved, so the phases run in order and the
counts above are ceilings on what each phase clears alone.

## How a duplicate is removed

In this order, stopping at the first that works:

1. **Drop the holder.** A holder nothing needs (a dev-only tool, an unused feature, a second
   implementation of something the stack already has) leaves the graph.
2. **Update our own holder** (class A) in whichever repository names it.
3. **Raise the bound** on a third-party holder with a newer release (class B). Bounds are lower
   bounds, never `=x.y.z`; no lock is edited.
4. **Fork the holder** (class C): move its dependency to the latest version and adapt its calls.
5. **Write our own**, when no crate gives a path or only an inefficient one. Each case comes to
   Mark with the measurement.

A skip that remains names the step that will remove it. It is a stopgap, never an end state.

## Forks

- One fork repository per upstream repository, under `mark-ik`, when the fork "only contains
  patches and has no greater addition being considered" (Mark). A fork that grows past patches is
  a crate of our own and comes back to Mark. `mere-rdfc` is already that case and stays in Mere.
- The fork's `main` is the upstream release tag plus our patch commits, each commit one change.
  A crate a family repository depends on directly names the fork by git in its own manifest, which
  reaches every consumer with no extra row. A transitive holder is reached through
  `[patch.crates-io]` rows that `house` writes in every root workspace from one list; nobody
  hand-writes them.
- When upstream's own main has already moved, the fork is upstream's main with no patch, and it
  retires at the next upstream release. Retiring a fork is part of its done-condition.
- Cryptography-domain crates (Mark: "Version ports with vectors"): a patch may only move
  dependency versions and adapt the calls, never algorithm code, and only when the crate's own
  test vectors pass unchanged. Without vectors the skip stays until upstream moves.
- What we find goes in the upstream ledger first; Mark reads and posts.

## Phases

**P0. Instrument.** `house` reports, per repository, each skip with its holders from a fresh
resolve and its class, and writes the fork `[patch]` rows from one list of forks.
Done when: the report reproduces the table above within the drift of a day's releases, and
`house hooks --check` fails a repository whose `[patch]` rows differ from the list.

**P1. Our own holders (A).** By repository, upstream first: P2mere, Dramatis (castellan), Armillary,
Eidetic, Mere, Genet, Netrender, Rtc, Graphshell, Gemot, Errand, then the apps.
Done when: no skip in any repository has a family crate among its holders, on a fresh resolve.

**P2. Third-party bounds (B).** Raise each bound where the holder is named; adapt call sites.
Done when: no skip has a holder whose latest release drops the old version.

**P3. Forks (C), by leverage.** For each upstream repository: check steps 1 and 3 again, then fork.
First candidates, by skips they take part in: the oxigraph crates (they block nothing else and are
five crates in one repository), snow, iroh's netwatch and noq-proto, wasmtime, icu4x, winit, the
RustCrypto SSH crates. Each fork is listed to Mark before its repository is created.
Done when: every remaining skip names a crypto crate without vectors, with the reason saying so.

**P4. The rest.** Target-only duplicates (windows, objc2, jni, embedded), `syn` 1 against 2, and
Genet's 62 unversioned skips, which become versioned skips with reasons or go.
Done when: every `deny.toml` in the family has no skip, or only the P3 crypto exceptions.

**P5. Close.** Any carried patch left in a family repository moves to a fork or is argued to stay.
Mere carries none today (`support/patches` is empty at 198e1e967): D143's rudof patch was never
built, so engine-split P6's SHACL lane builds it as a `mark-ik` fork named directly by git, listed
to Mark before its repository is created; `mere-rdfc` is Mere's own crate (D144).
Done when: D144's stopgap wording can be removed from each `deny.toml` header.

## Decisions

- **one-version D1** (Mark, 2026-11-13). Third-party crypto holders at their latest release:
  "Version ports with vectors". Links: draws on engine-split D144 · governs P3.
- **one-version D2** (Mark, 2026-11-13). Where patched crates live: "2, in mark-ik. If it only
  contains patches and has no greater addition being considered" (option 2: a fork repository per
  crate, as with wgpu). Links: draws on engine-split D144 · governs Forks.
- **one-version D3** (reader, listed to Mark). The fork unit is the upstream repository, not the
  crate: wgpu is one fork of many crates, and oxigraph's five holders are one repository.
  Links: applies one-version D2.
- **one-version D4** (reader, listed to Mark). Dropping a holder is tried before forking it.
  Links: applies engine-split D144.

## Open questions

- ssh-key 0.6.7 takes part in 29 skips and its 0.7 line is still a release candidate. Fork 0.6.7
  forward, take the candidate, or wait: a fork for Mark when P3 reaches it (it is a crypto crate).
- Whether holders such as turso_core, tch, tokenizers, rav1e and mockito are needed at all: each
  comes to Mark with its users when P3 reaches it.

## Findings

- The oxigraph duplicates (rand 0.9 under oxrdf 0.3.5, quick-xml 0.37 under oxrdfxml 0.2.5) entered
  every consumer with P6 L1 and were skipped in Mere, Isocosm, Woodshed, Turnstone and Knot.
- rcgen was not stuck: 0.14.10 takes pem 4, which takes base64 0.23.

## Progress

- 2026-11-13: plan written; classification in `skips_by_holder.md`; D144 recorded (6839c3138).
- 2026-11-13: first reader's review: direct dependencies name a fork by git, `[patch]` rows only
  for transitive holders; P5 corrected, since Mere carries no patch today and the rudof patch is
  built as a fork by P6's SHACL lane. Landed on Mere main.
