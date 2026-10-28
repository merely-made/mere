# Browser Carrier, Reachability and Persona Egress Plan

**Status:** 2026-10-28: cut from `2026-08-25_browser_webrtc_carrier_plan.md`, `2026-08-03_reachability_rungs_and_privacy_lanes_plan.md` and `2026-06-25_persona_transport_unlinkability_plan.md`, archived in `archive_docs/2026-10-28/`. The WebRTC carrier is physically proven through C4 (a real Graphshell client mounts a native-owned projection over WebRTC, direct and forced-relay, with reconnect by re-admission); public rendezvous, the release offer and the iroh relay lane (C5-C7) are open. The reachability ladder has R0 (peer discovery) and R1 (cached dial hints, LAN) landed with R2 and R3's emissary and Arti lanes open; persona egress has its device fabric landed and Mode 1, own-cluster v1 and the research tiers unbuilt.

**For:** three questions about how a peer is reached and how it is seen. First, how an ordinary browser with no extension joins a bounded live Graphshell session whose native application keeps state and authority (the browser WebRTC carrier ruling, C5-C7). Second, how two devices that hold only each other's identity find a route beyond the LAN without a third party seeing the mesh (the reachability ladder opened by the 2026-08-03 relay-leg experiment that disproved a relay-URL-only dial). Third, how a persona's network presence stays unlinkable to the person and to the persona's siblings (the persona transport brief's four leaks and its device fabric).
Not here: WebRTC audio and video tracks and media decode (calls and media plan); the portable release envelope and self-sufficient Luggage stage that C6 needs (family update resident plan); grant semantics on `pairing_id` and the legacy-record mint, the H6 addendum S2/S3 and the H10 DNS-SD consumer (Graphshell host plan); first contact through gossip bootstrap (device pairing plan, D1b); DNS-SD browse and advertise as Murm and Djinn work (Murm runtime plan, lane 3 of its diagnostics; the Djinn registry is F3 of the Djinn family resident services plan); vault custody and the lock (custody plan); the Moot constitution and replication carriage (Murm runtime plan); the retinue radio lane (low-power managed network plan).

Receipts from this plan's runs go to `archive_docs/<date>/`, never into this
plan (DOC_POLICY §8). Evidence that matters is a test in the repository or a
line in Findings.

## Constraints

