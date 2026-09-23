# Ranged Fetch Plan

**Date:** 2026-09-20  
**Status:** **plan accepted 2026-09-20 with D1 to D7 decided; lane F merged and pushed 2026-09-20; lane R pushed as Woodshed `bf5923d`; T1 and T3 pushed/built in Turnstone 2026-09-22; M1 built in Mere; T2 waits on the next lattice round.**  
**Authority:** the implementation home for rulings 2 to 4 of lane R3 in the
[family composition thesis](../../2026-08-12_family_composition_thesis_brief.md#r3-resource-resolution-opened-2026-09-20).
The research, probes and rulings stay there; this plan owns the work.  
**Companions:** the [netfetcher plan](../../archive_docs/2026-06-09_completed_plans/2026-05-25_netfetcher_plan.md)
(Mere owns networking, hosts run the fetcher, Genet consumes bytes), the
[session store plan](2026-06-23_native_session_store_plan.md) (one session
substrate per persona), the [engine profile boundary plan](2026-05-14_engine_profile_boundary_plan.md)
(a session's profile binding is persona, session or graph scoped), the
[net-media plan](2026-05-26_net_media_plan.md) (the likely second consumer), and
the [R3 receipt](../testing/receipts/2026-09-20_resource_resolution_probes/RECEIPT.md).
Cross-repo: `woodshed/design_docs/2026-09-01_listening_annotation_port_plan.md`
and `turnstone/design_docs/2026-09-14_redshank_episode_surface_plan.md` gate 1.

## What was ruled (Mark, 2026-09-20)

- The ranged contract is a trait in Mere's fetch crate (`crates/system/fetch`),
  with Redshank as its first consumer.
- It sits there behind features: the actor, persona cookie persistence and flip
  export become default-on features, so today's consumers (Turnstone,
  knot-editor, `crawl`) see no change and a small host turns them off.
- Standalone Redshank moves onto netfetcher; ureq leaves its workspace.
- Each host process fetches HTTP itself, over the one per-persona session store.
- Ranged requests are sent no-store (R3-A's pinned negative).

## What the probes fixed about the shape

One blocking call from a plain thread over a shared tokio runtime: a byte range
in, and bytes, total length, validator, final URL and content type out. A
changed representation must be visible to the caller. Whole bodies stream, so a
download never needs the body held whole. Nothing new is needed from netfetcher.

## Lanes and done-conditions

**F. Mere, the fetch crate.**
- F1, features. Default build unchanged. With default features off, the crate's
  dependency tree names none of armillary, eidetic, pandect, inker or errand,
  checked with `cargo tree` (features per D3), and the measured crate count standalone Redshank
  gains is recorded (the lockfile bound was 71 against 393). `crawl`,
  knot-editor and Turnstone build untouched.
- F2, the trait and its types, per D1 and D2, with no dependency beyond
  netfetcher, tokio, url and bytes.
- F3, the netfetcher implementation: a blocking host over one shared runtime and
  an injected store set (cookie jar, HTTP cache, HSTS, Alt-Svc), sending ranges
  no-store. Its tests are R3-A's six assertions, moved in from the receipt,
  pinned negative included.
- F4, scope, per D4: the store set is chosen by the caller's scope, and the
  fetch actor's per-fetch context is built from the same set, so a page login
  and a ranged read share cookies (test: R3-A's persona assertion through the
  actor and the host together).

**R. Woodshed, Redshank.**
- R1: `redshank-playback` depends on the fetch crate with default features off,
  not on ureq. The runtime is started with a fetch handle. The range source keeps
  its window, validator, budget and stats over the trait. Its existing tests run
  against the real implementation and the in-test server, plus a fake.
- R2: `redshank-cache`'s download and the desktop's feed fetch use the same
  handle (per D1). `ureq` is absent from `ports/redshank/Cargo.lock`.
- R3: the `remote-buffering` and `longnames` scenario receipts return
  `RESULT ok`; workspace tests and strict Clippy pass; the measured build-time
  and binary-size change is recorded beside R3-C's prediction.

**T. Turnstone.**
- T1: the Redshank host is handed a fetch handle built from Turnstone's store
  set. Gate 1 of the episode surface plan closes. `ureq` remains in Turnstone's
  lock only under the `download-cef` build helper.
- T2, per D5: the fetch actor and the page-image fetcher share that store set,
  which needs `mere-document-lanes`' `RemoteFetcher` to accept an injected host
  (a Mere slice, **M1**). Test: one cache and one jar observed from both paths.
- T3: persona cookie persistence gets a caller again (`load_cookies` at session
  adoption, `persist_cookies` on the event-loop drain), closing the session
  store plan's open persist-trigger gap for Turnstone.

**I. Integration.** Pins move mere, then knot-editor, then woodshed, then
turnstone, each consumer on one revision of each upstream, proven in a worktree
with no local cargo config. Pushes need Mark's sign-off.

## Decisions (Mark, 2026-09-20, put one at a time)

- **D1. One handle, three calls:** a byte range, a capped whole body (a feed),
  and a whole body streamed into a sink (an episode download).
- **D2. Names.** Trait `Fetch` (so `fetch::Fetch`), calls `read_range`,
  `read_all`, `read_into`, types `Range`, `RangeReply`, `FetchError`.
- **D3. Four default-on features:** `actor` (the page actor, armillary),
  `smolweb` (errand's lanes and Gemini identities, apart from the actor so a
  host can run http and https alone), `persona-cookies` (eidetic, pandect),
  `flip` (inker's cookie export).
- **D4. A caller-supplied store set.** The handle is built from a set the host
  owns (jar, cache, HSTS, Alt-Svc, and since 2026-09-22 the transport, so the
  privacy lanes and a test double reach a handle). The fetch crate does not decide what a scope
  is; Turnstone keys its sets by persona now and by the full profile binding
  later, and standalone Redshank passes one.
- **D5. Lanes T2 and T3 stay in this plan,** after Redshank's reader lands.
- **D6. `HostBlob` stays rejected** until Redshank's resident client work, which
  gets its own plan. Redshank's on-disk offline cache is untouched.
- **D7. Lane F starts now in an isolated Mere worktree,** touching only the
  fetch crate. Nothing is merged or pinned until the other session's lockfile
  and toolchain change lands; a rebase onto it may be needed.

## Not in this plan

Held episodes through the resident (R3 ruling 5). The wasm host: netfetcher is
native-only, `redshank-web` depends on neither playback nor a fetcher today, and
a browser host would use the browser's fetch. Any change to netfetcher's cache
so that it can answer a range from a stored body. The page-side media element.

## Hazards recorded 2026-09-20

- Turnstone's tree held another session's uncommitted work touching
  `Cargo.lock`, `.gitignore` and `rust-toolchain.toml`, so the repin that would
  carry Woodshed `fdc5980`'s connection fix into Turnstone was not made.
- The fetch crate is published (`mere-fetch` 0.0.1). Making existing behaviour a
  default-on feature is additive; removing a default later would not be.

## Progress

- **2026-09-20:** plan written from the R3 rulings; D1 to D7 decided the same day.
- **2026-09-20, lane F on `slice/ranged-fetch-20260920`, not merged:**
  - F2 and F3 landed (`af6dbc7a`): `Fetch`, `NetFetch`, `Stores` and the reply
    types in `crates/system/fetch/src/handle.rs`, with seven tests against a real
    local server moved in from the R3-A probe, the pinned cache negative
    included. One addition to D2's names: `Facts` (final URL, content type,
    declared length, validators) rides on every reply, and `Body` is what
    `read_all` returns.
  - F1 landed: four default-on features. Public shapes do not vary with features;
    only behaviour does (without `smolweb`, a small-web address fails as an
    ordinary fetch and the actor refuses a submission in words). Durable cookies
    and the flip export moved to `cookies_persist.rs` and `cookies_flip.rs`.
    Verified: each feature builds alone, and `actor` with `smolweb`; default
    tests 17 pass, defaults-off tests 7 pass; Clippy is clean for the crate both
    ways; `mere-crawl` checks untouched. With defaults off the dependency tree
    names none of armillary, eidetic, pandect, inker or errand, and standalone
    Redshank's lock would gain **48 crates** (the bound was 71, and 393 as the
    crate stood). Twenty-four of the 48 are netfetcher's HTTP/3 and WebSocket
    lanes, which Mere's workspace enables; taking netfetcher without them is a
    further cut, not made, that would bring a small host to about 24.
  - F4 landed: `spawn_fetcher_with(wake, stores)` gives the page actor a host's
    `Stores`, and `spawn_fetcher` passes `session_stores()`, the shared session
    jar with no cache, HSTS memory or Alt-Svc memory, which is what every fetch
    did before, so default behaviour is unchanged. Test: a login made by a page
    fetch through the actor is carried by a ranged read through a handle over the
    same stores, with the control that the process session saw nothing.
  - Rebased onto main after the lattice change landed (`68f78873`, `b45ecbbb`);
    main never touched the fetch crate. On Rust 1.98.1 against the tracked
    portable lock, `cargo test --locked -p mere-fetch` passes 18, so the lock
    needs no change. Clippy is clean across all seven feature sets, 8 tests pass
    with only `actor`, 7 with defaults off, and `mere-crawl` checks. The three
    unused-patch warnings (boa twice, iroh-mdns) are the inherited baseline the
    lattice sync pass plan owns under its P4.
  - **Lane F merged to main by fast-forward and pushed with Mark's sign-off**
    (`c0463e98`, `ba4f4951`, `e35898d1`). Lanes R, T and I pin from here; their
    pin moves belong with the [lattice sync pass](2026-09-16_lattice_sync_pass_plan.md),
    which owns cross-repo revisions.
- **2026-09-22:** `Stores` gains a `transport` slot (`None` is netfetcher's
  direct HTTP). Found in the R3 assessment: without it the reachability plan's
  I2P and Arti lanes, which plug into netfetcher's transport seam, could not
  reach a handle. Test: a supplied wire is what the handle sends through, with a
  `NoTransport` control. 19 tests by default, 8 with defaults off, Clippy clean.
- **2026-09-22, lane R, Woodshed `a1ebf6d` and `bf5923d` (local, awaiting
  sign-off to push):** R1 and R2 landed. `redshank-playback`, `redshank-cache`
  and the desktop take the fetch crate with defaults off at the Mere revision
  the port pins (`0e031fa5`, so one Mere revision with Turnstone; the transport
  slot at `50699731` arrives with the next lattice move). ureq is gone from the
  lock. `PlaybackRuntime::start_with` takes the host's handle and `start()`
  keeps its signature over one of its own, so Turnstone builds unchanged until
  T1. The cache download streams into a hashing, budgeting sink; the feed is
  `read_all` with its accept header and 4 MiB limit.
  - R3, measured: the lock gains 48 crates and loses 3 (R3-C predicted 48). 161
    tests pass, 7 ignored (the two Content-Range parser tests moved to Mere
    with the parser); strict Clippy clean. Release `redshank-desktop.exe` is
    32.1 MB after; the before size was not measured, and the release build with
    warm dependencies took 355 s, which is dominated by Cambium and Genet, not
    by the fetch crates.
  - The receipts found a defect the lane did not cause: on this machine the
    default output device now reports more than two channels, and the output
    runtime connected one edge per device channel into Firewheel's stereo graph
    output, so every load ended unavailable. The untouched pushed revision fails
    the same way. Fixed in `a1ebf6d` by sizing the volume node and its edges to
    the graph. The same commit makes `listen_playing.scn` and `longnames.scn`
    assert PLAYING, because both had reported RESULT ok through the failure;
    the whole scenario set passes with the assertions, longnames on its
    range-served item, so the ranged path is played rather than drawn.
- **2026-09-22, lane R pushed** (Woodshed `a1ebf6d`, `bf5923d`).
- **2026-09-22, lane T1, Turnstone (local, awaiting sign-off to push):** the
  cascade session had pinned Turnstone at Woodshed `f5a6649`; a separate
  repin takes `bf5923d`, with the portable lock regenerated by bare Cargo and
  `scripts/cargo_mode.py verify` passing on 1,318 packages, one Mere, Genet,
  knot-editor and Woodshed revision each. `Shell::new` builds the page actor
  with `spawn_fetcher_with` over `fetch::session_stores()` and a blocking
  `NetFetch` over the same stores, and hands that to `RedshankHost::open`; the
  host keeps the handle and `reopen` carries it into the next session, while a
  host that never had one (a fixture) reopens its model and starts no runtime.
  Test: `the_shells_fetch_handle_survives_a_session_reopen`. Gate 1 of
  Turnstone's episode surface plan is closed. The process session stores are
  used deliberately, not `Stores::in_memory()`: the page actor keeps exactly its
  previous behaviour (session jar, no HTTP cache), and a persona-keyed set with
  a cache is T2's decision, not T1's. Turnstone is not rustfmt-clean, so its
  formatter was not run.
- **2026-09-22, T1 pushed** (Turnstone `2c451d2`, `77b7ece`).
- **2026-09-22, M1 built (Mere `b3d52c74`, local):** `fetch::Resources`, the
  handle as Genet's `ResourceFetcher`, with the engine's default cap, final URL
  and content type, and a fallback for the schemes it does not serve, so a
  host chains it ahead of the small-web fetcher. Tested served, refused,
  redirected, capped, chained, and cookie-sharing. 20 tests, 9 with defaults
  off, Clippy clean both ways.
- **2026-09-22, T2 waits:** Turnstone taking M1 is a Mere pin move, which by
  the lattice pass's stop rule 4 means knot-editor realigns first. The cascade
  session reports the Genet roadmap session is about to bump Mere to Genet
  `b3ef95e9729`, which forces the same three-repo round; one round at a Mere
  revision carrying both changes is the plan. Rulings for T2 (Mark,
  2026-09-22): one shared set with an in-memory HTTP cache, so pages, images
  and episodes obey one cache policy.
- **2026-09-22, T3 built (Turnstone, local):** `cookie_custody` opens a fjall
  store under `<data_root>/cookies`, loads it into the session jar at start and
  flushes after each drain. Rulings (Mark, 2026-09-22): the built-in default
  persona for now, since Turnstone has no persona id of the store's kind, and a
  profile-level store rather than the session bin. Test: a cookie set in one
  run is there in the next. The session store plan's persist-trigger gap is
  closed for Turnstone.
