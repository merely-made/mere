# Stack modularity map: which concrete engines each product links, and the contract each should sit behind

**Date:** 2026-10-25
**Status:** findings with two landed checks, read at Mere `fb3e1224b`, Knot
`1766bf2`, Turnstone `871bfcc`, Murm `9ea81db34e6`, Eidetic `aa94fe2`; then two
landed cuts the same day, Eidetic `5afe6723` (stickleback `net`) and Knot
`46caf70` (knot-editor `peers`), which change the Knot desktop column and the two
cues that named them (marked below). Those two did force a repin round: Mere
`8bf1b5f98` took Eidetic, Knot took Eidetic, and Djinn opts in to `peers`.

**Origin.** Mark, 2026-10-25, while auditing Djinn's publication: "retinue
shouldn't have been a default any more than sennet, tucket, netfetcher, errand,
emissary, arti", and "knot should be usable as an offline application too".
The stack has sprawled: products link concrete transports and engines they do
not use, and licence or footprint decisions get made by accident. Runtime
loading would be ideal; compile-time optional linking is the floor, as
Turnstone already does for `weld`, `scry` and `servo` (`default = []`).

This note maps the links. It is written as cues: what is wrong and what would
improve the stack, not an order of work. The carriers themselves (stdio, local,
QUIC, WebRTC, app door) are surveyed in the
[transport agnosticism note](2026-10-15_transport_agnosticism_note.md); this
note is about what a build links, not how a session is carried.

## How it was measured

`cargo metadata` at a package's default features, walking normal (non-dev,
non-build) edges from the product. The first hop is the crate that introduces
the link, which is where a feature would go. Two scripts now keep the readings
repeatable:

- knot-editor `tools/check_profile.py`: `offline`, `default`, `library`, and
  positive controls (`--features retinue --expect retinue`, `library --features
  peers --expect mere-p2panda-net`).
- Turnstone `scripts/check_default_links.py`: the same walker over the turnstone
  package.

Metadata unifies features across a workspace, so a Mere reading can be a little
broader than `cargo build -p djinn`; the Djinn column below was cross-checked
with `cargo tree -p djinn`. It also unifies dev-dependencies' features: a
dev-dependency that turns `stickleback/net` on for its own tests showed iroh
under a library that never links it. `check_profile.py` therefore reads the live
graph with `cargo tree -p <package> -e normal`, which resolves features for that
package alone, and the Knot desktop default column below is that reading, not the
metadata one the first version of this note used.

## The map