- Layering of the browser join: `mer3ly.net` serves a versioned client and relays opaque signaling; WebRTC supplies the encrypted data channel (TURN only when direct fails); a carrier challenge authenticates the native host and derives Notochord's connection-specific `shared_link`; the browser redeems the private capability into a narrow delegation for an ephemeral Personae subject; Notochord admits that subject before the application reads a byte; Graphshell carries projections, diffs, resources and admitted intents; the native application stays authoritative. From: `archive_docs/2026-10-28/2026-08-25_browser_webrtc_carrier_plan.md:25-44`
- The carrier reports no authenticated initiator; signaling data never becomes a carrier fact because a Worker relayed it; `InviteV1` grants only a named service action with bounded expiry and never alters executable trust; a Luggage release reference is a claim until a separately trusted publisher key verifies the exact signed manifest; the first visit still trusts the `mer3ly.net` HTTPS origin for the bootstrap verifier; joining never installs a release or changes publisher, feed, channel or update policy; a refused stream never reaches Graphshell; browser and native share wire types and test vectors, not one runtime. From: `archive_docs/2026-10-28/2026-08-25_browser_webrtc_carrier_plan.md:46-86`
- The bilateral `Transport` trait stays intact until a second listener-only carrier proves a shared acceptor trait useful. From: `archive_docs/2026-10-28/2026-08-25_browser_webrtc_carrier_plan.md:83-84`
- Sans-I/O session core (ruled): `RetainedEndpointSession` keeps its public API and is the blocking adapter over an I/O-free core; the browser drives the same core through an event adapter. No cross-origin isolation, no genet change, no second copy of the protocol. From: `archive_docs/2026-10-28/2026-08-25_browser_webrtc_carrier_plan.md:289-309`
- Receipts for the browser path are taken by the page-side scenario lane (`?scenario=`, genet-probe verbs), not by hand. From: `archive_docs/2026-10-28/2026-08-25_browser_webrtc_carrier_plan.md:790-798`
- Release identity has one definition: `ReleaseRefV1` lives in `luggage::release` (the crate is Wasm-clean by feature) and the carrier consumes it; a different field set is a `ReleaseRefV2`, never an edit. From: `archive_docs/2026-10-28/2026-08-25_browser_webrtc_carrier_plan.md:1395-1446`
- C5 treats "at least one resolvable remote candidate" as a hard precondition: default Chrome emits only mDNS `.local` host candidates, str0m has no mDNS resolver, and a candidate-stripped offer hangs both ends in `checking`. From: `archive_docs/2026-10-28/2026-08-25_browser_webrtc_carrier_plan.md:1170-1178`
- Direct WebRTC with Notochord stays the product path whatever C7 shows; C7 is a fallback for networks that block WebRTC, relay-only by nature, and starts only for a named consumer the direct carrier cannot reach. From: `archive_docs/2026-10-28/2026-08-25_browser_webrtc_carrier_plan.md:1015-1052`
- Plane split for the ladder: iroh is the byte plane; identity-routed lanes (retinue announces, emissary destinations) are the control and rendezvous plane; radio is the constrained plane and zero-infrastructure rendezvous; nothing moves blobs off iroh. From: `archive_docs/2026-10-28/2026-08-03_reachability_rungs_and_privacy_lanes_plan.md:41-46`
- A dial hint is disposable and never an admission: a parse or dial failure is logged and skipped, a hint for an unpaired writer is ignored, re-pairing does not discard a learned route, unpairing drops it, and `node_id` stays "deliberately not a ticket". From: `archive_docs/2026-10-28/2026-08-03_reachability_rungs_and_privacy_lanes_plan.md:146-170, 203-246`
- The retinue lane in `mere-transport` is trusted-mesh only until retinue R8 (IFAC) and R9 (ratchets) land. From: `archive_docs/2026-10-28/2026-08-03_reachability_rungs_and_privacy_lanes_plan.md:7-8`
- Noise composes over iroh and is not a `Transport`; the Noise identity is a parameter distinct from the carrier key (layered identity). From: `archive_docs/2026-10-28/2026-08-03_reachability_rungs_and_privacy_lanes_plan.md:320-354`
- Reputation and anonymity are one slider: a main persona is fully linkable, a forked persona has depreciated standing and relay diversity, a burner is a fresh zero-standing chain on the most unlinkable transport. From: `archive_docs/2026-10-28/2026-06-25_persona_transport_unlinkability_plan.md:23-43`
- Per-persona NodeId is a caller choice: `derive_keypair(BLAKE3('persona' || persona_id))`. Discovery is optional and off for non-main personas. From: `archive_docs/2026-10-28/2026-06-25_persona_transport_unlinkability_plan.md:57-70, 88-96`
- Own-cluster routing hides current location, not the household from the home ISP; routing every persona through one home node shares one IP, so those faces are linkable to each other behind it; the product states this boundary plainly. From: `archive_docs/2026-10-28/2026-06-25_persona_transport_unlinkability_plan.md:107-139, 280-283`
- Decided 2026-06-25: the device fabric is the direction: per-device exposure flag (hidden by default, exposed egress an explicit opt-in), per-persona egress binding, authored over the personal mesh. I2P and Yggdrasil are skipped for personas. From: `archive_docs/2026-10-28/2026-06-25_persona_transport_unlinkability_plan.md:159-208, 233-236, 285-287`

## Phases

### C4 carried rows
Three rows the carrier phases carried forward and no receipt text records as met.
- Done when: the headed browser-log, referrer and signaling hygiene check shows the invite fragment and its redemption secret absent from every log path; the snapshot resume branch is exercised against a revision aged out of the retained window; a packet capture evidences, rather than reasons, that the TURN relay carries ciphertext only.
- Needs: a headed browser session; a live TURN relay.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-25_browser_webrtc_carrier_plan.md:199-202, 1338, 1375-1377`

### C5. Public rendezvous
Deploy `https://mer3ly.net/join/<rendezvous>#<private-capability>` after C4. A pinned bootstrap loader resolves the invitation's `ReleaseRefV1`, verifies the detached manifest signature against its own publisher trust store, selects the browser artifact and verifies its digest and signature before importing JavaScript or instantiating Wasm. A Worker brokers opaque offer, answer and ICE messages and mints short-lived TURN credentials; a Durable Object may hold bounded rendezvous mailboxes and presence but no application state and no admission. The page shows verified or unknown publisher, source revision, release identity, client selection, requested action and invite expiry before joining. Rate, size and lifetime limits are user-configurable at the native host within protocol ceilings.
- Done when: a fresh browser with no extension joins through the public URL; the URL fragment is absent from HTTP requests and service logs; signaling substitution cannot impersonate the signed native host; destroying or revoking the invite prevents a later join; direct and TURN paths both close the C4 receipt through the public service; official release manifests and browser bundles verify before client code executes while an altered bundle is refused before import; an unknown fork publisher is never described as Merely-signed and cannot add itself to the trust store through the invitation; service deletion leaves the native application state intact.
- Needs: C4 carried rows; a Luggage verification entry point for the Wasm loader (the Wasm-clean core can name and compare a release but not verify one, and exporting one adds public API to a shipping crate); a resolvable remote candidate.
- Rulings: sans-I/O core; Luggage Wasm-clean by feature (2026-08-28, 2026-08-30).
- From: `archive_docs/2026-10-28/2026-08-25_browser_webrtc_carrier_plan.md:940-971, 1170-1178, 1417-1420`

