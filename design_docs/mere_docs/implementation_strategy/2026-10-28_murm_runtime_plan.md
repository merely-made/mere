# Murm Runtime and Replication Retention Plan

**Status:** 2026-10-28: cut from `2026-07-12_murm_peer_runtime_and_moot_domain_plan.md` and `2026-07-12_deletion_retention_and_native_drop_plan.md`, archived in `archive_docs/2026-10-28/`, with the transport shutdown gate taken from `2026-10-19_family_promotion_plan.md`. The replication foundation (now the Mere-owned `stickleback` package) has its shared processor, `JoinedSpace`, the Moot constitution, roster and aggregate drops, mesh and conversation cutovers, and the native drop codec with file carriage. Open: the multi-domain service API, group-key lifecycle, a conversation retention checkpoint, reference tracing, Iroh and Retinue carriage of drops, live peer command wiring, the four next conversation and diagnostics lanes, and the rest of the Murm family move.

**For:** recasting Murm as the reusable peer-exchange family with Moot as a governed-space domain over one replication foundation, and giving every replicated space one deletion and retention law with a transport-independent drop. Opened by the 2026-07-12 retirement of the sibling posture (the purity rule forced reusable mechanism into two product families and made the application restore the invariant) and by the finding that p2panda already supplies the anti-resurrection primitive (`PruneFlag`).
Not here: browser WebRTC rendezvous and the browser Murm consumer (browser carrier plan, C5); Moot protocol mods, discovery and proof, Meadowcap or other structural-capability providers at the injected seam, and commitment proofs (Moot protocols plan); key custody, recovery and enrollment (custody plan); Retinue's link and resource segmentation and radio work (Retinue's own plans and the low-power managed network plan); gathering and capture (Fleece and Eidetic), local browsing history and Athanor image collection, which keep their owners; DNS-SD browsing and advertisement (Murm browsing with the Djinn registry, consumer in the Graphshell host plan's H10); transport, delivery, rendered host controls, presence and deletion or moderation folding, which keep their domain owners and acceptance boundaries.

Receipts from this plan's runs go to `archive_docs/<date>/`, never into this
plan (DOC_POLICY §8). Evidence that matters is a test in the repository or a
line in Findings.

## Constraints

- Murm is the peer-exchange family; Moot is a governed-space domain over its replication foundation and depends on that crate, not on the high-level conversation API; Mere stays offline-first; Turnstone configures services and never builds `LogSync`, drains p2panda events or decides retention law; local authoring, gossip, LogSync and drop import pass through one accepted-operation path; a murmur is invited exchange identified by its participants, a moot a durable governed space whose identity survives membership changes. From: `archive_docs/2026-10-28/2026-07-12_murm_peer_runtime_and_moot_domain_plan.md:41-71`
- The durable purity rule: the replication foundation owns mechanism, domains own meaning and authority, applications own settings and interaction. From: `archive_docs/2026-10-28/2026-07-12_murm_peer_runtime_and_moot_domain_plan.md:923-930`
- Stop rules: no generic replication machinery under a community-shaped name; `moot` never depends on the direct-conversation facade; no operation verification or insertion callbacks in Turnstone; no `murmuring` kept for a hypothetical protocol; Mere's default graph library never depends on network runtimes; key custody, carrier framing, social authorization and blob liveness are never combined in the replication crate; the current folder layout is never promoted with its public boundary repaired later. From: `archive_docs/2026-10-28/2026-07-12_murm_peer_runtime_and_moot_domain_plan.md:542-552`
- Transport ownership (ruled (b), 2026-07-25): `JoinedSpace` in the replication crate owns the join ceremony; session types appear only inside that crate, and gemot's code names no session type. From: `archive_docs/2026-10-28/2026-07-12_murm_peer_runtime_and_moot_domain_plan.md:598-720`
- Filter at read (ruled 2026-07-24, Mark): admission validates what one operation can prove about itself (signature, moot address, wire grammar, prune flag); the fold and its projections decide what converged authority makes effective. The unfiltered roster stays available so a surface may show an unauthorized contribution as pending. From: `archive_docs/2026-10-28/2026-07-12_murm_peer_runtime_and_moot_domain_plan.md:857-912`
- The intake paths return the same decision for one operation corpus except that aggregate-drop import accepts historical checkpoint authority and prune ancestry which live intake rejects as stale; that exception is pinned by a test and is not to be tidied away. From: `archive_docs/2026-10-28/2026-07-12_murm_peer_runtime_and_moot_domain_plan.md:781-826`
- A roster's `members` is who claims to be present; the authority is `MootGroup` at the authorization seam, stated wherever the roster is consumed. From: `archive_docs/2026-10-28/2026-07-12_murm_peer_runtime_and_moot_domain_plan.md:849-855`
- Phase G (promoting separate `murm` and `moot` repositories) is withdrawn; two requirements outlive it: Mere builds offline without peer features, and committed manifests contain no local paths. From: `archive_docs/2026-10-28/2026-07-12_murm_peer_runtime_and_moot_domain_plan.md:486-512`
- `Cabal*` is an internal noun; product copy says murmur or group conversation; the inherited `mere/cable/v1` ALPN does not establish cabal-club Cable interoperability and any change needs an explicit protocol and version migration; carrier identity, post author identity and persona root stay distinct. From: `archive_docs/2026-10-28/2026-07-12_murm_peer_runtime_and_moot_domain_plan.md:1238-1262`
- Native-protocol adapters (Sennet, Tucket, Signalman, tinySSB) sit at the Comms consumer boundary with their own identities, privacy and receipt semantics; Murm's post grammar does not become their wire format. Receiving or showing a conversation captures nothing and grants no capability or retention; a replication frontier or checkpoint notice grants no permission to gather, capture, fork or mutate what it references; authoring success, transport acknowledgment, remote owner acceptance and a human read stay separate observations. From: `archive_docs/2026-10-28/2026-07-12_murm_peer_runtime_and_moot_domain_plan.md:1195-1215, 1280-1304`
- Retention law: prunable current state is a retention checkpoint plus a tail; no generic permanent tombstone (a shared withdrawal stays a signed negative event until a checkpoint records the absence); use p2panda's `PruneFlag` and `LogPrune`, no second cut certificate; withdrawing a fact, erasing a body, collecting a blob or key and pruning signed headers are four distinct acts and status must say which happened; retention has a floor and a ceiling, and a node whose floor exceeds its ceiling stops claiming the hosting role; the drop is a carrier and never a second sync engine; one ingest path; BLAKE3-native first. From: `archive_docs/2026-10-28/2026-07-12_deletion_retention_and_native_drop_plan.md:54-89`
- Only an author can sign a prune point in their own log, so an inactive author's old bodies can be erased under accepted space policy while small signed headers may remain; status reports `body erased` apart from `history pruned`; removing one operation from mid-log is not a v0 promise. From: `archive_docs/2026-10-28/2026-07-12_deletion_retention_and_native_drop_plan.md:224-242`
- Deletion stop rules: no custom prefix pruning while `PruneFlag`, `validate_prunable_backlink` and `LogPrune` fit; no drop semantics in `transport` or Retinue; a checkpoint is never allowed merely because its snapshot decodes; body erasure is never described as full history deletion; no promise of deletion from hostile peers, backups or devices that held plaintext; no group encryption epoch as a stable namespace id; no private plaintext drops as a fallback; no Retinue radio segmentation optimisation before the catch-up drop works through the file carrier and the importer proves anti-resurrection. From: `archive_docs/2026-10-28/2026-07-12_deletion_retention_and_native_drop_plan.md:710-726`

## Phases

### M1. Processor and runtime service (old Phase B)
The shared policy-before-insert processor and `JoinedSpace` are landed; mesh, Tessera and Murm route local authoring, LogSync, gossip and drop import through it. Left: a reusable multi-domain service API (domains register their extension and policy), the remaining domain checkpoint laws, and peer command wiring (placed in Phase B by one note and in Phase C by another).
- Done when: the intake paths return the same decision for one operation corpus, with the pinned aggregate-drop exception; unauthorized or stale operations leave the live store unchanged; the runtime joins, leaves, resyncs and reports one space without its consumer naming a session, subscription or topic type (met by `JoinedSpace`); injected settings control limits and transport preference.
- Needs: -
- Rulings: transport ownership (b); the aggregate-drop exception.
- From: `archive_docs/2026-10-28/2026-07-12_murm_peer_runtime_and_moot_domain_plan.md:267-308, 781-826, 1026-1027`

### M2. Murm direct exchange (old Phase C)
The native conversation engine runs on the shared store (runtime cutover landed 2026-07-14; configured redb reopen; causal history reads; observed frontiers and authorized retention roots in October). Left: the done-conditions were never ticked, and group authorization, live group-key agreement and distribution, and persisted group-key history still lack a Murm consumer.
- Done when: direct exchange has no hand-rolled p2panda log index; durable reopen and two-peer catch-up pass on the shared store; membership revision and key rotation are distinct and tested; the comms adapter consumes only the high-level Murm service.
- Needs: M6 is the group-key consumer.
- Rulings: naming boundary (`Cabal*`, `Conversation*`, murmur); ALPN stability.
- From: `archive_docs/2026-10-28/2026-07-12_murm_peer_runtime_and_moot_domain_plan.md:310-349, 1263-1265`

### M3. Moot domain service (old Phase D)
Constitution, roster and object lanes, aggregate `Moot` service, delegation and authorized fauna are landed. Left: broader membership and capability policy, and a statement of the roster-claim rule where the roster is consumed.
- Done when: a consumer joins a Moot through one domain API per layer without importing a p2panda crate (met under ruling (b)); the same Moot converges through live sync and native drop; unauthorized membership, checkpoint, retention and prune operations fail before mutation; tessera and constitution remain separate policy rings with explicit inputs.
- Needs: -
- Rulings: filter at read; roster-claim rule.
- From: `archive_docs/2026-10-28/2026-07-12_murm_peer_runtime_and_moot_domain_plan.md:351-429, 849-855`

### M4. Remaining consumers (old Phase E)
Mesh is rebased (met 2026-07-26). Isometry collaboration and other shared domains are not. The modular integration's S5 (comms and sync) is carried here and by the Moot collections plan.
- Done when: one backend and processor implementation serves conversation, Moot and mesh; each domain's two-peer tests cover online exchange, offline catch-up, withdrawal, checkpoint and late-peer return.
- Needs: M1.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-07-12_murm_peer_runtime_and_moot_domain_plan.md:431-446`; `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:555-559`

### M5. Host constraint (old Phase F)
A forward constraint: when a host grows a peer lane it consumes services and never assembles protocol. Turnstone now has a peer lane (its place worker reads the authorized Moot view) and the first criterion holds, no longer vacuously.
- Done when: Turnstone has no direct `p2panda-net`, `p2panda-sync` or store-trait dependency; connect, disconnect, resync, import, export and status are ordinary service commands; the app runs fully offline with peer services disabled; a headed receipt shows honest connected, catching-up, retained, body-erased and prefix-pruned states.
- Needs: M1; the Turnstone repository's peer lane.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-07-12_murm_peer_runtime_and_moot_domain_plan.md:448-484`

### M6. Private group conversation (next lane 1)
Compose one private Murm conversation with the existing Stickleback group sessions and an explicit membership and history-access owner. The current `GroupSession` composes data-scheme DCGKA, and a `Message` profile label alone implements neither message encryption nor its ratchets.
- Done when: acceptance covers invite and join, remove and rekey, durable reopen, renewed admission and offline catch-up, with durable data-epoch history versus the message scheme's forward-secrecy policy chosen explicitly.
- Needs: M2; Stickleback's group sessions and an explicit membership and history-access owner.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-07-12_murm_peer_runtime_and_moot_domain_plan.md:1297-1304`

### M7. Comms consumers (next lane 2)
Protocol-faithful Comms consumers for Sennet and Tucket and the existing Signalman owner; tinySSB enters through foreign-feed admission and its own projection; each owner's retained store is reused and unsupported actions are reported. Also carried: the comms shell's live pieces (the misfin server socket, networked murm cabals, the misfin send path), and the comms-gating items never built (the refresh-ticket versus rotate-identity verb split, the Windows and firewalld bind posture, the `mere/misfin/v1` ALPN, the blueprint-over-protocol ruling and the LXMF store-and-forward finding).
- Done when: Sennet and Tucket consumers and the Signalman mapping project each protocol's own delivery facts and refuse what it cannot do; a tinySSB feed is admitted and projected through its own path.
- Needs: M9's Reticulum source refresh.
- Rulings: native adapters sit at the Comms consumer boundary.
- From: `archive_docs/2026-10-28/2026-07-12_murm_peer_runtime_and_moot_domain_plan.md:1305-1307, 1280-1293`; `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:373-375, 983-985`

### M8. Transport diagnostics (next lanes 3 and 4)
A bounded transport diagnostic report from the existing `peers_for_topic`, `peer_paths`, open connections and sync counters: interface, address source and age, attempted versus active direct and relay paths, last connection and sync failures, and the observation time, with advertised records treated as claims. Diagnostic fields are designed against mDNS and DNS-SD (RFC 6762, 6763), QUIC (RFC 9000) and, for WebRTC, ICE (RFC 8445); measurements name what they measure (QUIC RTT is not application acceptance latency; one-way delay carries the clock requirements of RFC 7679); a standards reference is not a conformance receipt.
- Done when: the report exists with those fields; runtime collection and a consumer view follow as implementation.
- Needs: the device-list accessor in the browser carrier plan's R0.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-07-12_murm_peer_runtime_and_moot_domain_plan.md:1308-1319`

### M9. The Murm family move
`murm`, `mere-transport` and `mere-webrtc-carrier` now live in the continuing Retinue repository's native workspace after the first cutover (2026-10-13), keeping Mere's ancestry, names, versions and MPL boundaries; Stickleback and the host and governance owners stay here. Left: the receiving workspace beyond that cutover, source imports, consumer repins and the repository rename (specification `murm/design_docs/2026-10-13_murm_family_move_plan.md`); main integration of the retained `codex/murm-family-refresh` branch; the optional Reticulum source (159 commits behind Retinue main) needing a separate adapter refresh and acceptance, never a silent repin; the hosted native workflow run, whose queued state establishes nothing; the twelve live-network cases excluded under the OS-monitor restriction; and reusable Fetch and WebSocket wire implementations as a later Murm lane (Genet's Netfetcher policy stays). The Reticulum adoption tails belong here: a forged legacy-binding test, a README section on ALPN mapping, announce discovery and binding, the Phase 3 decision (the Direction trigger has fired and the feature is default-off; Retinue's own plan covers the backend), and whether the reference-discipline section restates its rule or points at Retinue's notice.
- Done when: the planned imports, repins and rename have executed; the Reticulum adapter is refreshed and accepted; the hosted workflow's success is established; live-network, physical-radio and headed-browser receipts, which stay separate, are taken.
- Needs: -
- Rulings: workspace separation under the stack-wide working toolchain and the separate shared MSRV; Stickleback remains in Mere as a neutral shared supplier.
- From: `archive_docs/2026-10-28/2026-07-12_murm_peer_runtime_and_moot_domain_plan.md:1290-1293, 1329-1359, 1384-1387`; `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:1342-1348`

### M10. Transport shutdown gate
The unfiltered Murm transport run hung on `a_killed_peer_off_the_overlay_reads_not_connected_when_its_connection_closes`; the follow-up ran the rest with that case filtered. The source move did not repair transport shutdown, and no unfiltered Murm transport-suite or physical-radio pass is claimed.
- Done when: the unfiltered Murm transport suite runs to completion, including the killed-peer case.
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-10-19_family_promotion_plan.md:155-159, 538-539`

### M11. Retention policy, checkpoints and reference tracing (D1-D2 remainder)
Mesh and the Moot object lane carry typed retention checkpoints, governed policy revisions and atomic body erasure. Left: explicit `PruneFlag` policy cutovers for the remaining domains (moot and tessera records, remaining mesh records); a conversation retention checkpoint (catch-up uses the live per-author frontier meanwhile) and a governed retention policy for murmur history; complete live and checkpoint reference tracing in each domain so blob collection is safe; the mesh snapshot stays inline until live blob carriage can guarantee availability before checkpoint admission; on durable reopen the host must reinstall accepted retention revisions; and the downstream mesh and Moot rerun that stalled after payload hydration.
- Done when: for each remaining domain, full replay equals checkpoint plus tail replay; the accepted frontier never rewinds; a false snapshot digest, foreign space, stale policy revision or unauthorized checkpoint is rejected; diagnostics distinguish live withdrawal, body erasure, blob collection and full prefix prune.
- Needs: M1; M3 for the Moot side.
- Rulings: retention law; deletion stop rules.
- From: `archive_docs/2026-10-28/2026-07-12_deletion_retention_and_native_drop_plan.md:95-101, 487-541, 683-686, 822-823, 905-907`; `archive_docs/2026-10-28/2026-07-12_murm_peer_runtime_and_moot_domain_plan.md:1174-1176`

### M12. Drop carriage, peer wiring and group keys (D4-D5)
The drop codec, protected path, staged import, file carriage, receipt frames and domain selectors are landed. Left: live peer command wiring for receipt frames; Iroh blob and Retinue R4 resource carriage of the same `DropId`; feeding Murm's protector from authorized p2panda group state; Personae and p2panda installing and persisting authorized epoch keys; migrating the remaining domains.
- Done when: file, Iroh and Retinue resource carriage preserve the same semantic `DropId` and identical import reports even when segmentation or protection changes; transport loss and resume change delivery only; private export is impossible without a configured protector; key rotation does not change `SpaceId` or checkpoint identity; each domain has a two-peer test for withdrawal, checkpoint, late-peer return, payload erasure and eventual prefix prune; stale documents that prescribe permanent federation tombstones are corrected.
- Needs: Retinue's R4 resource layer and real private protection (Retinue's plans); M11.
- Rulings: retention law; Willow Drop interoperability stays a separate adapter.
- From: `archive_docs/2026-10-28/2026-07-12_deletion_retention_and_native_drop_plan.md:626-629, 665-708`

### M13. Held items
Deferred with triggers of their own: literal Willow Drop interoperability (Willow Entry, Bab and WILLIAM3) as a separate adapter; a checkpoint-aware log-root transition so inactive authors' headers can go, which is upstream p2panda's; Retinue radio segmentation optimisation after the file-carrier catch-up drop and anti-resurrection proof; privacy classifications, profile names and priorities in each domain's own rollout; a sibling repository for `stickleback`, gated on a real external consumer ("revisit when one appears, and not before").
- Done when: each item's trigger arrives.
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-07-12_deletion_retention_and_native_drop_plan.md:87-89, 234-242, 641-643, 723-724`; `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:424-430`

## Open questions

- Is Phase A closed? Its done-conditions read as met on the removal of the compatibility re-exports, but mesh still depends directly on `p2panda-core`, `p2panda-store` and `p2panda-net`. Declare it closed, or keep it open until those dependencies go. From: `archive_docs/2026-10-28/2026-07-12_murm_peer_runtime_and_moot_domain_plan.md:241-248`
- Should Phase F be restated as non-vacuous, with Turnstone's place worker as the consumer the criteria are held against, or left as written with the correction? From: `archive_docs/2026-10-28/2026-07-12_murm_peer_runtime_and_moot_domain_plan.md:474-484`
- Do moots host live shared graphs? No moot record carries a graph edit, so a peer's petition arriving over the wire has no substrate; a new edit-batch event kind would be a product decision, not an implementation detail. From: `archive_docs/2026-10-28/2026-07-12_murm_peer_runtime_and_moot_domain_plan.md:874-882`
- Does Stickleback's September group-key lane (add, remove and rotate frames, parking records sealed to a missing epoch, forgetting epochs) satisfy "feed Murm's protector from authorized group state", or does binding group keys to the drop protector stay open? From: `archive_docs/2026-10-28/2026-07-12_deletion_retention_and_native_drop_plan.md:667-680`
- Is rotated-signer bootstrap still a cross-store package problem? The checkpoint status says the object checkpoint names constitution authority proven by another store, while the later aggregate-drop entry records constitution evidence carried with the drop so a fresh peer bootstraps through a rotated checkpoint chain. From: `archive_docs/2026-10-28/2026-07-12_deletion_retention_and_native_drop_plan.md:527-530, 895-900`
- Where does peer command wiring for receipt frames belong, Phase B (service API) or Phase C (conversation)? From: `archive_docs/2026-10-28/2026-07-12_murm_peer_runtime_and_moot_domain_plan.md:283-284, 1026-1027`
- Should the Phase B first done-condition be reworded as the 2026-07-25 finding proposes (same decision for one corpus, except the aggregate-drop exception)? The phase text was never changed. From: `archive_docs/2026-10-28/2026-07-12_murm_peer_runtime_and_moot_domain_plan.md:298-299, 812-816`

## Findings

- 2026-07-26: done-conditions are claims about the tree and decay as the tree moves; three had drifted (Phase E #1 already met, Phase F's subjects gone, Phase G void). Auditing them costs one pass whenever a phase is declared complete. From: `archive_docs/2026-10-28/2026-07-12_murm_peer_runtime_and_moot_domain_plan.md:556-596`
- 2026-10-11: Murm's `RetentionCheckpoint` extension (post type 6) carries p2panda's `PruneFlag` and a signed authority revision; the default conversation policy refuses it, possession of a cabal secret grants no retention authority, and older Murm readers refuse the unknown kind. The checkpoint removes only the signer's own earlier log records and is not a snapshot of old topics. From: `archive_docs/2026-10-28/2026-07-12_murm_peer_runtime_and_moot_domain_plan.md:1146-1180`
- 2026-10-13: Murm's `local_peer_id` returned the persona master key although production transports accept a scoped `DerivedKeypair`; it now reads the transport's own identity. From: `archive_docs/2026-10-28/2026-07-12_murm_peer_runtime_and_moot_domain_plan.md:1238-1255`
- 2026-10-06: surfaces read the authorized fauna view in three places (`ports/moot/src/captured_web.rs` and two Turnstone place files). From: `archive_docs/2026-10-28/2026-07-12_murm_peer_runtime_and_moot_domain_plan.md:913-922`

## Progress

- 2026-10-28: cut from `2026-07-12_murm_peer_runtime_and_moot_domain_plan.md` and `2026-07-12_deletion_retention_and_native_drop_plan.md`, plus the transport shutdown gate from `2026-10-19_family_promotion_plan.md`; nothing new landed.
