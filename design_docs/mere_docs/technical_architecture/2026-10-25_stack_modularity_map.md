# Stack modularity map: which concrete engines each product links, and the contract each should sit behind

**Date:** 2026-10-25
**Status:** findings with two landed checks. Read at Mere `fb3e1224b`, Knot
`1766bf2`, Turnstone `871bfcc`, Murm `9ea81db34e6`, Eidetic `aa94fe2`. Nothing in
Mere, Murm or Eidetic changed, so nothing here forces a repin.

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

- knot-editor `tools/check_profile.py`: `offline`, `default`, and a positive
  control (`--features retinue --expect retinue`).
- Turnstone `scripts/check_default_links.py`: the same walker over the turnstone
  package.

Metadata unifies features across a workspace, so a Mere reading can be a little
broader than `cargo build -p djinn`; the Djinn column below was cross-checked
with `cargo tree -p djinn`.

## The map

| Link | Turnstone | Djinn | Knot desktop, default (`mere-retention`) | Knot desktop, `--no-default-features` | First hop | Contract it should sit behind |
|---|---|---|---|---|---|---|
| retinue (+ tulle) | **was linked, now opt-in** (`nomadnet`) | off (`knot-site-nomadnet`, `moot-site-nomadnet`) | off (`knot-desktop/retinue`) | off | `turnstone` | a page-fetch contract; `mere-transport`'s `reticulum` feature |
| outrider, radio-hand, sennet, tucket | absent | absent | absent | absent | reached only through `retinue`, `mere-signalman`, `postilion`, `seneschal` | none needed; they stay out of product graphs |
| emissary, arti | absent | absent | absent | absent | `emissary` only in Murm's `murm-i2p`; no manifest in the family repos read names `arti` | none needed |
| mere-transport, iroh, iroh-blobs, snow, quinn | linked | linked | linked | **absent** | `turnstone`, `distillery`, `graphshell-host`, `mere-mesh`, `knot-editor`, `djinn` | `Transport` trait with an in-memory implementation |
| mere-p2panda-net | linked | linked | linked | absent | `stickleback`, `mere-transport`, `mere-mesh`, `knot-editor` | `stickleback` without its net half |
| murm-webrtc (`mere-webrtc-carrier` before the 2026-10-25 rename; Turnstone and Knot still lock the old name) | linked | linked | linked | absent | `graphshell-host` (default `webrtc-join`) | `graphshell-host` carriers as features |
| netfetcher, reqwest, hyper | linked | linked | linked | **absent** | `mere-fetch` (unconditional), `mere-document-lanes` | netfetcher as the HTTP contract, behind `mere-fetch`'s own feature |
| errand and every smolweb protocol client | linked | linked | linked | linked | `nematic`, `mere-fetch`, `mere-document-lanes`, `turnstone` (`feed.rs`) | errand split into grammar and clients |
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
- **Knot desktop is already an offline application, one flag away.** With
  `--no-default-features` it links no peer transport, no HTTP stack and no
  Reticulum, because `knot-editor` is the only door to them and it is optional
  (`mere-retention`). That is a property of the product feature set, so it is
  now checked (below). It is not yet true of `knot-editor` the library.

## Landed with this note

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

### `stickleback` links p2panda-net for four modules

`joined_space`, `synced_space`, `address_book` and `group_lane` use p2panda-net
(plus the `Endpoint`/`Gossip` re-export in `lib.rs`); the rest of its 11k lines
(causal index, processor, store, keyring, group crypto, epoch retention, drop
carriage) is local. `knot-editor` uses `DataKeyring`, `GroupCiphertext` and
`CausalIndex` outside its network modules. A `net` feature on those four modules
is what stands between the Knot library and an iroh-free tree. It is the cheapest
cut with the largest effect on this map, and it lives in Eidetic, so landing it
is a repin round across Knot, Mere, Turnstone and Redshank; adding it with
`default = ["net"]` first would keep every pinned consumer building.

### `knot-editor` has no seam between its authority and its peers

The library's network code is concentrated, which makes it cuttable:

| Module | Links |
|---|---|
| `mark.rs` | quinn, rustls (a QUIC read host) |
| `publish_carrier.rs`, `publish_client.rs`, `publish_host.rs` | `transport` with `noise`, notochord |
| `sync.rs` | `join()` takes p2panda-net `Endpoint`/`Gossip`; one test module uses `P2pandaTransport` |
| `resident.rs` | `KnotSyncHost` over `transport::p2panda_transport` |
| `clip_evidence.rs`, `endpoint.rs` | `transport::BlobStore` |
| `bin/knot_endpoint.rs` | `mere-fetch`, `inker`, `script-rhai` (a bin-only dependency that the library carries) |

A `peers` feature over those modules, `default = []` like `embed-bert` and
`mere-overlay`, with `graphshell-endpoint`'s `door` and `stdio` named as their
own feature, would let each consumer choose: Djinn needs the sync host, Turnstone
needs publishing and `BlobClipEvidenceStore`, and Knot desktop needs neither.
Knot desktop's `mere-retention` would then be an offline retention rather than a
retention that implies a network. The `door` is a local contract (custody over
the application door), not an Internet link, but it is also the thing an offline
Knot with no resident would have to replace with a local key provider.

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
the feature own those dependencies (`dep:` entries, with `knot-editor/peers` once
that exists) would make the flag mean what it says. Djinn is a legitimate
`peers` consumer, because it uses Knot's sync host, and would opt in by name.

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
  Reticulum engine unnoticed.
- Neither check covers the *carried* links above; both print them, so a reader
  can see the gap close.