### C6. Luggage release offer and native adoption
Make the release reference useful beyond display. Luggage owns a Wasm-clean signed-envelope core; its native side keeps feed polling, verified staging, update policy and platform apply. The signed manifest names application id, version, source revision, supported invitation and Graphshell wire versions, and artifacts by kind and target. Artifact locators live in a disposable `ReleaseOfferV1` outside the signed content, so a mirror, native host or peer may change without changing the release identity. A fork is adopted only by two visible changes (its publisher key and its update feed); joining its session performs neither.
- Done when: one signed manifest names distinct browser, Windows, Linux and macOS hashes under one release identity and source revision; native and browser verifiers derive the same `ReleaseRefV1` from frozen vectors; corrupted manifests and artifacts fail identically whether bytes came from HTTPS, WebRTC or Iroh; changing only offer locators preserves the release reference while changing any signed content fact changes it; the host can seed an exact platform artifact by content hash without becoming publisher authority; an installed app stages and applies the offer through Luggage's self-sufficient signed stage while a browser download produces a verification receipt and waits for explicit installation; accepting a session leaves publisher trust, feeds, channels and update policy byte-for-byte unchanged.
- Needs: C5's public live-session receipt; the portable release envelope and self-sufficient stage in the family update resident plan.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-25_browser_webrtc_carrier_plan.md:973-1013, 1299-1302`

### C7. The iroh browser relay as a second lane
For networks that block WebRTC outright. Candidate is iroh 1.1's own browser support (`iroh-relay` 1.1.0 on `wasm32-unknown-unknown` via `ws_stream_wasm`; a browser `Endpoint` with `presets::N0` dialling by `EndpointId` over an ALPN), not the external donor. Settled: it is relay-only, and it costs about 10.8 times the browser payload of the direct carrier because iroh compiles into the Wasm.
- Done when: a headed browser reaches a native host through a relay and Notochord admits the session on facts the relay cannot forge; `shared_link` binds iroh's connection identity (no DTLS fingerprints here, so the C0 transcript survives and `fingerprint.rs` does not); reconnect runs fresh admission as C3 requires of the direct lane; the relay operator learns no Graphshell key and no application capability; setup and resume latency are measured against the direct carrier and the payload cost is re-measured optimised (`wasm-opt`, `opt-level="z"`, LTO, `panic="abort"`), since both current figures are unoptimised floors.
- Needs: C4 closed (it is); a named consumer the direct carrier cannot reach.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-25_browser_webrtc_carrier_plan.md:1015-1052`

### CR. Carrier residue
Items the carrier phases left open by name, each with its own trigger.
- Done when: `sctp_window_bytes` default is revisited for long links (the 16 KiB default capped throughput at about 16 KiB per RTT; the dedicated-thread placement carries a larger window); wildcard bind with a multi-address advertise list no longer mislabels inbound datagrams as `advertise[0]` (either one advertised address or per-packet destinations); the str0m `do_poll_output` teardown recursion is filed upstream; the SDP fingerprint parser (uppercase hex only, per RFC 8122) is revisited when a nonconforming stack is met; a type-level fix for the `CarrierControl` early-drop hazard lands if the hazard appears a third time; the one-component-instance-per-page limit, the duplicated `read-fonts`/`skrifa` stack (genet-owned) and the web manifest's `[profile.dev]` setting are each resolved or recorded as bounds.
- Needs: genet for the font stack.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-25_browser_webrtc_carrier_plan.md:506-509, 903-905, 1263-1268, 1294-1297, 1319-1325, 1522-1525`