| Link | Turnstone | Djinn | Knot desktop, default (`mere-retention`) | Knot desktop, `--no-default-features` | First hop | Contract it should sit behind |
|---|---|---|---|---|---|---|
| retinue (+ tulle) | **was linked, now opt-in** (`nomadnet`) | off (`knot-site-nomadnet`, `moot-site-nomadnet`) | off (`knot-desktop/retinue`) | off | `turnstone` | a page-fetch contract; `mere-transport`'s `reticulum` feature |
| outrider, radio-hand, sennet, tucket | absent | absent | absent | absent | reached only through `retinue`, `station-custody` (the Mere custody adapter used by Murm’s Signalman), `postilion`, `seneschal` | none needed; they stay out of product graphs |
| emissary, arti | absent | absent | absent | absent | `emissary` only in Murm's `murm-i2p`; no manifest in the family repos read names `arti` | none needed |
| mere-transport, iroh, iroh-blobs, snow, quinn | linked | linked | **absent** (was linked via knot-editor) | **absent** | `turnstone`, `distillery`, `graphshell-host`, `mere-mesh`, `knot-editor` (now `peers`), `djinn` | `Transport` trait with an in-memory implementation |
| mere-p2panda-net | linked | linked | **absent** (was linked via knot-editor) | absent | `stickleback` (now `net`), `mere-transport`, `mere-mesh`, `knot-editor` (now `peers`) | done for stickleback and knot-editor; `mere-transport` and `mere-mesh` remain |
| murm-webrtc (`mere-webrtc-carrier` before the 2026-10-25 rename) | linked | linked | absent | absent | `graphshell-host` (default `webrtc-join`) | `graphshell-host` carriers as features |
| netfetcher, reqwest, hyper | linked | linked | **absent** (was linked via knot-editor's `mere-fetch`) | **absent** | `mere-fetch` (unconditional; knot-editor now takes it with `peers`), `mere-lanes` (was `mere-document-lanes`) | netfetcher as the HTTP contract, behind `mere-fetch`'s own feature |
| errand and every smolweb protocol client | linked | linked | linked | linked | `nematic`, `mere-fetch`, `mere-lanes` (was `mere-document-lanes`), `turnstone` (`feed.rs`) | errand split into grammar and clients |
| gemini-protocol (server), spartan, rustls, rcgen | linked | linked | linked | linked | `knot-site`, `djinn` | a loopback `serve` feature in `knot-site` |
| script engines (boa, nova/vano, piccolo) | absent unless `piccolo` | absent | absent | absent | `script-engine-api` adapters | already behind `script-engine-api` |
| CEF, WebView2, Servo | absent unless `weld` / `scry` / `servo` | absent | absent | absent | inker adapters | already behind inker's reader routing |

What the table says in one line each:

- **retinue was the only Reticulum-licensed engine in the three product defaults
  read here**, and Turnstone's own manifest was the only unconditional edge to
  it. Djinn, Knot and `mere-transport` already kept it behind a feature.
- **Everything else the engines list names is already out of the defaults.** The
  remaining sprawl is the peer transport and the smolweb stack, not the mesh
  engines.
- **Knot desktop is an offline application at its default and at
  `--no-default-features`.** With `--no-default-features` it links no peer
  transport, no HTTP stack and no Reticulum, because `knot-editor` is the only
  door to them and it is optional (`mere-retention`). Since `knot-editor`'s
  `peers` feature, `mere-retention` is offline too: the library without `peers`
  links none of them. Both are checked (below).

## Landed

- **Workshop beside its host (2026-10-26).** `tabard-workshop` moved from
  `crates/cambium/` to `ports/tabard/workshop`, so the toolkit directory
  carries no document-stack link and `check_port_boundaries.py` refuses any new
  one. Embedding is unchanged: Pelt and the Tabard desktop take it by workspace
  path.
- **Turnstone `nomadnet`.** `retinue` is `optional`; the `nomadnet` feature is the
  only way it enters a build, with `default = []` beside `weld`, `scry`, `servo`,
  `piccolo` and `wasm`. Only `src/nomadnet/link.rs` names retinue. Address
  parsing and the Micron projection stay built, so a build without the feature
  refuses a `destinationhex:/path` page with a typed message instead of failing
  to resolve the address. `scripts/check_default_links.py` and
  `.github/workflows/default-links.yml` keep the default free of Reticulum
  engines, with a positive control.
- **Knot offline profile.** `tools/check_profile.py offline` and an `ubuntu` job
  in knot-editor's `ci.yml` fail if `knot-desktop --no-default-features` reaches
  mere-transport, p2panda-net, iroh, quinn, snow, stickleback, a Reticulum or
  mesh engine, netfetcher, reqwest or hyper. The Windows job also runs
  `cargo check -p knot-desktop --no-default-features`, so the profile is known
  to compile, not only to resolve.
- **Eidetic `stickleback` `net`** (`5afe6723`). `joined_space`, `synced_space`,
  `address_book` and `GroupKeyLane::join`/`publish` sit behind `net`, the only
  thing that links p2panda-net, and so iroh and QUIC. `default = ["net"]`, so
  every pinned consumer builds unchanged; `lane_id` and `JoinError` moved to an
  ungated `lane` module so `GroupKeyLaneError` keeps one shape. `cargo tree -p
  stickleback -e normal`: 306 crates with `net`, 109 without, none of them iroh,
  quinn, p2panda-net or reqwest.
- **Knot `peers`** (`46caf70`). `mark`, `publish_carrier`/`client`/`host`,
  `KnotSyncStore::join`, `resident`, the iroh blob store behind clip evidence
  (`BlobClipEvidenceStore`, `content_blob_store`, `blob_hash`) and the bins'
  `mere-fetch` sit behind `peers`, `default = []`, which turns on
  `stickleback/net`. `knot-editor` no longer names p2panda-net directly, and
  without `peers` it links no iroh, quinn, p2panda-net, p2panda-sync or reqwest
  (`p2panda-core`, `-store` and `-encryption` remain: operation format, local
  store, group crypto). `check_profile.py library` and `default` hold that, with a
  positive control. Djinn opts in by name (`ports/djinn/Cargo.toml`); Turnstone
  does the same in its own repository.

## Cues: what is wrong, and what would improve the stack

### `mere-transport` is one crate where the contract and its one implementation are fused

`lib.rs` exports `Transport`, `memory::MemoryTransport`, `Alpn`, `PeerID` and the
optional `noise`, `notochord` and `reticulum` layers, but `blobs`,
`p2panda_transport` and `p2panda_host` are unconditional, and `iroh`,
`iroh-blobs`, `iroh-tickets`, `p2panda-net` and `stickleback` are unconditional
dependencies. So linking the trait links QUIC. A `p2panda` (or `iroh`) feature
over `P2pandaTransport`, `BlobStore` and the peer-route tickets would leave the
contract plus the in-memory transport as the base, which is what the doctrine's
"generic over implementation" line already promises
(`T: Transport`, not `Box<dyn Transport>`). It would also let tests and the
offline products use `MemoryTransport` without iroh. `BlobStore` is the awkward
part: `knot-editor`'s clip evidence and `endpoint.content_blob_store()` take it,
while `FileClipEvidenceStore` shows a file-backed store already satisfies the
same trait-shaped need, so a small blob-store contract (or a `blobs` feature)
would let Knot keep local clip evidence offline.

### `stickleback` linked p2panda-net for four modules (landed: `net`)

`joined_space`, `synced_space`, `address_book` and `group_lane`'s join and
publish used p2panda-net (plus the `Endpoint`/`Gossip` re-export in `lib.rs`);
the rest of its 11k lines (causal index, processor, store, keyring, group crypto,
epoch retention, drop carriage) is local. They are now behind `net`, on by
default. What is left is the consumers: `mere-mesh` and Murm's `mere-transport`
take stickleback with its defaults and so still link the net half. A consumer
that never joins a peer space can now say `default-features = false`.

### `knot-editor` had no seam between its authority and its peers (landed: `peers`)

The library's network code was concentrated, which made it cuttable, and the
cut is the `peers` feature:

| Module | Links | Now |
|---|---|---|
| `mark.rs` | quinn, rustls (a QUIC read host) | `peers` |
| `publish_carrier.rs`, `publish_client.rs`, `publish_host.rs` | `transport` with `noise`, notochord | `peers` |
| `sync.rs` | `join()` takes p2panda-net `Endpoint`/`Gossip`; one test uses `P2pandaTransport` | `join` and that test under `peers` |
| `resident.rs` | `KnotSyncHost` over `transport::p2panda_transport` | `peers` |
| `clip_evidence.rs`, `endpoint.rs` | `transport::BlobStore` | `clip_evidence/blob.rs` and the retention accessors under `peers`; `FileClipEvidenceStore` stays |
| `bin/knot_endpoint.rs`, `bin/knot_sync_host.rs`, `examples/*peer*.rs` | `mere-fetch`, the blob store, a peer transport | `required-features = ["peers"]` |

`default = []` like `embed-bert` and `mere-overlay`. Djinn uses the sync host and
the publish lane and opts in by name; Turnstone needs publishing and
`BlobClipEvidenceStore` and does the same; Knot desktop needs neither, so
`mere-retention` is an offline retention. `graphshell-endpoint`'s `door` and
`stdio` stay unconditional: the `door` is a local contract (custody over the
application door), not an Internet link, though it is the thing an offline Knot
with no resident would have to replace with a local key provider.

### `mere-fetch`, `mere-mesh`, `graphshell-host` link their engines unconditionally

- `mere-fetch` takes `netfetcher` as a plain dependency, so anything that wants
  `FetchCommand`/`FetchUpdate` vocabulary links an HTTP client. The actor, the
  smolweb lane and the cookie jar are features; the base is not.
- `mere-mesh` takes `stickleback` and `p2panda-net` unconditionally, with
  `transport` optional (`admitted-ingress` is its only feature). The policy and
  condition vocabulary (`DeviceConditions`, `ConditionSource`) does not need
  either.
- `graphshell-host` defaults to `webrtc-join`, which brings the sans-I/O WebRTC
  carrier crate, and its `native` feature adds `mere-transport` on top of it. A
  Turnstone that serves places over iroh still links the WebRTC carrier crate;
  only the live session (`webrtc-session`, str0m) is separate.

### errand fuses a grammar with its protocol clients

`errand` lists `gemini-protocol`, `gopher-protocol`, `finger-protocol`,
`guppy-protocol`, `spartan-protocol`, `nex-protocol`, `scroll-protocol` and
`misfin` as unconditional dependencies, with `tokio`, `rustls` and `tokio-rustls`
beside them and a single `serve` feature on top (`default = []`).
`nematic` and Turnstone's `feed.rs` use it for the RSS/Atom XML grammar. So the
offline Knot, which only renders, still links every smolweb client, and any
product that parses a feed links them too. Splitting the grammar from the clients
(a `clients` feature, or a grammar crate) would make "errand is not a default"
literally true for renderers.

### `knot-site` links a loopback server whether or not it serves

`gemini-protocol` (with `server`), `rustls`, `rcgen`, `spartan-protocol` and
`scroll-protocol` are unconditional, because `LocalServer` is part of Knot's
publish preview. The desktop does use it, so this is a product decision: the
offline profile today means *no peer or Internet engine*, and the loopback
server is carried. A `serve` feature would make "offline" mean "no listener"
if Mark wants that stricter reading.

### Djinn's `personal-sync` gates code but not the dependencies it implies

`personal-sync = []` is empty, yet 91 `cfg(feature = "personal-sync")` sites in
`ports/djinn/src` hang off it, and `cxf` and `inference-stages` name it. The
sync stack's dependencies (`p2panda-core`, `p2panda-store`, `stickleback`,
`transport`, `knot-editor`) are unconditional, so a build without
`personal-sync` compiles less of Djinn and still links all of the stack. Making
the feature own those dependencies (`dep:` entries, with `knot-editor/peers`)
would make the flag mean what it says. Djinn is a legitimate `peers` consumer,
because it uses Knot's sync host, and now opts in by name, unconditionally;
`personal-sync` could own that dependency.

