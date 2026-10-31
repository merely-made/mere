# Low-Power Radio and Managed Network Plan

**Status:** 2026-10-28: cut from `2026-07-24_low_power_managed_network_plan.md`, archived in `archive_docs/2026-10-28/`. V3 to V8 are landed: ingress and Mere acceptance context preserved end to end, the notochord local evaluator, one authorized Murm service over Memory, p2panda, Reticulum/TCP and headed Reticulum/direct-PHY RF, and Retinue transit policy with traffic classes. V1 and V2 are implemented and compile-verified in Retinue with Retinue's 2026-07-29 receipt only partial, so the V0/V2 power and sleep bench is open; V9 (signed offers), V10 (selective replication) and V11 (Tulle feedback, failover, bonding) have not started, so completion conditions 1, 5, 6 and 7 are unmet.

**For:** joining two pieces that are stronger together: a Heltec V4 direct-PHY radio that listens continuously while its ESP32-S3 sits in Light-sleep, and owner-controlled Mere service access and Reticulum transit enforced from honest incoming-session facts. The first receipt is a useful low-power radio; the second is one authorized Murm service crossing that radio; transit, offers, replication efficiency, failover and bonding follow those proofs. The plan lives with Mere because Mere owns product policy and the service boundary; the radio, Reticulum and queue work lands in the Retinue workspace.
Not here: the full-iroh, LAN-direct-noq and radio-device profile assessment (Retinue's `murm/design_docs/2026-09-07_radio_hand_hardware_family_plan.md`), the iroh carrier for the existing signed control exchange (`2026-08-30_wall_node_management_plan.md`) and the canonical device-operation probe across LoRa and IP (`2026-08-25_compact_signed_feed_and_local_control_plan.md`), which are source-reviewed proposals with no MCU build, RF, power or reopen receipt; Murm's consumption of the accepted session and its comms consumers (Murm runtime plan); live audio and voice-note radio carriage (calls and media plan, A5); announce-carried dial hints over LoRa airtime (browser carrier plan, R2); Moot membership and constitutional authority (Gemot). The session-policy spine that took V3 to V7 through a shared facts and admitted-session boundary is complete and archived with the Dramatis repository's plans.

Receipts from this plan's runs go to `archive_docs/<date>/`, never into this
plan (DOC_POLICY §8). Evidence that matters is a test in the repository or a
line in Findings.

## Constraints

- Decisions D1 to D8 below bind every phase: native USB stays the development transport; power policy is explicit; the network policy axes stay independent; transport facts are facts and not claims; Personae stays the authority grammar; the session proof signs the transport context; ordinary Reticulum stays ordinary Reticulum; bonding requires independent capacity. From: `archive_docs/2026-10-28/2026-07-24_low_power_managed_network_plan.md:60-193`
- Ownership: Tulle owns PHY configuration, host-radio transport, power state, airtime, channel and collision information, turnaround and radio observations; Retinue owns Reticulum links, interface ingress, anonymous transit policy, packet classes, queue budgets, forwarding and path use; `mere-transport` reports authenticated peer and ingress facts without deciding policy; notochord evaluates local service policy and runs the bounded session handshake (a session-policy component, not a master policy engine); the delegation and revocation proof primitives are insigne's since the 2026-09-24 move and Personae supplies issuing only; Gemot owns Moot membership and constitutional authority; Murm and other service domains keep their own authorization beyond session admission; the host persists and presents owner settings. From: `archive_docs/2026-10-28/2026-07-24_low_power_managed_network_plan.md:195-217`
- Evidence ladder: every claim names its level (API wired; deterministic unit test; in-memory integration; real TCP or p2panda transport; real OS serial; real RF; measured power or current; headed owner-policy scenario). Later evidence never widens an earlier receipt, and a one-hop bench proof is not a range, recovery, roaming or multi-hop-discovery proof. From: `archive_docs/2026-10-28/2026-07-24_low_power_managed_network_plan.md:830-844`
- Non-goals for this round: Light-sleep while native USB Serial/JTAG is connected; Wi-Fi power management; a universal policy engine for storage, compute or domain operations; a new membership or identity token beside Personae and Gemot; changing ordinary Reticulum wire behaviour; inferring an authenticated peer from an unsigned Reticulum claim; simultaneous protocol personalities on one SX1262; bonding logical lanes over one radio or collision domain; treating advertised willingness as an authorization decision; hiding measurement conditions behind a single current number. From: `archive_docs/2026-10-28/2026-07-24_low_power_managed_network_plan.md:846-857`
- COM numbers are observations, not identities: every headed run re-queries the ports by USB identity before and after flashing, and a committed receipt names board roles and firmware commits. From: `archive_docs/2026-10-28/2026-07-24_low_power_managed_network_plan.md:234-241, 598-629`
- UART wake consumes the triggering character and may lose adjacent characters, so the host sends a wake preamble, waits, then sends the complete command; the firmware ignores the wake byte only at a frame boundary. From: `archive_docs/2026-10-28/2026-07-24_low_power_managed_network_plan.md:295-338`
- Offers describe current willingness and never expose `LocalNetworkPolicy`; they come only after V5 to V8 stabilise the vocabulary. From: `archive_docs/2026-10-28/2026-07-24_low_power_managed_network_plan.md:692-720`
- The verification wall (package-specific Retinue and Mere test commands, formatting and diff checks, and the two Espressif-toolchain builds of the V4 personalities) stands as the archived plan's runbook at `archive_docs/2026-10-28/2026-07-24_low_power_managed_network_plan.md:785-828`; use it per slice and do not copy its commands here.

## Phases

### V0. Reproducible power baseline
Retinue only, no code change. COM7 is the V4 target, COM6 the unchanged V4 control and COM10 the T114 RF peer, all measured through one consistent supply path in five scenarios: current firmware in continuous receive; SX1262 standby; UART firmware with Light-sleep disabled; UART firmware with Light-sleep enabled and the SX1262 receiving continuously; a representative packet workload. Each run records supply path and voltage, board and firmware revision, firmware commit, radio profile, steady and peak current, observation interval and energy over the interval. Raw captures stay under an ignored receipt directory and only the summary is committed.
- Done when: the existing receive-current claim has a repeatable baseline on the actual board and supply path.
- Needs: a bench session with a meter, a known supply and someone to power-cycle the boards; it cannot be driven remotely and is the head of the lane.
- Rulings: D2.
- From: `archive_docs/2026-10-28/2026-07-24_low_power_managed_network_plan.md:263-293, 893-899, 925-929`

### V1. UART direct-PHY without sleep (headed proof)
The firmware feature split (`host-usb` default, `host-uart-low-power`), the UART host I/O generalisation and the wake sequence are implemented and host-tested; Light-sleep stays disabled so transport correctness is isolated from power state. The headed proof is open: the existing 4 KiB Resource publish and fetch acceptance between the UART-controlled V4 and COM10.
- Done when: changing the host transport does not change radio behaviour or the direct-PHY protocol.
- Needs: bench session (V0).
- Rulings: D1, D2.
- From: `archive_docs/2026-10-28/2026-07-24_low_power_managed_network_plan.md:295-343`

### V2. Guarded Light-sleep (headed proofs)
The UART personality's custom idle hook enters Light-sleep only when a small atomic gate permits it; wake sources are DIO1 high level and UART0 activity; the command loop closes the gate before parsing a partial command, configuring the SX1262, SPI activity, transmitting, writing a host event or awaiting a timer. Counters expose sleep entries and wake causes as diagnostics. Headed proof: 100 consecutive UART wake and configure cycles; 1,000 received RF frames with no loss attributable to sleeping; 100 transmissions whose first complete UART command is retained; the existing bidirectional 4 KiB Resource proof; quiet continuous-RX current and representative-workload energy; the default USB proof on COM6 as a regression control. Timer-aware Light-sleep is deferred, not declined, and V2 must not preclude it (D2).
- Done when: quiet continuous RX falls by at least 5x on the same supply path; whole-board continuous RX is at or below 12 mA, or the receipt identifies the remaining consumer; RSSI and SNR remain comparable to the awake control; no wake cycle corrupts a host command; the default USB firmware remains unchanged. This is the first useful milestone.
- Needs: V1 headed proof; V0 baseline; bench session.
- Rulings: D2.
- From: `archive_docs/2026-10-28/2026-07-24_low_power_managed_network_plan.md:345-388, 893-899`

### V7 arm. The sleeping direct-PHY RF link
V7's carrier and policy matrix passes over Memory, p2panda, Reticulum/TCP and headed Reticulum/direct-PHY RF, but the RF arm ran on USB-powered development personalities and is explicitly not power evidence. The last clause of V7's done-condition, one real sleeping direct-PHY RF link, belongs with the sleep bench.
- Done when: the accepted service session, locally denied service, expired delegation, revoked delegation, replayed valid signature on a different link, and valid certificate presented by the wrong session signer all behave as in the V6 matrix over the V2 UART/Light-sleep firmware on a real RF link.
- Needs: V2.
- Rulings: D4, D6, D7.
- From: `archive_docs/2026-10-28/2026-07-24_low_power_managed_network_plan.md:598-629, 1032-1052`

### V9. Signed offers
A signed, expiring `NodeOffer` (accepted profiles, supported protocols, offered roles, capacity bands, transfer paths, expiry) in Mere with a small Retinue discovery adapter. For Reticulum, an independently signed offer digest is appended after the existing peer binding and the full offer is fetched through a bounded control protocol (older parsers already tolerate trailing app data); for p2panda the same object travels over its address-book and control connection. No new identity system.
- Done when: a node discovers an offer, verifies it, observes expiration, and still receives a local denial because the private policy changed.
- Needs: V5 to V8 (landed); the question below on where V9 to V11 live.
- Rulings: D3, D5.
- From: `archive_docs/2026-10-28/2026-07-24_low_power_managed_network_plan.md:692-720, 780-783`

### V10. Selective replication
Built on `DropExportSelector` and `DropExportBudget`: exchange retained frontiers and content inventories; request one missing operation or blob from one peer; prefer an available nearby cache; skip known inventory on later sessions; apply privacy selection before scheduling; resume blob chunks by content hash; permit store-and-forward with expiration.
- Done when: repeating an unchanged sync transfers only the compact inventory exchange, and partial sync obeys both semantic and byte budgets.
- Needs: V9; the Murm runtime plan's drop machinery.
- Rulings: D3.
- From: `archive_docs/2026-10-28/2026-07-24_low_power_managed_network_plan.md:722-737, 780-783`

### V11. Tulle feedback, failover and bonding
After V2 establishes the power behaviour, a transport-neutral Tulle `InterfaceObservation` (channel, collision group, queue depth, estimated and actual airtime, turnaround, power state, wake reason, last failure) that Retinue consumes for path selection and diagnostics without owning PHY policy. Then `single` (ordinary Reticulum behaviour), `failover` (a complete transfer surviving lane loss), and `bonded` only for two independently measured bearers. Bonded acceptance needs byte-exact reassembly under loss and reordering, duplicate suppression, graceful loss of either lane, per-lane airtime budgets obeyed, and aggregate goodput approaching the sum of isolated-lane measurements. Retinue's singular path table stays intact until then.
- Done when: the evidence distinguishes useful independent capacity from two logical paths sharing one bottleneck.
- Needs: V2; two independent bearers for bonding.
- Rulings: D8.
- From: `archive_docs/2026-10-28/2026-07-24_low_power_managed_network_plan.md:739-783, 189-193`

### Z. Round completion
The round is complete when: (1) the V4 has a measured low-power continuous-RX personality with preserved direct-PHY behaviour; (2) every incoming Mere transport session carries honest protocol, peer and ingress context; (3) one owner-configured Murm service is admitted or rejected through the same evaluator over Memory, p2panda, Reticulum/TCP and direct PHY; (4) Retinue transit cannot starve locally reserved traffic; (5) offers reveal current capabilities without revealing private policy; (6) repeated unchanged replication sends inventory rather than payload again; (7) failover and bonding claims are limited to the independent-bearer evidence actually obtained. Conditions 2 and 3 are met; 1, 5, 6 and 7 are unmet; 4 relates to V8.
- Done when: all seven hold.
- Needs: V0, V1, V2, V9, V10, V11.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-07-24_low_power_managed_network_plan.md:1118-1133`

## Open questions

- Does Retinue's 2026-07-29 receipt count toward V2? Record it as V2's first partial headed proof, or keep it as Retinue-side evidence and leave V2 wholly open until the bench. From: `archive_docs/2026-10-28/2026-07-24_low_power_managed_network_plan.md:939-942`
- Where should V9, V10 and V11 live, given that no `NodeOffer`, `InterfaceObservation`, failover or bonding code exists in Mere or Retinue? Keep them here as this round's open phases, extract them to their own plan, or retire them. The 2026-10-28 cut holds them here and does not answer the question. From: `archive_docs/2026-10-28/2026-07-24_low_power_managed_network_plan.md:780-783`
- Completion condition 4 relates to V8, which landed; is the sustained public-transit starvation test (a sustained transit load cannot consume the configured local interactive reservation, and disabling transit leaves local service traffic unaffected) stated as met anywhere? From: `archive_docs/2026-10-28/2026-07-24_low_power_managed_network_plan.md:631-690, 1118-1133`

## Decisions

### D1. Native USB remains the development transport

The Heltec V4 uses the ESP32-S3 USB Serial/JTAG peripheral. In Light-sleep its
clocks stop, it cannot answer the host, and the host may fail to re-enumerate it
after wake. The existing USB direct-PHY firmware therefore remains the default
development and recovery personality.

The low-power personality uses UART0 on the exposed header:

- GPIO44: UART0 RX
- GPIO43: UART0 TX
- GPIO14: SX1262 DIO1 wake

The UART adapter carries RX, TX, and ground. It does not power the board during
current measurements.

Primary hardware references:

- [ESP32-S3 USB Serial/JTAG sleep behavior](https://docs.espressif.com/projects/esp-idf/en/stable/esp32s3/api-guides/usb-serial-jtag-console.html)
- [Heltec WiFi LoRa 32 V4 datasheet](https://resource.heltec.cn/download/WiFi_LoRa_32_V4/datasheet/WiFi_LoRa_32_V4.2.0.pdf)

### D2. Power policy is explicit

The V4 firmware already waits asynchronously for USB input or SX1262 DIO1. Its
runtime idles with `WAITI`; the missing step is genuine Light-sleep.

Light-sleep is permitted only while:

- continuous receive is armed;
- the host-command parser is between commands;
- host output is drained;
- SPI and SX1262 setup are inactive; and
- the pending work consists only of UART RX or DIO1.

The first implementation does not sleep across an Embassy timer deadline. The
ESP32-S3 time source used by the current runtime does not advance in
Light-sleep, so timer-aware sleep needs a separate clock-compensation design.

A later sensor-node personality (wake on a timer, read an attached sensor,
emit one signed reading, sleep) would make timer-aware sleep load-bearing
rather than optional. The clock-compensation design is therefore deferred,
not declined, and V2 should avoid choices that preclude it.

### D3. Network policy axes remain independent

Discovery, incoming services, transit, replication, compute, path selection,
and capacity commitments are separate owner settings. There is no global
public/private or client/router mode.

A node may:

- publish a service without carrying transit;
- carry transit without exposing owner data;
- accept one community profile while refusing another;
- share storage while refusing compute; or
- use Reticulum reachability without accepting Mere protocols.

### D4. Transport facts are facts, not claims

Mere transport acceptance reports the peer only when the transport
authenticated it.

```rust
pub struct AcceptedSession<S> {
    pub stream: S,
    pub protocol: Alpn,
    pub peer: Option<PeerID>,
    pub ingress: IngressContext,
}
```

`peer: Some` means transport-authenticated. A subject named by application
bytes is not placed there.

p2panda can report `Connection::remote_id()`. Reticulum best-effort acceptance
cannot identify its initiator yet; its session proof supplies the application
identity later.

### D5. Personae remains the authority grammar

`personae::delegation` already supplies:

- signed capability certificates;
- domain, resource, path, and action scopes;
- attenuation;
- validity intervals;
- bounded delegation depth; and
- signed revocation statements.

The new local notochord crate validates chains against configured roots
and keeps its own revocation ledger. It does not create another membership
token. Gemot remains responsible for Moot membership and constitutional
authority.

**Corrected 2026-10-06 (S14 pass):** the grammar has since split. `5364dfa0`
(2026-09-24) moved the delegation statements and their checks from personae to
insigne; notochord depends on insigne with its `verify` feature
(`repos/dramatis/crates/notochord/Cargo.toml`), and personae keeps issuing.

### D6. The session proof signs the transport context

The handshake signs a canonical transcript containing:

- wire version;
- network and profile;
- protocol;
- requested action and traffic class;
- nonce;
- subject;
- authenticated transport peer, when present;
- ingress interface and Reticulum link, when present; and
- hashes of the supplied delegation certificates.

For p2panda, the signed subject must equal the transport-authenticated
`PeerID`. For Reticulum, the proof binds the subject to the current link and
nonce.

### D7. Ordinary Reticulum remains ordinary Reticulum

The policy handshake is an application protocol selected by a service owner.
It does not alter ordinary Reticulum routing or require unrelated RNS peers to
understand Mere.

Anonymous Reticulum transit is enforceable by interface, packet type,
destination, hop count, and budgets. Persona policy applies only after a Mere
session proves a subject.

### D8. Bonding requires independent capacity

Logical paths over one SX1262, frequency, or collision domain are not
independent bearers. Retinue's singular path table remains intact until two
independent bearers demonstrate aggregate goodput and graceful lane loss.

## Findings

- 2026-07-29: Retinue's `murm/design_docs/2026-07-29_v4_light_sleep_rf_wake_acceptance.md` is partially accepted on hardware: Light-sleep resumes without reset, the SX1262 stays usable across repeated timer wakes, and one direct RF wake was proved end to end. Repeated RF-triggered sleep continuity, current and energy remain unmeasured. From: `archive_docs/2026-10-28/2026-07-24_low_power_managed_network_plan.md:912-947`
- 2026-07-27: the headed direct-PHY run found three carrier defects now closed (a 263-byte authenticated announce over the PHY cap, a best-effort relay emitting a packet over the negotiated 255-byte MTU, best-effort delivery for a multi-frame policy handshake on LoRa); Reticulum used a 255-byte reliable link with a one-frame window. From: `archive_docs/2026-10-28/2026-07-24_low_power_managed_network_plan.md:1032-1052`
- 2026-10-10: Mere resolves iroh 1.3.0 and the exact `mere-p2panda-net` 0.7.5 fork; `crates/stickleback/src/lib.rs` owns joined-space receive processing, policy-before-insert storage and domain-supplied authorization. Forwarding retains the original signed bytes and operation id, and gateway re-origination is a separate authorship event with explicit provenance; carrier authentication, session admission, device authority and operation admission stay distinct. From: `archive_docs/2026-10-28/2026-07-24_low_power_managed_network_plan.md:1100-1116`

## Progress

- 2026-10-28: cut from `2026-07-24_low_power_managed_network_plan.md`; nothing new landed.