### R0. Local-link discovery: what remains
Peer discovery is landed (p2panda mDNS, active in the product hosts; policy owned once by `P2pandaHostPolicy` and `P2pandaOverlayHost`). Left are the seams around it.
- Done when: `mere-transport` can enumerate what discovery found (the device-list accessor DNS-SD browsing and the Djinn registry need first); macOS joins as a discoverable peer from a signed bundle carrying the local-network usage declaration (an unsigned binary is denied local-network egress on Sequoia); F3's service browser has a short DNS-SD service type and an explicit network selector, qualified apart from the upstream private naming scheme (the service string is 41 characters, over the 15-character service-name limit of RFC 6763 §7); the dead `swarm-discovery` fork pin is dropped at the next revisit.
- Needs: signed macOS bundle; Murm and Djinn work on browsing and advertisement.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-03_reachability_rungs_and_privacy_lanes_plan.md:65-79, 88-114, 123-144`

### R1. Cached dial hints: the genuinely remote receipt
R1 is landed on the LAN for Graphshell and Knot. The LAN's direct IPs are stable across restarts, so no receipt yet shows which component of the hint carried a dial.
- Done when: an off-LAN reconnect with both ends restarted, no ticket and mDNS dead succeeds through the cached hint alone, and the receipt isolates whether the direct address or the relay component carried the dial. Also watched, not scheduled: the resident host is a second writer of the settings file with a milliseconds-wide lost-update window against CLI pair and unpair (a generation counter "if it ever bites").
- Needs: a remote second device.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-03_reachability_rungs_and_privacy_lanes_plan.md:172-201, 247-249`

### R2. Announce-carried dial hints
Carry the relay-tagged iroh `EndpointAddr` in the retinue announce so the mesh is its own address cache. The announce now sends no app data (the 96-byte binding made it 263 bytes, over a 255-byte LoRa frame; the signed Retinue identity carries the Mere key), so the original premise of extending authenticated app data is overtaken. A serialized hint is roughly 150-200 bytes: fine on TCP, tight on LoRa airtime. Either RF announces carry a minimal marker with the full hint fetched over the link, or the full hint goes everywhere with the airtime cost accepted; decide against measured announce sizes.
- Done when: a device with no cached hint and dead mDNS learns a sibling's iroh address from an announce over a TCP transport node and completes a blob fetch through it.
- Needs: the H6 addendum S2/S3 in the Graphshell host plan (the transfer lane is the first consumer of durable remote reachability); the question below on reopening R2.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-03_reachability_rungs_and_privacy_lanes_plan.md:251-292, 391-395`

### R3a. Emissary (I2P) lane
An embeddable Rust I2P router (`emissary-core`) is forked and local. Consumers in order: the netDB rendezvous receipt; the Turnstone `i2p://` web lane (scheme-routing through the embedded proxies, then a SAM streaming lane in the fetch stack); Retinue over I2P (an emissary-backed interface makes WAN links private and interoperable with existing Reticulum-over-I2P nodes). Operational facts to carry: the router needs a reseed on first run; embedding makes the resident host an I2P participant, so transit policy, ports and battery are owner decisions; parity with Java and C++ I2P is a program, kept as its own checklist once the first consumer is live.
- Done when: a sibling with mDNS dead, no ticket and no cached hint resolves the iroh dial hint published at a persona-derived destination through the netDB and fetches; then the `i2p://` web lane and Retinue over I2P follow in that order.
- Needs: emissary entering the tree as a dependency.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-03_reachability_rungs_and_privacy_lanes_plan.md:296-318, 382-383`

### R3b. Noise tail
The in-stream Noise plane is landed over iroh streams. Not done and not needed yet: Noise over iroh datagrams (the low-latency composition) and PSK, `Noise_KK` or `Noise_IK` patterns for capability-scoped sessions; `XX` stays the default while both ends are ours.
- Done when: a consumer needs either composition; not scheduled.
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-03_reachability_rungs_and_privacy_lanes_plan.md:355-363`