### Turnstone's places lane is the product, but it is still cuttable

`transport`, p2panda, `gemot`/`commons`, `stickleback` and `muniment` are used in
`src/place/{worker,lanes,invite,projection_host}.rs`, `place.rs`,
`publish_service.rs`, `share_reader_service.rs` and `knot_authoring.rs`;
`errand` in `feed.rs`; `netfetcher` in `cookie_custody.rs`. Turnstone already
has a runtime `TURNSTONE_OFFLINE=1` receiver (`src/offline.rs`) that answers
fetches with a typed refusal, so a build with no `places` feature could do the
same for places at compile time and drop the shared-place stack from a
browser-only build. That is a larger cut than `nomadnet` because the place
worker owns identity and persistence, not one fetch.

### The toolkit directory carries one document-stack link, through `tabard-workshop`

`tabard` is a leaf over `tinct`, so `mere-lanes` naming its reading
preferences, typography roles and smolweb palette is a library link downward,
not a cycle. The upward link is `tabard-workshop`: its reader specimen extracts
and lowers a real article through `mere-lanes`, `document-canvas` and `inker`,
and its stylesheet specimen links `genet-livery`, `genet-scripted-dom` and
`netrender`. `cambium-genet-web-host` reaches the workshop only as a wasm
dev-dependency. So any grouping that files the workshop with the toolkit makes
the toolkit depend on the document stack while the document stack depends on
the toolkit's token library. The 2026-10-08 theme-modes decision put the
workshop under `crates/cambium/` so applications other than the Tabard port
could embed it; a crate is embeddable from any directory, so the location is
the open part: beside its desktop host under `ports/tabard/`, or left where it
is with its specimens behind features. `scripts/check_port_boundaries.py`
refuses a new unconditional toolkit-to-document-stack link.

