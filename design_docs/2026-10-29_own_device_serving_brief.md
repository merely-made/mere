# Own-device serving brief: keeping a published site up across your devices

**Date:** 2026-10-29
**Kind:** research brief digesting a chat chain with Mark (2026-10-29). It began as a
Knot-versus-IDE comparison and reached Beaker Browser, peer publishing and the
product split. Analysis, a primary-source reading of Beaker's post-mortem and
vocabulary alignment are added here.
**Status:** direction only. Nothing is scheduled.
**Anchors:**
[persona transport unlinkability plan](archive_docs/2026-10-28/2026-06-25_persona_transport_unlinkability_plan.md)
(the own-device cluster and the "availability anchor"),
[moot tiers and voluntary hosting brief](mere_docs/implementation_strategy/2026-05-07_moot_tiers_and_voluntary_hosting_brief.md),
[Gemot author-offline publishing](https://github.com/merely-made/gemot/blob/main/design_docs/moothold_docs/implementation_strategy/2026-06-12_moot_object_m1_plan.md#community-collections-and-author-offline-publishing-2026-09-04),
[family update resident plan](mere_docs/implementation_strategy/2026-10-28_family_update_resident_plan.md)
(the built device resident and Djinn as desktop composition root),
[Knot device resident V1 receipt](https://github.com/merely-made/knot-editor/blob/main/crates/knot-editor/docs/2026-08-22_device_resident_v1_receipt.md),
[Knot governed native Scroll process receipt](https://github.com/merely-made/knot-editor/blob/main/design_docs/2026-09-11_small_web_authoring_plan.md#governed-native-scroll-process-receipt-2026-10-22),
[Turnstone peer-web reframe](https://github.com/merely-made/turnstone/blob/main/design_docs/2026-07-28_turnstone_peer_web_reframe.md),
[family composition thesis](2026-08-12_family_composition_thesis_brief.md),
[GUI catalog §8](mere_docs/design/2026-10-26_gui_catalog_design.md) (the contribution kinds).
The Knot-facing half of the chain (writer ergonomics, scripts that edit, the
project model) is in Knot's
[writer ergonomics brief](https://github.com/merely-made/knot-editor/blob/main/design_docs/2026-10-29_knot_writer_ergonomics_brief.md).

## Origin framing

Mark, 2026-10-29, on Beaker's idea:

> Yeah, people should be able to browse, edit, and publish peer-to-peer sites.
> That is, hell yeah, that is exactly what I want to do.

On the problem to solve:

> Okay, nobody hosting when you're offline, and the no mobile story. Those are two
> crucial issues. I think the mesh is part of how to handle that. Mesh and daemons.
> Because if all of your devices are asleep at once, and I think then you're kind of
> SOL, right? You're going to need somebody else, another device to serve it. But if
> one of your devices is up, you know, and then maybe two of your devices are up, and
> then maybe one of your devices, a different device is up, then really what needs to
> be done is to distribute the work of serving, you know, across your mesh, right?

On applications as modules:

> I just like the idea of the apps themselves, the capabilities that they offer, also
> being modules, right? Like, also being things that can either fuel your graph or
> allow you to edit content or provide you with, like, renderers or parsers that you
> need to understand some data, or you know, allow you to render content and present
> it to other people who maybe don't even have that mod rendered or loaded, right?

## 1. Beaker, from its own post-mortem

Source: Paul Frazee's
[archive notice](https://github.com/beakerbrowser/beaker/blob/master/archive-notice.md),
2022-12-27. Blue Link Labs built Beaker on Dat (later Hypercore). Frazee had joined
Bluesky earlier in 2022. The last stable release was 1.1.0 in December 2020.

**Correction to the chain.** The chain's first answer guessed protocol churn and the
cost of maintaining a browser as causes. The post-mortem names neither. It names:

- **No stickiness.** The demo was strong, but usage stalled. In hindsight the minimum
  viable product was a static-site maker, and it was not sticky.
- **Four problems that fed each other:**
  - sites unreachable from ordinary browsers;
  - no mobile browser;
  - no safe way to sync state between a person's devices;
  - APIs too weak to build good applications.
- **Costs of pure peer-to-peer:** discovery and delivery, availability (users go
  offline), slow first connections, hard debugging, device resource use, and
  per-device indexes that don't scale.
- **Hard to add thick-client UI to the web.** Toolbars, side panels and location-bar
  buttons either went unnoticed or got in the way.

The closing lesson is that mission needs product-market fit.

| Beaker problem | Family position |
|---|---|
| Sites unreachable from ordinary clients | Knot publishes small-web formats with existing protocols and clients. Peer addressing is additive, not required to read. |
| No mobile | Open. Phones are authors first (§2). |
| No safe multi-device sync | Knot device resident V1: paired devices replicate, and unpairing revokes serving and fetch. Physical two-device receipts remain open. |
| Weak APIs, web thick-client friction | Native hosts and contributions, not injected web APIs. |
| Availability | This brief (§2). |
| Discovery | Announces on Reticulum/retinue, iroh discovery; Gazetteer is incubating. |
| Stickiness | Knot must be useful offline and alone as a writing environment. Publishing adds to that value; it is not the whole of it. |

## 2. Serving across your own devices

The rules, as stated in the chain:

1. **A site is up whenever any of your devices is awake.** Availability is the union
   of your devices' uptime. Serving is handed between them as they wake and sleep.
2. **Replicate whole; don't shard.** A published site is a small, immutable, signed
   snapshot. Every device keeps a full copy, readers fetch from whichever answers,
   and load spreads with no coordinator. Shard only large media.
3. **Serving is not authority.** A serving device needs the signed snapshot, not the
   signing key or the vault. Readers verify against the site key, so a server cannot
   forge or alter. Lower-trust devices can serve: an old phone, a Raspberry Pi, a
   friend's machine.
4. **The address names the key, not a device.** Readers find a current holder through
   announces or discovery.
5. **The signed published head carries a version.** Readers detect a stale copy and
   look for a newer holder.
6. **Publishing completes once N other devices hold the version.** A laptop that
   publishes and sleeps must not strand it. The UI shows "held by N devices."
7. **Last one out.** Before sleeping, a device checks for another reachable holder.
   If it holds the only copy, it warns or briefly delays sleep. On macOS it can ask
   the system to stay awake.
8. **The device resident serves.** Holding, announcing and serving published copies
   becomes a duty of each device's resident, so serving continues with the editor
   closed. The device resident is built (the Djinn resident is the desktop
   composition root). Knot's governed native Scroll and Micron publisher/resident
   processes are the publishing-side starting point.
9. **Phones are authors, and serve only while the app is open.** iOS and Android
   suspend background work. An always-on device (desktop, NAS, Pi) is what makes the
   set dependable.

**All devices asleep.** Only other people's devices remain. All are safe because the
content is signed:

- readers re-serve what they fetched (public sites only);
- friends host each other's sites by agreement (the voluntary hosting brief);
- a rented host holds only the signed snapshot (the brief's "cheesecloth pinning").

**Private material differs.** The vault stays on your own devices, or on a host that
holds sealed blobs it cannot read (Stickleback retention).

**Relation to existing docs.** The persona transport plan's availability anchor is a
single always-on node: a persona stays reachable while that node is up. This brief
generalizes it to a rotating set of devices that needs no single always-on node, and
falls back to the voluntary hosting tiers when the whole set sleeps. Gemot's
author-offline publishing proof covers the community case.

## 3. Applications as modules

Mark's framing matches two existing rulings:

- **Data domains, not applications.** The family composition thesis keeps one data
  domain per mere, open to any of an identity's applications.
- **Contribution kinds.** The GUI catalog's four contribution kinds are presentation,
  pane adapter, pane and servitor. Knot documents are a reader and the Knot editor is
  a surface producer.

"Present it to people who don't have the mod" is progressive enhancement: publish a
portable rendering beside the native data. Knot already does this with site
snapshots.

**On the product split** (Turnstone, Graphshell, Knot, Signalman): splitting is not
the risk; several half-finished shells are. A standalone application earns its place
when someone would use it alone. Knot does. Embedding keeps the same capability
composable. Apple's OpenDoc failed partly for lack of a host people wanted. VS Code
and Obsidian extensions succeeded because the host came first.

## 4. Vocabulary alignment

- **"the mesh"** is ambiguous in the stack. Here it means the owner's **personal
  device mesh** (mesh M1 in the persona plan). The radio mesh household (retinue,
  Tulle, Tucket, Sennet) is one carrier for it, not the same thing.
- **"daemons"** are the **device resident** (the Djinn resident on desktop) and
  Knot's publisher/resident processes. TERMINOLOGY has no entry for *device
  resident*. Sidequest 5 proposes one.
- **"Signalmen"** in the transcript is **Signalman**. "not considered on its own" is
  **Knot**.

## 5. Sidequests (nothing scheduled)

1. **Held-by-N publication:** a receipt where a publish completes only after a second
   paired device holds the signed head, and the publisher can then sleep.
2. **Last one out:** a resident that detects it holds the only reachable copy and
   warns or holds a sleep assertion. macOS first.
3. **Keyless serving:** a resident that serves a site's signed snapshot without
   holding the signing key; readers refuse a forged or stale head.
4. **Phone posture:** a written author-first policy for iOS and Android: what
   replicates, what is served only in the foreground.
5. **TERMINOLOGY:** a *device resident* entry, distinguished from *servitor*.
6. **Reader re-serving policy** for public sites: opt-in, bounded storage, and how it
   interacts with retention leases.