### R3c. Arti (clearnet plane)
Consumer posture: Tor's network is the common privacy infrastructure for ordinary browsing. The concrete consumer is a "browse privately" toggle in Turnstone's web lane; nothing depends on it.
- Done when: the toggle is wanted and Arti is evaluated against it; not scheduled.
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-03_reachability_rungs_and_privacy_lanes_plan.md:365-371`

### E1. Persona Mode 1 and relay diversity
Per-persona distinct NodeId (`bind` with a derived endpoint key), discovery off for non-main personas, ticket-only invites, and a different relay (or a small rotation) per persona. Defeats the directory and, with diversity, the relay operator; leaves a direct counterparty's view of your IP.
- Done when: two personas on one device present two unlinkable NodeIds, share no discovery record and use different relays, so a directory watcher and moot members cannot correlate them. Explicitly does not defeat a counterparty who sees your IP across two faces.
- Needs: "which persona's endpoint does this moot ride" threaded through the moot and sync layer (today one transport per process; N endpoints cost N sockets, N holepunch states and battery).
- Rulings: device fabric decision of 2026-06-25.
- From: `archive_docs/2026-10-28/2026-06-25_persona_transport_unlinkability_plan.md:88-105, 258-260, 276-278`

### E2. Own-cluster v1 and the egress seam
Route a persona's outward dialling through an always-on home node over a WireGuard tunnel, re-originating below iroh so iroh need not expose deliberate relay-through-node. The identity-level fabric is landed (`DeviceRoster`, `DeviceExposure`, `ACTION_TRANSPORT_EGRESS`). Still absent: `PersonaManifest.egress`, session-startup transport resolution and the `connect`-site check.
- Done when: a roaming device's persona egresses through the home node; peers see the home IP, not the current network; zero third-party trust; the home-ISP and household linkability boundary is stated plainly in the UI so no one mistakes location privacy for anonymity.
- Needs: an always-on exposed node owned by the user; E1's per-persona endpoint routing.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-06-25_persona_transport_unlinkability_plan.md:79-86, 107-208, 261-264, 280-283`

### E3. Mesh-job relay and friend relay
Path 2 models "relay through device D" as a mesh M2 compute job (buildable once M2 resource-routing lands). Friend or social relay widens the cluster to trusted friends' always-on nodes, mapping onto gemot and moothold federation tiers; the cost is that a friend's node sees traffic metadata.
- Done when: a persona egresses through a mesh-job relay once M2 lands; a friend-relay rotation gives a small anonymity set; neither is scheduled.
- Needs: mesh M2 resource-routing; E2.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-06-25_persona_transport_unlinkability_plan.md:144-147, 210-220, 265-266`

### E4. Research tier: Nym and the iroh-relay-protocol path
Nym is the single option touching a global passive timing adversary, at the price of third-party trust and a QUIC mismatch; kept only as an optional burner egress layered after your own. A deliberate relay-negotiation protocol on the peer is probably not worth it for v1.
- Done when: research only; adopt Nym if the threat model includes a global timing adversary, and the iroh-relay-protocol path only if in-protocol relay proves worth it.
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-06-25_persona_transport_unlinkability_plan.md:148-150, 222-231, 267-268`

## Open questions