Ruled and landed 2026-10-26: Mark chose "beside its host"; the package is
`ports/tabard/workshop`, embedded by workspace path as before, and no crate
under `crates/cambium/` links the document stack. The web host's
`shared_recovery` wasm example still mounts the workshop as a dev-dependency,
which the port boundary check now prints as a carried dev-only link rather
than refusing; whether examples may mount a product surface is Mark's to
confirm.

### Which rung each cut belongs on

Doctrine Stack 5 puts "cold and narrow" behind a component when it wants to be
pluggable, and "hot or wide" native. NomadNet page and form requests are called
once per navigation and cross one narrow byte boundary, so they are the kind of
thing a component (or an external launcher like Knot's
`tools/knot-nomadnet-go` page server) could load at runtime, with the `nomadnet` feature as
the compile-time floor. The peer transport is wide (identity, admission,
persistence all lean on it) and stays a feature. The smolweb clients are cold
and narrow and are the best next candidates for a runtime rung after errand is
split.

## Guard rails now in place

- A default Turnstone cannot link a Reticulum-licensed engine unnoticed.
- The offline Knot profile cannot gain a peer transport, an HTTP stack or a
  Reticulum engine unnoticed, and neither can the default desktop or the
  `knot-editor` library without `peers`.
- Neither check covers the *carried* links above; both print them, so a reader
  can see the gap close.
- No crate under `crates/cambium/` can gain an unconditional link to the
  document stack (`inker`, `document-canvas`, `mere-lanes`, `nematic`,
  `fleece`, `mere-pageload`, `mere-fetch`, `knot-editor-host`) unnoticed; none
  does since `tabard-workshop` moved to `ports/tabard/workshop` (2026-10-26).
  A dev-only link from a crate to a port is printed, never refused.