- Does R2's design need reopening now that the announce carries no app data: reopen it and decide where the hint rides (a marker with the full hint fetched over the link, or app data reintroduced within 255 bytes), or keep R2 as written and settle it under the open design point? From: `archive_docs/2026-10-28/2026-08-03_reachability_rungs_and_privacy_lanes_plan.md:272-288`
- Is the Noise in-stream plane R3 progress or its own lane or rung, separate from the emissary gate on the privacy lanes? From: `archive_docs/2026-10-28/2026-08-03_reachability_rungs_and_privacy_lanes_plan.md:359-363`
- Is remote reachability needed before R1, R2 and R3 land? n0 DNS discovery stays the WAN-now shortcut, superseded as the preferred path but not retired, and a self-hosted iroh relay for the byte plane behind hard NAT is a separate decision (also persona own-cluster piece (a)). From: `archive_docs/2026-10-28/2026-08-03_reachability_rungs_and_privacy_lanes_plan.md:373-379`; `archive_docs/2026-10-28/2026-06-25_persona_transport_unlinkability_plan.md:111-118`
- Does Mode 1 with per-persona egress stay in this plan or move under dramatis, now that the device fabric landed through the wallet plan? From: `archive_docs/2026-10-28/2026-06-25_persona_transport_unlinkability_plan.md:270-272`
- Does iroh 1.0.3 (the current pin) expose deliberate relay-through-node? The brief still cites 0.98, and the pin has not been checked. From: `archive_docs/2026-10-28/2026-06-25_persona_transport_unlinkability_plan.md:71-74`
- Own-cluster v1 shape: an egress-only tunnel (mask the IP, client runs iroh) or a home-hosted endpoint (the persona lives on its egress device, better availability, needs a control channel)? From: `archive_docs/2026-10-28/2026-06-25_persona_transport_unlinkability_plan.md:187-196, 291-292`
- Fallback when the user owns no always-on exposed node: Mode 1 (real IP and public relays) or an external egress (a friend's node, Tor)? From: `archive_docs/2026-10-28/2026-06-25_persona_transport_unlinkability_plan.md:197-203, 293`
- Relay-only-forced (always hide the IP) or direct-preferred (fast when direct)? From: `archive_docs/2026-10-28/2026-06-25_persona_transport_unlinkability_plan.md:294`
- Friend-relay coordination, availability and anonymity-set sizing. From: `archive_docs/2026-10-28/2026-06-25_persona_transport_unlinkability_plan.md:295`
- Is the global-timing tier (Nym) ever in scope, or is "unlinkable to the directory, members and a counterparty's current-location read" the product ceiling? From: `archive_docs/2026-10-28/2026-06-25_persona_transport_unlinkability_plan.md:296-297`
- Voluntary hosting (`HostingCommitment`, the tessera reward ladder) is the minimal case of cluster routing and ties to this plan, but no home for it is named. From: `archive_docs/2026-10-28/2026-06-25_persona_transport_unlinkability_plan.md:238-247`
- Who takes the separate work the carrier plan listed as outside it (Gemini or HTML projection from Knot; multi-host fanout, SFU operation and cloud-held graph state; OS code-signing procurement and unattended first installation)? The old plan made each follow the live session's authority, relay and reconnect receipts, which C4 has closed. From: `archive_docs/2026-10-28/2026-08-25_browser_webrtc_carrier_plan.md:1054-1062`

## Findings

- 2026-09-01: R0 was built without a home in any plan: `P2pandaOverlayHost` and the product mDNS surfaces. The standards survey's claim that no local-link discovery story existed was true only for DNS-SD service browsing. The service on the wire is a p2panda one (`_p2pandav1<network-digest>._udp.local` since the October 13 supplier snapshot), so a diagnostic derives the exact name from the configured network id; old and new advertisements do not discover each other and explicit endpoint tickets remain the mixed-version bootstrap. From: `archive_docs/2026-10-28/2026-08-03_reachability_rungs_and_privacy_lanes_plan.md:48-79, 397-411`
- 2026-10-06: the `iroh-mdns-address-lookup` fork pin was never in effect (`mere-p2panda-net` required 0.5.0, so the 0.4.0 fork was an unused patch) and was removed; the macOS stall is policy, not sockets. The H10 receipts dialled a known peer id; first contact through gossip bootstrap fails (device pairing plan, D1b). From: `archive_docs/2026-10-28/2026-08-03_reachability_rungs_and_privacy_lanes_plan.md:88-114, 135-145`
- 2026-08-26: str0m's `do_poll_output` recurses once per queued SCTP packet at teardown, so stack depth equals the outstanding SCTP window; proven by dose-response and mitigated by `CarrierConfig::sctp_window_bytes`. From: `archive_docs/2026-10-28/2026-08-25_browser_webrtc_carrier_plan.md:1255-1268`
- 2026-08-26: an ICE restart keeps the DTLS connection and needs no re-admission; a new DTLS connection is a new link and runs host challenge and Notochord admission again; the browser keeps its C2 delegation across a drop, the invite stays one-use and the C2 TTL clamp bounds the reconnect window. From: `archive_docs/2026-10-28/2026-08-25_browser_webrtc_carrier_plan.md:1221-1230`
- 2026-06-25: the persona transport seam is a design gap (no persona-owned transport config), not an iroh limitation; cluster routing hides current location but not the household from the home ISP. From: `archive_docs/2026-10-28/2026-06-25_persona_transport_unlinkability_plan.md:313-327`

## Progress

- 2026-10-28: cut from `2026-08-25_browser_webrtc_carrier_plan.md`, `2026-08-03_reachability_rungs_and_privacy_lanes_plan.md` and `2026-06-25_persona_transport_unlinkability_plan.md`; nothing new landed.
