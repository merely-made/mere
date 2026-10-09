# Device Pairing by Key Plan

**Date**: 2026-10-02
**Status (2026-10-06)**: in progress, paused before D2. Rulings 1 to 74.
- D1 landed (`4963b489`).
- D1b's mere fix (M1) landed (`177b927c`); its fork fix (F1) shipped in the
  0.7.5 repin, pushed 2026-10-04 (fork `1bec457e`, Knot `92367ec`, mere
  `031b3dcc`).
- `connected` follows the gossip overlay (ruling 31, `fdb02bd3`) and, off
  it, open connections (rulings 47 to 56, `005e27ad`).
- The overlay's gap after restarts has a ruled fix in iroh-gossip, held
  until its next release (rulings 64 to 72).

The pause of ruling 73 (for chatelaine P4a) has lapsed: P4a landed as
`007fbe7c`. The identity work then moved to the vault lock plan. D2 has
not been put back to Mark, and its restart waits on his word (see §7,
2026-10-06).
**Scope**: Mark's machines find, reach and trust each other by device
identity, not by address: the stack's own peers already do on one network;
SSH, the path Mark uses daily, does not. Pairing a device becomes one
ceremony. A self-hosted relay covers the second network.

**Related**:

- [reachability rungs and privacy lanes plan](2026-08-03_reachability_rungs_and_privacy_lanes_plan.md):
  the shape (Syncthing's: identity is the key, addresses are disposable
  hints), R0 local mDNS and R1 cached dial hints, both landed. This plan is
  its SSH and pairing rung.
- [SSH CA projection plan](../../dramatis_docs/implementation_strategy/2026-08-12_ssh_ca_projection_plan.md):
  personae's SSH certificate authority; its host-certificate half was never
  built (§4).
- [djinn family resident services plan](2026-08-22_djinn_family_resident_services_plan.md):
  the resident that runs the agent, personal sync and Knot, and would run the
  directory and the SSH-by-key service.
- [device-grant delegation reconciliation](../technical_architecture/2026-08-11_device_grant_delegation_reconciliation.md):
  the delegation grammar that ruling 6's CA approval rides.

---

## 1. Why

Mark's machines take DHCP addresses on a `/22` mesh Wi-Fi, and the leases
move. On 2026-10-01 the ThinkPad had moved from `.28` to `.32` (and `.28` had
become an iOS device), Q-PC from `.44` to `.68`, and finding them took a
sweep of the `/22` for port 22 and a hand comparison of host-key
fingerprints against `known_hosts`. The ThinkPad does not advertise over
mDNS at all: avahi runs with no service files, and its own name did not
resolve. One iMac sometimes sits only on a separate wired network with no
route to the mesh. Mark: "these are not static ip addresses. Is there a
coordination strategy we can employ to pair devices easily?"

## 2. What exists (assessed 2026-10-01)

From a read-only research lane, with the load-bearing claims re-checked in
code:

- **Four unrelated device identities.** personae's wallet `DeviceRoster`
  (`crates/dramatis/personae/src/carry/mod.rs:350`, device id, Ed25519
  public key, label, mode, exposure, no address field); the personal-graph
  node id that djinn's `PairedDevice` keys on
  (`ports/djinn/src/settings.rs:468`), derived per graph; Knot's own
  `PairedWriter` key; and SSH's device id, a hash of the hostname
  (`crates/dramatis/personae/src/enroll.rs`).
- **R0 and R1 work for the stack's own peers only.** djinn refreshes each
  paired device's `last_endpoint` hint every 5 s while connected
  (`settings.rs:511`, `:673-688`). The live copy is in-process and the
  saved copy is an iroh ticket in a settings file: nothing outside the
  resident can ask "where is device X now?". `relay_urls` default empty
  (`settings.rs:462`), so off-LAN reach is unconfigured.
- **SSH uses none of it.** No code generates ssh config, a `ProxyCommand` or
  a `KnownHostsCommand`.
- **Host certificates were never built.** `SshCertAuthority::mint_host_cert`
  (`ssh_ca.rs:276`) has no caller outside tests (`ssh_ca.rs:646`,
  `tests/ssh_ca_live.rs:146`); `personae-vault enroll-host` writes one
  `cert-authority` line into the target's `authorized_keys` and prints
  "host-key prompts still apply" (`bin/personae-vault/certs.rs:185`).
- **The CA is per vault.** It derives from the profile master
  (`certs.rs:34`), and a fresh vault generates its master at random
  (`bootstrap.rs:135`); there is no master export or import.
- **The machines today** (read 2026-10-02 from each running agent's public
  identities with `ssh-add -L`, no vault unlocked):

  | Machine | personae agent | Offers a certificate | CA in use |
  |---|---|---|---|
  | Windows laptop | running | yes | `SHA256:bKHWIGTTJcAq9kdCdbiY0b/SpP/kkfp54REdZrpd/W8` |
  | M4 iMac | running | no: the bare key `mark-ik@imac-2026-08`, an agent older than certificates | none |
  | ThinkPad | not installed | — | none |
  | Q-PC | not installed | — | none |

  *2026-10-04:* Mark named a fifth machine, "There's also a surface laptop 3
  running kubuntu"; "Not a powerful device (8gb ram, 256gb ssd), but another
  linux". Its agent, address and SSH state have not been read.

- **Pairing today** takes six steps and two 128-hex copy-pastes for personal
  sync (`djinn --pairing-facts`, `--pair-node`), a separate Knot
  `--pair-writer`, and a third SSH ceremony that needs a working login and a
  TOFU prompt before `enroll-host`. pandect's wallet-grant pairing (ticket,
  typed code, 6-digit SAS, `crates/system/pandect/src/wallet_grant/pairing.rs`)
  has no caller.
- **DNS-SD (djinn F3) is unbuilt** and would not help SSH: service records
  are link-local claims, not identity.

## 3. Rulings

Mark's answers, from multiple-choice rounds; each is the option label quoted
verbatim unless marked as his free text.

**Ruling 1.** *Take up "pair devices by key, not address" now?* Options:
assess it as its own objective; host certificates only; a router stopgap now;
note it and move on. Mark: **"Assess it as its own objective
(Recommended)"**. Follows: this plan.

**Ruling 2.** *Which identity does "device X" resolve to?* Options: the
personal-graph node id; the wallet `DevicePublicKey`; a new device-presence
key. Mark: **"Personal-graph node id (Recommended)"**. Follows: the key
djinn already refreshes hints for is the one SSH dials.

**Ruling 3.** *How does SSH reach a device by key?* Options: a `ProxyCommand`
via the local resident; generated ssh config from cached addresses; stable
names only; trial iroh-ssh first. Mark: **"ProxyCommand via the local
resident (Recommended)"**. Follows: phase D3.

**Ruling 4.** *Which relay reaches the other network?* Options: a
self-hosted iroh-relay; n0's public relays; leave the other network out.
Mark: **"Self-hosted iroh-relay (Recommended)"**. Follows: phase D6; where
it runs is open.

**Ruling 5.** *How are host-key prompts retired?* Options: stable names now,
pinning next, certificates later; finish host certificates now; pinning now.
Mark: **"Finish host certificates now"**. Follows: phase D4 is not deferred.

**Ruling 6.** *Which CA shape?* Options: trust every machine's CA; one
designated CA machine; one shared master. Mark (free text): **"how about
each machine has it's own ca, and you can use your master to determine
which ones can sign host certificates (trusted)."** Follows: per-machine
CAs, approved by one master. *Reading, not ruled*: OpenSSH cannot chain
certificate authorities, so the master's approval is an insigne
`SignedDelegationCertificate` per machine CA ("may sign host
certificates"), and a client-side step checks each approval (and its
revocation) and writes one `@cert-authority` line per approved CA.

**Ruling 7.** *How is each machine's CA read as evidence?* Options: Mark runs
`personae-vault ca` on each; I run it over SSH. Mark: **"I run it over
SSH"**. Follows: §2's table. *Reading, not ruled*: `personae-vault ca` opens
the vault, which on macOS and Linux needs the passphrase (no OS unlock
backend there), and taking a passphrase from a keychain is not done; the
CAs were read instead from each running agent's certificate with `ssh-add
-L`, which unlocks nothing.

**Ruling 8.** *What should pairing a new device be?* Options: one ticket and
SAS recording everything; one ceremony for sync only; keep the separate
ceremonies. Mark: **"One ticket + SAS, recording everything
(Recommended)"**. Follows: phase D5.

**Ruling 9.** *How does a resident run on the ThinkPad and the iMacs?*
Options: extend djinn with a systemd unit and a launchd plist; a lighter
presence daemon. Mark: **"Extend djinn: systemd unit + launchd plist
(Recommended)"**. Follows: phase D2.

**Ruling 10.** *Whose master is the root that approves machine CAs?*
Options: the Windows laptop's vault master; a new dedicated root; the M4's
vault master. Mark: **"The Windows laptop's vault master
(Recommended)"**.

**Ruling 11.** *Where does "where is device X now?" live?* Options: a djinn
app-door route and CLI; reading the settings file directly. Mark: **"A
djinn app-door route + CLI (Recommended)"**. Follows: phase D1, on the
`djinn-site` pattern.

**Ruling 12.** *Is the Windows laptop an SSH target in scope?* Options: not
now; yes. Mark: **"Yes"**. Follows: it needs Windows' OpenSSH Server
feature (an administrator step), and D3 and D4 include it.

**Ruling 13.** *Where does the plan live?* Options: a new plan in
`mere_docs`; extend the reachability plan; `dramatis_docs`. Mark: **"A new
plan in mere_docs (Recommended)"**. Follows: this file, linked from the
reachability and SSH CA plans.

**Ruling 14.** *No machine runs djinn yet, so D1's directory has no paired
peers to list on real machines. How does D1 proceed?* Options: build D1 now
and prove it locally; do D2 first. Mark: **"Build D1 now, prove it locally
(Recommended)"**. Follows: D1 is proven with two djinn instances on one
machine; its real-machine receipt moves after D2.

**Ruling 15.** *On Windows, moving to djinn retires the legacy
`graphshell-device-host`, which serves Mark's SSH agent today. How?*
Options: side by side, then Mark switches; I switch it in D2; leave Windows
on the legacy host. Mark: **"Side by side, then you switch
(Recommended)"**. Follows: D2 tests djinn on its own pipe beside the legacy
host until its agent signs a real SSH login; then Mark runs djinn's
installer, which retires the legacy task; rolling back re-enables that
task.

**Ruling 16.** *Two residents on one machine never made first contact by
mDNS alone (each learned the other's address and neither dialled; a ticket
connected them in 3.45 s), while in August mDNS-only first contact worked
across two physical machines. What is first contact?* Options: the pairing
ceremony's ticket, measuring mDNS in D2; fix mDNS-only first contact before
D2. Mark: **"Fix mDNS-only first contact before D2"**. Follows: phase D1b.

*Amended 2026-10-02*: the question's premise was wrong. August's
cross-machine receipts never exercised this path: the Fedora and Windows ones
dialled a known peer id explicitly in a retry loop (`g5_peer --discover`,
`ports/graphshell/src/bin/g5_peer/connect.rs:167-176`), and Knot's K2 and
personal sync's first contact carried a ticket. Devices that have never met
have always needed a ticket on this path, so D1b is a fix, not a regression,
and the ruling's direction stands.

**Ruling 17.** *Keep D1's names, route `device-directory-v1` and CLI
`djinn-devices`?* Options: keep them; `paired-devices-v1` and `djinn-peers`.
Mark: **"Keep device-directory-v1 / djinn-devices (Recommended)"**.

**Ruling 18.** *Which app label may read the directory?* Options: a new
`djinn` label only; also grant turnstone; reuse `knot-editor` as djinn-site
does. Mark: **"A new `djinn` label only (Recommended)"**.

**Ruling 19.** *Where does the first-contact fix go?* Options: F1, in the
p2panda fork (refresh topic watchers when a node's record is written); M1, in
mere-transport (give a paired peer a bare record before tagging it); both.
Mark: **"Both"**. Follows: M1 lands now, F1 with the fork release of ruling
20; M1's side effect (a paired device not yet seen appears with
`reachable = false`) is accepted with it.

**Ruling 20.** *If the fork is patched, what does the new release build on?*
Options: the pinned tag `0a54ab82`, released as `mere-p2panda-net-0.7.5`; the
fork's current main; upstream p2panda's main. Mark: **"Upstream p2panda's
main"**. Follows: upstream's 54 commits since the fork's last merge
(2026-09-10 to 2026-09-30: iroh 1.0.3 to 1.3.0, authorisers renamed to
allow and block lists, a new `SyncHook`, stream orderer changes; 70 files,
13 in p2panda-net) are merged into the fork first, F1 on top. Tagging and
pushing the fork stay Mark's.

**Ruling 21.** *Offer the fix to p2panda upstream?* Options: I draft and Mark
files; not now. Mark: **"Not now"**.

**Ruling 22.** *`connected` can read false on a working link when both sides
dial at once (the duplicate connection closes and iroh marks the shared
address inactive for about 5 s while gossip keeps delivering); fix it?*
Options: count a gossip neighbour as connected; debounce; leave it and make
the test dial one way. Mark: **"Count a gossip neighbour as connected
(Recommended)"**.

**Ruling 23.** *The fork merge compiles everywhere except stickleback, where
upstream renamed `StreamItem` to `LogEntry`; how does the fork lane finish
its check?* Options: a scratch-only rename; a `StreamItem` alias in the fork;
leave it to the repin. Mark: **"Scratch-only rename (Recommended)"**.

**Ruling 24.** *Upstream made `p2panda_core::cbor::decode_cbor` lenient
(`decode_cbor_strict` keeps the old behaviour); mere calls it in about 40
files.* Options: accept and audit strict sites; keep strict in the fork;
accept with no audit. Mark: **"Accept, and audit strict sites
(Recommended)"**. Follows: the repin moves every call that feeds a hash,
signature, content address or wire validation to `decode_cbor_strict`.

**Ruling 25.** *mere's per-interface mDNS fork (H10's fix for multi-homed
Windows hosts) has been an unused patch; what now?* Options: bring the fork
to 0.6.0 with the repin; check upstream first; leave it. Mark: **"Bring the
fork to 0.6.0, with the repin (Recommended)"**. *Superseded by ruling 26 on
new evidence.*

**Ruling 26.** *The per-interface mDNS port (upstream PR #7, now on 0.6.0)
works as intended but fails upstream's own `mdns_subscribe` test on this
laptop every time, the failure it fixes is absent here today, and first
contact was about 0.6 s slower; ruling 25 adopted it with the repin. Now?*
Options: keep it out and diagnose first; adopt it as ruled; stock, and drop
the fork. Mark: **"Keep it out; diagnose first (Recommended)"**.

**Ruling 27.** *The dead `=0.4.0` patch row reads like an active fix;
meanwhile?* Options: drop it with the repin; leave it. Mark: **"Drop it with
the repin (Recommended)"**.

**Ruling 28.** *Tag `mere-p2panda-net-0.7.5`, push it to `mark-ik/p2panda`,
then repin knot first and mere second, with the stickleback rename and the
strict CBOR sites?* Options: go (tag, push, repin, with mere's and knot's
pushes coming back to Mark); tag locally only; wait for the `connected`
fix. Mark: **"Go: tag, push, repin (Recommended)"**.

**Ruling 29.** *Four strict sites decode plaintext that encryption already
authenticated; strict anyway?* Options: strict anyway; lenient for those
four. Mark: **"Strict anyway (Recommended)"**.

**Ruling 30.** *With gossip neighbours counted as connected, the old
transport test still fails at its assertion that iroh shows an active direct
path, because iroh drops that path state during simultaneous dials (its
abandon check looks only at the closing connection's paths, unchanged in
iroh 1.3.0). What should the test assert?* Options: dial one way in that
test; patch iroh's abandon check; relax the assertion. Mark: **"Dial one way
in that test (Recommended)"**.

**Ruling 31.** *Under the OR rule a closed peer reads connected for about
60 s, because iroh keeps its path active after gossip drops the neighbour in
about 0.1 s. Should `connected` fall when the neighbour goes down?* Options:
gossip decides while the overlay is up; keep the OR rule. Mark: **"Gossip
decides while the overlay is up (Recommended)"**. Follows: for a peer on the
overlay, the gossip neighbour state is authoritative; iroh's path counts
only for peers not on it. Amends ruling 22.

**Ruling 32.** *Which meaning of "on the overlay" for ruling 31?* Options:
subscribed (this node has subscribed to the topic, read exactly from the
address book's self record); joined (from gossip's `Joined` to `Left`, not
readable through the fork's API); per peer (gossip-authoritative once the
peer has been a neighbour since this node joined). Mark: **"Subscribed
(Recommended)"**. Follows: before the first neighbour comes up, every peer
reads not connected; a device reachable only over a later SSH channel would
too.

**Ruling 33.** *GitHub's fork `main` had moved to `94947fd1` (Mark's
2026-09-28 sync merge of upstream, adding no files beyond the release's own
tree), so the release would not fast-forward. How should it sit?* Options:
rebuild the release on top; merge on top of the release; push only the tag.
Mark: **"Rebuild the release on top (Recommended)"**.

**Ruling 34.** *Signalman's own workspace resolves `p2panda-core` from
crates.io 0.7.1, where the audited CBOR split cannot compile. Which?*
Options: patch it onto the fork tag; leave it on crates.io 0.7.1. Mark:
**"Patch it onto the fork tag (Recommended)"**. Follows: a `[patch.crates-io]`
row in `ports/signalman`; its graph drops the dalek-2 family (287 to 278
packages, measured). A desktop workspace outside mere that patches retinue
will need the same row. *Superseded by ruling 35.*

**Ruling 35.** *Mark asked "why not just align signalman with the rest of
the stack?" Make it a member of mere's workspace?* The evidence: mere's root
excluded it because "it consumes the neighboring Retinue checkout", but its
Retinue dependencies are now git pins (rev `6af5c0ff`), and Retinue at that
rev names no mere package, so joining creates no second lockstep. Options:
join mere's workspace; keep ruling 34's patch row. Mark: **"Join mere's
workspace (Recommended)"**. Follows: signalman drops its empty `[workspace]`
table and its exclude entry, inherits the root patch table and lock, and is
covered by the portable gate and every compiler census (insigne's phase A
missed it as a nested workspace); mere's lock gains Retinue's crates at that
rev. *Amended by ruling 42:* the root already pinned `retinue` at a newer
rev, which this evidence missed.

**Ruling 36.** *After a crash-restart the restarted resident gets no gossip
neighbour (12 of 12 runs, the same on `main` and on ruling 31's branch; §6):
the surviving side's iroh-gossip can hold a stale pending neighbour request
and never answers the restarted side's `Join`, whichever restart comes
second. Data from the restarted side reaches the survivor 17 to 113 s late,
through LogSync retries; `main`'s directory reads connected meanwhile, ruling
31's reads not connected. What now?* Options: keep ruling 31, merge it now,
and fix the overlay in its own lane; keep ruling 31 and hold it until the
fix; loosen D1b's check; revisit ruling 31. Mark: **"Keep 31, merge now, fix
lane (Recommended)"**. Follows: ruling 31 merges onto `main` with D1b's
receipt failing at its second restart, recorded as expected until the
overlay fix; that lane assesses where the fix lives (an iroh-gossip patch,
the p2panda fork, or a rejoin in mere-transport) and brings it to Mark.

**Ruling 37.** *The simultaneous-dial control passes 8 of 8 run alone but
failed 1 of 10 parallel suites (no trip in 16 pairs), and suites take 11 to
202 s, mostly in it. How should it run?* Options: ignore it in the default
suite and run it alone; raise the pair budget; leave it. Mark: **"Raise the
pair budget"**. Follows: it stays in the default suite. *Reading, not ruled:*
the new budget is set from trips measured in parallel suites, recorded with
the change.

**Ruling 38.** *Beyond ruling 30's wording, the one-way tests now have the
receiver subscribe before the dialler joins: iroh-gossip drops a `Join` for a
topic the receiver has not subscribed to (`proto/state.rs:247-275`) and
nothing retries it. Keep that order?* Options: keep receiver-first; revert.
Mark: **"Keep Bob-first (Recommended)"**.

**Ruling 39.** *A peer on the overlay with no gossip neighbour, while iroh
still marks a path active, reads "not connected (a path is still marked
active)" on the resident card and in `djinn-devices`. Keep that wording?*
Options: keep it; plain "not connected"; name the overlay ("something like
not connected (no gossip neighbour; a path is still active)"). Mark: **"Name
the overlay"**. Follows: that wording, on both.

**Ruling 40.** *On the release stack (iroh 1.3.0), D1's stopped-peer control
fails at its 120 s limit: with 600 s, a killed peer read connected for
199.5 s, against 73.5 s on iroh 1.2, `main`'s `connected` being iroh's path
alone. How should the pushes proceed?* Options: land ruling 31 first, rerun
on the merged stack, then push knot, mere and knot's repin back to back;
raise D1's patience and push now; look into iroh 1.3 first. Mark: **"Look
into iroh 1.3 first"**. Follows: no knot or mere push until it is known why
iroh 1.3.0 holds a dead path longer; peers that are not subscribed still go
by iroh's path under ruling 31. *2026-10-03, relayed by another session (the
physics session), which asked Mark whether this hold covered its own
unrelated mere push:* Mark: **"Push, then tell pairing"**. *That session's
reading, not ruled:* the hold covers only the 0.7.5 repin's pushes.

**Ruling 41.** *Knot's `main` moved 8 commits (Collapse, a new
`knot-composition`, mere `c6707958`, genet `b1eb3af1`) and merges into the
release branch without conflict. Which knot commit does mere pin?* Options:
`562353aa`, the verified repin (Knot `3dfb70b`, 30 commits past mere's
current pin `855cb75d`, plus the p2panda move); Knot's merge commit. Mark:
**"562353aa (Recommended)"**. Follows: Knot's eight newer commits reach mere
at its next ordinary knot repin. Step (c) moves Knot's genet to mere's
`bd3e8861`, which descends from Knot's `b1eb3af1` (checked).

**Ruling 42.** *With signalman joined, mere's lock holds two copies of
retinue 0.1.1: signalman's four Retinue crates at `6af5c0ff` and the root's
`retinue` at `85e716c7`, 61 commits newer (mere-transport's optional mesh
feature). At `85e716c7` the four crates exist at the same versions and none
names a mere package. Which?* Options: move signalman to `85e716c7`, with
`retinue` through `workspace = true`; keep both. Mark: **"Move signalman to
85e716c7 (Recommended)"**. Follows: one Retinue in mere; any code change the
61 commits need comes back as a fork.

**Ruling 43.** *Mark's local `crates/p2panda` was clean on `main` at
`0a54ab82`, behind GitHub's `1bec457e`. Bring it up to date?* Options:
fast-forward it; leave it. Mark: **"Fast-forward it (Recommended)"**. Done
2026-10-03 with `--ff-only`; clean afterwards.

**Ruling 44.** *Ruling 42's target was gone: another session's merge on
`main` (`f62581c7`) moved the root's `retinue` and signalman's four Retinue
rows to 0.2.0 at `fa4f925`, so mere already had one Retinue, while signalman
still named that rev in its own rows and each Retinue repin edits two
manifests. When signalman joins the workspace (ruling 35), how does it
follow the root?* Options: through `workspace = true`, with outrider,
postilion and radio-hand in the root's table; keep signalman's own rows.
Mark: **"Through workspace = true (Recommended)"**. Follows: amends ruling
42, whose `85e716c7` target is superseded by the root's `fa4f925`. The
release branch's `b7913620`, made to ruling 42 before `main` moved, is
superseded by a forward change when `main` merges in, not dropped.

**Ruling 45.** *Ruling 40's investigation found iroh 1.3 no slower than 1.2:
a killed peer reads connected for 71 to 90 s on a quiet machine on every
stack, and 100 to 312 s under build load on both, the 199.5 s being one such
run. The cause is the same in both: iroh keeps a closed connection's paths
Active until the peer's remote actor idles out, 60 s after its last queued
message, and mere's own `remote_info` polls (406 in 70 s) keep resetting
that timer. Ruling 31 sidesteps it for overlay peers: on that day's merge,
D1's stopped peer read not connected after 10.45 s. How do the pushes
proceed?* Options: accept 1.3 and push after D1 passes on the repin with
ruling 31 merged; patch iroh too; raise D1's patience. Mark: **"Accept 1.3;
push after D1 (Recommended)"**. Follows: the release branch merges `main`
(with ruling 31) and reruns D1, D1b and the gate; knot, mere and knot's
repin then go back to back, each with Mark's OK. Ruling 47 adds a lane
before them.

**Ruling 46.** *Should iroh hear about the `remote_info` finding? (Ruling 21
kept p2panda changes from upstream for now; iroh had not come up.)* Options:
draft an issue for Mark's review; not now. Mark: **"Hmm. Burn, p2panda, and
iroh are key dependencies developing hard. They may appreciate a heads up in
the form of an issue. But they likely get a lot, and I would hate to be part
of overwhelming them. That’s a big problem for open source developers today:
tons and tons of good faith issues and prs that strike like a cholesterol
glut in a project’s heart. So that’s part of my reluctance. Plus every issue
is so dry and technical… i would just not write them like that unless I knew
they were using LLMs to analyze the issue anyway (like prns). Keep note of
these issues for a review later. If a (pre)release of any of the three goes
by without the issue being addressed, it is then worth bringing up in a
chill way that does not demand full mental bandwidth from stressed
developers (i.e. describing the issue and why it matters in plain english,
with a few key technical points as necessary)."** Follows: the [upstream
candidates ledger](../research/2026-10-03_upstream_candidates_ledger.md)
holds the items (eight at opening). Each is checked again when its project's
release lands in a repin, and goes to Mark as a plain-English note only
after a release passes it by; nothing is posted without his read.

**Ruling 47.** *Peers not on a gossip overlay still read iroh's path under
ruling 31, so a killed one reads connected for 60 s or more, extended by our
own polls; D3 (SSH by key) is the first such case, dialling a device on its
own ALPN. A mere-side fix would track liveness from iroh's connection events
instead of path usage. When?* Options: decide at D3; fold into the overlay
lane; build it now (a lane for it before the pushes). Mark: **"Build it
now"**. Follows: a lane assesses and builds connection-event liveness for
peers off the overlay before knot's and mere's pushes, its design forks
coming to Mark.

**Ruling 48.** *Ruling 47's signal: an iroh endpoint hook (`after_handshake`)
sees every connection on the endpoint, whoever opens it, and its weak
handle's `closed()` fires when that connection ends; in a probe both
connections to a killed peer closed 12.0 to 12.2 s after the kill, while the
path rule cleared at 72 s. Off the overlay a peer would read connected while
at least one connection to it is open. Count which connections?* Options:
every open connection; exclude short-lived ones. Mark: **"Every open
connection (Recommended)"**. Follows: a brief fetch, or a handshake later
rejected, counts while open; an idle peer with no open connection reads not
connected.

**Ruling 49.** *Should an open connection ever make a peer on a subscribed
overlay read connected (an SSH session to a device with no gossip neighbour,
as in ruling 36's half-state)?* Options: gossip alone, as ruled; gossip or an
open connection. Mark: **"Gossip alone, as ruled (Recommended)"**. Follows:
ruling 31 stands; the count serves only peers off the overlay.

**Ruling 50.** *When a peer reads not connected while iroh still shows an
active path, the card says "not connected (no gossip neighbour; a path is
still active)" (ruling 39); off the overlay the reason is no open
connection. Wording?* Options: two wordings, off the overlay "not connected
(no open connection; a path is still active)"; one wording, "not connected
(a path is still active)". Mark: **"Two wordings (Recommended)"**.

**Ruling 51.** *Where does the hook go, and where does the build land? The
API is identical in iroh 1.2 and 1.3 and in p2panda 0.7.4 and 0.7.5
(checked), and hooks stack beside p2panda's own authoriser hook.* Options:
always on, landing on `main`; always on, landing on the release branch; opt
in through the builder, landing on `main`. Mark: **"Always on; land on main
(Recommended)"**. Follows: the hook is installed in
`P2pandaTransport::bind_inner`; the build lands on `main`, and the release
branch merges `main` again and reruns its checks on iroh 1.3 before the
pushes. *2026-10-03 correction:* the question's "beside p2panda's own
authoriser hook" was wrong. p2panda passes only the hooks its caller gives
it (`iroh_endpoint/actors/endpoint.rs:214`); `ConnectionBlockList`
(`authoriser.rs:76`) is a hook type a caller may add, and mere adds none,
so ours is the only hook on mere's endpoint (checked). The ruling does not
rest on it.

**Ruling 52.** *Ruling 50's two wordings need the card and `djinn-devices` to
know which rule decided `connected`, and the directory does not carry it;
`DeviceDirectoryV1` and `PairedDeviceV1` reject unknown fields at version 1.
How does it travel?* Options: a per-device flag, staying at version 1; the
same, bumped to version 2; a directory-level flag; a per-device enum naming
the rule. Mark: **"Per-device flag, stay v1 (Recommended)"**. Follows:
`on_overlay` on the transport's `KnownPeer`, carried into `PairedDeviceV1`
with a serde default; the resident and `djinn-devices` ship together in one
crate.

**Ruling 53.** *The polling check failed in 1 of 10 suites without polling
changing anything: each run's rule followed its own connection close within
19 and 52 ms, but iroh closed at 14.92 s in one run and 9.98 s in the other,
and the check compared absolute times within 2 s. What should it compare?*
Options: the delay after each run's own close; absolute times with a wider
tolerance. Mark: **"Delay after own close (Recommended)"**.

**Ruling 54.** *The dual-dial test's delivery-gap check failed in 1 of 10
suites on the liveness branch ("the link stopped delivering at 6.99 s"),
quiet in 20 earlier suites that ran without the hook and without the new
killed-peer test; the data cannot separate the hook from load. What next?*
Options: an A/B first (10 suites without the killed-peer test, 10 with it
but no hook); loosen the check; accept the flake. Mark: **"A/B first
(Recommended)"**.

**Ruling 55.** *The killed-peer control was reworded because "the path rule
still reads connected at kill + 60 s" races iroh's own 60 s timer (it
cleared at 60.007 s once). As built, the path rule must read connected at
59 s and clear no earlier than 60 s, which held in 12 of 12 runs. Keep it?*
Options: keep it; the original wording. Mark: **"Keep 59 s / not before 60 s
(Recommended)"**.

**Ruling 56.** *Ruling 54's A/B separated nothing: the dual-dial delivery gap
has appeared once in 50 parallel suites, on the first liveness build, and
not again in any arm (hook on without the killed-peer test 0 of 10; hook off
with it 0 of 10; the final build 0 of 10, some under heavy outside load; 0
of 20 before the hook). What now?* Options: merge and watch for recurrence;
a bigger hook-on run first; loosen the gap check. Mark: **"Merge; watch for
recurrence (Recommended)"**. Follows: the gap is recorded as one
unexplained event; a second occurrence reopens it, with its logs.

**Ruling 57.** *On iroh 1.3, D1b failed at its first restart in 1 of 3 runs
(the second in the other 2). The resident logs carry no gossip detail.
Reading: ruling 36's stale pending entry also forms at first contact when
one side's `Join` lands before the other has joined; the side that joined
alone holds it, and restarting the other side fails at once.* Options:
record it under ruling 36 and let the overlay-fix lane confirm it; confirm
it before the pushes. Mark: **"Record under 36; overlay lane confirms
(Recommended)"**.

**Ruling 58.** *Push (a), Knot: merge Knot's GitHub `main` (`ea3e99e`) into
the repin and push two commits, that merge and `562353a`; Knot is red on its
own until (c).* Options: push; hold. Mark: **"Push (a) (Recommended)"**.
Done 2026-10-04: GitHub's `main` was still `ea3e99e`, the merge tree matched
the lane's conflict-free check (`5689433f`), and `ea3e99e..eb934b4` was
pushed with exactly those two commits.

**Ruling 59.** *Push (b), mere: after (a), commit the lock with Knot
`562353aa` from GitHub, rerun the unmodified gate, djinn, D1 and D1b, and
fast-forward GitHub's `main` from `7587f0df` with the verified branch only:
nine commits (`a5543904`, `b7913620`, `259f2741`, `28292406`, `a44ee831`,
`005e27ad`, `0292f369`, `ea75dfdb` and the lock), not the local `main`'s
unpushed physics commits; stop if the lock diff shows more than Knot's move
from path to git, a check fails, or GitHub's `main` has moved.* Options:
push after its checks; hold. Mark: **"Push (b) after its checks
(Recommended)"**.

**Ruling 60.** *Push (c), Knot's repin onto the pushed mere: one commit
moving mere's rev in 40 lines and genet to `bd3e8861` in 14 rows, after
Knot's workspace check, library, desktop, retinue and knot-document tests
and `cargo tree -d`.* Options: push after its checks; hold. Mark: **"Push
(c) after its checks (Recommended)"**.

**Ruling 61.** *Push (c) stopped at its first check: mere `031b3dcc` added
`on_overlay` to `KnownPeer` (ruling 52), and Knot's test-only helper
`peer(seed, reachable, connected)` (knot-editor `resident.rs:686`) builds
one, so it no longer compiles; it is Knot's only break, and the helper's
tests read only `connected`. What does the helper set?* Options:
`on_overlay: true`; `on_overlay: false`; a new parameter. Mark:
**"on_overlay: true (Recommended)"**. Follows: the line rides in Knot's
repin commit.

**Ruling 62.** *Mere's checks missed it because djinn compiles
knot-editor's library, not its tests. Should the mere/Knot lockstep compile
Knot's test targets against mere's tree before mere pushes a change Knot
consumes?* Options: add it to the lockstep; not now. Mark: **"Add it to the
lockstep (Recommended)"**. Follows: before such a push, `cargo check
--workspace --all-targets` runs in a scratch Knot worktree patched at mere's
tree; recorded in the mere/Knot lockstep memory the same day.

**Ruling 63.** *Push (c) is ready apart from three knot-desktop Collapse
tests failing with "missing Show Preview" (`apps/desktop/tests/collapse.rs:44`),
which fail identically on Knot's `main` before the repin (`ea3e99e`, a clean
control worktree on its own lock); every other check passes. Commit and push
(c)?* Options: push, naming the three failures; hold until Collapse is fixed
on Knot `main`. Mark: **"Push (c), name the 3 failures (Recommended)"**.
Follows: the commit message names them with the control, and they are left
to Collapse. Done 2026-10-04: GitHub's Knot `main` was still `eb934b4`, and
`eb934b4..92367ec` was pushed with the one repin commit.

**Ruling 64.** *Ruling 36's assessment (13 traced runs, iroh-gossip 0.101.0
unchanged on this stack): a restart fails exactly when the survivor holds a
stale pending entry for the restarted side, which predicted all 26 restarts.
Where does the fix live?* Options: patch iroh-gossip; a nudge from
mere-transport (close the peer's connections and re-tag after a dwell).
(The p2panda fork was assessed and reduces to the nudge, since iroh-gossip's
API offers only broadcast and join.) Mark: **"Patch iroh-gossip
(Recommended)"**. Follows: ruling 46 keeps it out of upstream; the upstream
ledger's item 3 records it.

**Ruling 65.** *Which change? A1: in `on_join` (`proto/hyparview.rs:380-401`),
clear a peer's pending entry when it is already an active neighbour, so its
`Join` is always answered (about 3 lines; every route to a stale entry). A2:
in `on_neighbor` (`:450-459`), do not record a reply as pending (about 2
lines; this route only, and it touches refill and forward-join).* Options:
A1 only; A1 and A2; A2 only. Mark: **"A1 only (Recommended)"**.

**Ruling 66.** *How is the patched iroh-gossip carried?* Options: vendored in
mere's `support/patches/` with Knot pointing at mere.git; a tagged fork
repository, like vello and p2panda. Mark: **"A tagged fork repo"**.
Follows: a `mark-ik/iroh-gossip` fork with a tag, patched by row in both
mere and Knot. Creating it, its base commit and its tag come to Mark first.

**Ruling 67.** *Proving data moves both ways after a restart needs a write
on a resident that is already running; the receipts spawn the real binary,
so a test-only path cannot do it. How?* Options: a real resident flag
(`--seed-node-after <secs> <address> <title>`); a receipt-only environment
variable; a cargo feature. Mark: **"A real resident flag (Recommended)"**.

**Ruling 68.** *The fork's base: the `v0.101.0` tag (`2ce78afe`, what mere
and Knot run) or upstream `main` (`2885dd9f`, unreleased)?* Mark: **"Determine
the difference between upstream main and this pinned commit. The fork may be
entirely unnecessary"**. Follows: the comparison in §6 (2026-10-04, upstream
iroh-gossip): `main` does not fix it, and no release does.

**Ruling 69.** *Create the public `mark-ik/iroh-gossip` fork and clone it?*
Options: create and clone; Mark creates it. Mark: **"Wait"**. Superseded by
ruling 72.

**Ruling 70.** *The fork's tag?* Options: `mere-iroh-gossip-0.101.0`, the
p2panda fork's pattern; `v0.101.0-mere.1`. Mark:
**"mere-iroh-gossip-0.101.0 (Recommended)"**.

**Ruling 71.** *With the comparison in hand, base the fork on the `v0.101.0`
tag or upstream `main`?* Options: the tag, `2ce78afe`; upstream `main`,
`2885dd9f`. Mark: **"Upstream main, 2885dd9f"**.

**Ruling 72.** *Create the fork now?* Options: create and clone it; wait for
upstream, rechecking PR #159 and issue #172 at iroh-gossip's next release,
with D1b failing at second restarts meanwhile; Mark creates it. Mark: **"Wait
for upstream"**. Follows: no fork is made. Ruling 36's fix is on hold, and
so is ruling 67's flag, which serves only its receipt; D1b's restart item
stays open. At iroh-gossip's next release, #159 and #172 are rechecked, and
if the release passes the bug by, the upstream ledger's rule applies
(ruling 46), with rulings 70 and 71 then giving the fork's tag and base.

**Ruling 73.** *The 0.7.5 repin and both liveness rules are pushed and the
restart-gap fix waits on iroh-gossip's next release; D2 is next in this
plan. What next?* Options: assess D2; chatelaine P4a; pause here. Mark:
**"Chatelaine P4a"**. Follows: this plan pauses before D2, and the
chatelaine plan resumes (its ruling 51).

**Ruling 74.** *This phase's scratch (the `C:\t\mere-repin`, `knot-repin`,
`knot-control-ea3e99e` and `mere-iroh12` worktrees, and the build targets
in `C:\t\p2panda-075-targets` and `C:\t\mere-verify-target`): remove it?*
Options: worktrees and targets; logs too; keep everything. Mark: **"Remove
worktrees and targets (Recommended)"**. Done 2026-10-04, after checking each
worktree clean with its `HEAD` on GitHub: C: went from 470 GB to 714 GB
free. Mark's standing rule, given the same day: keep C: above 500 GB free,
and clean before building below 100 GB.

Also given in the same conversation (2026-10-01, Mark: "You can edit known
hosts"): `known_hosts` entries may be updated, which was done for the
ThinkPad (`.32`) and Q-PC (`.68`, `q-pc.local`), each key added only after
its fingerprint matched the machine's recorded key.

## 4. Corrections to other documents

Found by the assessment, each checked against code; carried into those
documents the same day as dated corrections.

1. **SSH CA plan.** Its T2 and design text say `enroll-host` writes a host
   certificate and retires TOFU fleet-wide; the code writes only an
   `authorized_keys` `cert-authority` line, and nothing mints or installs a
   host certificate. Its premise that one master is the fleet authority does
   not hold either: each vault's master, and so its CA, is its own.
2. **Reachability plan, R1.** It places `PairedDevice` in Graphshell's
   `device_sync`; it now lives in djinn (`ports/djinn/src/settings.rs:468`).
3. **Reachability plan, R2.** It says announces already bind authenticated
   app data; the code sends none, deliberately, so an announce fits a
   255-byte LoRa frame (`crates/murm/transport/src/reticulum_transport/announce.rs:38-51`),
   which also constrains R2's proposed `EndpointAddr` payload.

Lane-reported and not re-checked here (code comments, recorded for the
phases that touch them): `ports/djinn/src/pairing.rs:46-48` says the host
never writes the settings file back, while `personal_sync.rs` writes hints;
`crates/murm/transport/src/peer_id.rs:11-14` says a `PeerID` derives from the
master, while hosts pass per-graph seeds; `djinn --pairing-facts` computes
the group pre-key and omits it from its output (`ports/djinn/src/bin/djinn.rs:405-439`).

## 5. Phases

Each phase lands verified, as the chatelaine plan's lanes did. Nothing here
changes personae's agent path in the same slice as anything else: it is how
Mark SSHes into his machines.

- **D1 — the directory.** A djinn app-door route lists each paired device:
  its node id, label, whether it is connected, its current path and its last
  hint; a CLI reads it (ruling 11). Proven locally first (ruling 14). Done
  when:
  - [x] with two djinn instances on one machine (separate profiles, paired to
        each other), the output matches the resident's peer directory, and a
        peer restarted on a new endpoint shows its new path within one poll;
  - [x] a stopped peer shows as not connected (a negative control);
  - [x] only the owner's processes are admitted to the route;
  - [ ] after D2, the same holds across real machines when a lease moves.

- **D1b — mDNS-only first contact (ruling 16).** Two paired residents that
  share a network find and connect to each other with no ticket, the way
  R0's August receipts did across two machines, including two residents on
  one machine. Diagnosed before it is fixed: the first evidence points at
  p2panda's gossip healer re-joining only when a topic's set of nodes
  changes, not when a known node gains an address. A fix inside the
  `mark-ik/p2panda` fork (pinned by tag, `mere-p2panda-net-0.7.4`) means a
  new tag and a workspace repin, so where the fix lives comes to Mark first.
  *Ruled 2026-10-02* (rulings 19 to 21): both fixes. M1, in mere-transport,
  lands now; F1, in the fork, lands with a fork release built on upstream
  p2panda's main (its 54 new commits merged first), tagged and pushed only on
  Mark's word, and nothing goes upstream for now. Done when:
  - [x] the cause is shown in the code and reproduced, with the evidence
        recorded in §6;
  - [x] two residents on one machine, paired with no ticket and with mDNS
        their only way to meet, connect, and keep doing so across restarts
        (a control without the fix fails the same run);
        *2026-10-02 annotation:* ticked on `connected` under ruling 22's
        rule, which counted iroh's path; after a restart that signal hid an
        overlay gap (§6, ruling 36), so the restart half is reopened by the
        next item;
  - [ ] (added 2026-10-02, ruling 36) at every restart, in either order, the
        restarted side gets a gossip neighbour and data moves both ways within
        one poll, `connected` following the overlay (ruling 31);
  - [ ] the same holds between two real machines, which D2 makes possible.

- **D2 — a resident on every machine.** djinn runs as a systemd user unit on
  Fedora and a launchd agent on macOS (ruling 9), beside its existing Windows
  installer; on macOS, LAN discovery additionally needs a signed app
  carrying the local-network usage declaration (reachability R0), while
  dialling out does not. Done when:
  - [ ] each of the four machines runs the resident at login, its personal
        sync paired with the others;
        *2026-10-04, reading, not ruled:* five machines, with the Surface
        Laptop 3 (Kubuntu, so a systemd user unit as on Fedora) assessed
        at D2's start;
  - [ ] the M4's agent is the current one (its agent predates certificates),
        with Mark's own SSH into each machine unaffected throughout;
  - [ ] on Windows, djinn runs beside the legacy `graphshell-device-host` on
        its own pipe until its agent signs a real SSH login, and then Mark
        switches with djinn's installer (ruling 15).

- **D3 — SSH by key.** `ssh <device>` runs a `ProxyCommand` helper that asks
  the local resident to dial the device's node id on its own ALPN; the
  target's resident admits paired devices only and forwards to its local
  `sshd` (rulings 2, 3, 12). Done when:
  - [ ] `ssh <alias>` reaches each machine after its address changed, with
        no sweep and no new prompt;
  - [ ] an unpaired peer, and a peer after unpairing, is refused;
  - [ ] a forged host key still fails;
  - [ ] each machine is reached as a target, the Windows laptop included,
        once its OpenSSH Server is installed (an administrator step, Mark's).

- **D4 — host certificates, approved by the master (rulings 5, 6, 10).**
  Done when:
  - [ ] each machine's own CA signs that machine's host certificate, through
        a new mint-and-install path (installing it is a root or
        administrator step on each host, Mark's);
  - [ ] the Windows laptop's master issues an approval for each machine CA,
        and revoking one removes its trust everywhere within one refresh;
  - [ ] each client's `known_hosts` carries one `@cert-authority` line per
        approved CA, written from checked approvals;
  - [ ] a host's first contact shows no prompt, and a host key that is not
        certified by an approved CA is refused.

- **D5 — one pairing ceremony (ruling 8).** One ticket, typed code and SAS
  records a new device for personal sync, Knot and SSH trust (its machine CA
  submitted for the master's approval); unpairing removes all three. Done
  when:
  - [ ] pairing a fresh device takes one ceremony, its step count measured
        against today's six plus two separate ceremonies;
  - [ ] a wrong code fails (a negative control).

- **D6 — the other network (ruling 4).** A self-hosted iroh-relay both
  networks can reach, set on every resident. Where it runs is a fork for
  Mark at this phase's start. Done when:
  - [ ] the M4 on its wired-only network is reached by key through the relay;
  - [ ] a control isolates the relay as what carried the dial (direct paths
        blocked), which is also the reachability plan's open R1 receipt.

## 6. Findings

**2026-10-01.** The ThinkPad's lease moved and it does not advertise over
mDNS (absent from a `_ssh._tcp.local` browse while both iMacs answer); avahi
runs with no service files; firewalld's `FedoraWorkstation` zone opens UDP
1025-65535, which includes 5353, so its firewall is not the block. Q-PC's
lease moved too. Both were confirmed by host-key fingerprint before any
`known_hosts` change.

**2026-10-02: no machine runs djinn.** The Windows laptop's SSH agent is
served by the legacy `graphshell-device-host.exe` (built 2026-08-16, in
`AppData\Local\Graphshell\bin`, its log showing the session's signing
requests), the resident djinn's installer is written to retire
(`ports/djinn/install-windows.ps1:18`). The ThinkPad, M4 and Q-PC run no
djinn. A first check reported djinn running on the ThinkPad: `pgrep -f djinn`
had matched its own command line, which contained the word; `pgrep -a` and
`systemctl --user` showed nothing. Instruments that match on command lines
need a control that excludes themselves.

**2026-10-02: D1 landed.** Built as `e15b8fcb` (lane, Opus) and merged onto
`main` as `4963b489` after verification in the normal-depth worktree. The
transport gained `peer_paths` (every address the endpoint holds for a peer,
each marked active or not) and a ticket decoder, so nothing outside it parses
an iroh ticket; djinn serves `device-directory-v1`, a read-only route listing
each paired device (node id, label, root, pairing id, time added, connected,
reachable, current path, the saved hint decoded), granted to the `djinn`
label only; `djinn-devices [--json]` reads it. Verified: djinn 84 unit tests
plus its integration targets, `mere-transport` 49, graphshell's library 191,
the portable gate; and the two-instance live receipt, run again by me
(temporary profiles and pipes, the installed resident PID 53336 before and
after): first contact 2.2 s after the second resident started, given a
ticket; a restarted peer's new path in 56 ms and its saved hint 1.6 s later;
a stopped peer shown not connected after 73.5 s, the transport's path
timeout. The lane's refusal test admits the granted label and refuses
Turnstone, an unknown app and an old protocol hello; its three controls each
failed their target.

Findings from D1, for D1b and D2:

- **mDNS alone made no first contact between two residents on one machine**:
  each learned the other's address and neither dialled for 45 s; with a
  ticket they connected in 3.45 s. Ruling 16 makes this D1b.
- **Saved hints accumulate**: after a restart a hint holds the old and new
  ports, and includes the laptop's WSL/Hyper-V adapter address
  (`172.28.32.1`) and global IPv6 addresses; existing R1 behaviour.
- **A local djinn cannot share the standard agent pipe** with the installed
  resident: its listener accepts only `\\.\pipe\openssh-ssh-agent`, so local
  instances need `--receipt-agent-endpoint`; D2's side-by-side run on
  Windows (ruling 15) needs the same.
- **A third node on the LAN**, `9b662f09…`, sends transport info the current
  p2panda rejects. That id is the supplier in August's personal-sync
  receipts (reference host plan, reachability plan R1), so it is most likely
  the installed legacy resident's own sync identity on this laptop
  (*reading, not verified*).

**2026-10-02: D1b's cause, found and checked.** A paired device id is tagged
onto the gossip overlay (`P2pandaOverlayHost::seed_peers`,
`crates/murm/transport/src/p2panda_host.rs:103-125`) before the address book
holds any record for it; topic membership means "has a record and the topic"
(stickleback's store and upstream's SQLite store alike), so gossip's one-time
bootstrap query returns nobody and joins with nobody. When mDNS later writes
the record, the fork's address book never tells the healer: it recomputes
topic watchers only on topic writes (`p2panda-net/src/address_book/actor.rs:131-145`),
while `InsertNodeInfo` and `InsertTransportInfo`, where mDNS writes, notify
only per-node watchers (checked at `0a54ab82`). A ticket works because it
writes the record before the join. The lane's instrumentation showed each
side's record arriving by the first poll and the healer's view staying
empty. It is not single-host: mDNS succeeded on one host, and two machines
meeting for the first time with no ticket would fail the same way.
Reproduced by a new ignored test, `ports/djinn/tests/mdns_first_contact_two_instance.rs`
(no contact in 90 s on the clean tree). Both fixes passed it end to end:
F1 (about 20 lines in the fork, with a fork test that fails in 10 s
unpatched and passes in 0.16 s) gave first contact 2.19 s after spawn; M1
(mere only) 2.25 s; each with a control that fails. The fork patch applies
cleanly to `0a54ab82`. The comment at
`ports/graphshell/src/native/personal_sync_host.rs:240-243`, which says
`g5_peer` proved this path, is wrong and is fixed with M1.

**2026-10-02: M1 landed.** Built as `8a8d8fc2` on the reproduction test
`c73e6082` (lane, Opus); merged onto `main` as `177b927c` after verification
in the normal-depth worktree. `set_topics` and `add_topics` give a paired
peer an empty address-book record before tagging it, written through a new
stickleback `insert_node_info_if_absent` that checks and writes inside one
muniment transaction, so a record mDNS writes first is kept and one written
later builds on it. The directory says "not connected (no address known)"
for a device not yet seen; djinn's warning classifier gives an info line
when no paired device has an address and keeps the firewall warning for
when one does and nothing connects. Verified: djinn, `mere-transport`,
stickleback and graphshell's library tests, the portable gate; D1b's
ticketless receipt, run again by me, connected on first contact 2.61 s
after the second resident started and reconnected after each side
restarted without its hint (2.17 s, 2.30 s); D1's ticketed receipt still
passes; the lane's control (M1's two calls disabled) found no contact in
90 s. The installed resident stayed on PID 53336.

**2026-10-02: a flaky `connected`.** The transport test
`the_peer_directory_separates_a_known_address_from_a_live_path` fails 4 of 30
runs on `main` without M1 and 5 of 30 with it (my runs; the lane measured 11
of 80 and 5 of 80), so it predates M1. The lane's instrumentation tied it to
simultaneous dials: a one-sided copy failed 0 of 80 against 7 of 80. Ruling
22 is the fix; a lane is on it.

**2026-10-02: the fork merge, in scratch.** In a clone at `C:\t\p2panda-merge`
(branch `mere-merge-upstream-2026-10-02`), upstream's main merged into the
fork as `8efae5ff`, with one textual conflict (`sync/log_sync/builder.rs`:
the fork's `protocol_id` kept beside upstream's hooks) and one semantic one
(a test's `StreamItem`, renamed upstream to `LogEntry`); no fork patch is
made redundant. F1 applied unchanged as `d532713f`; its test failed 40 of
40 on the merged tree without the fix and never at the join with it. The
fork's suite is flaky on this machine in every tree; repeated interleaved
runs show no failure attributable to the merge or to F1. Building mere
against it: one compile error (stickleback's `StreamItem`); iroh, iroh-base
and iroh-relay move 1.2.0 to 1.3.0 and `iroh-mdns-address-lookup` 0.5.0 to
0.6.0, one copy each; six manifests and knot pin `=0.7.4`, so the release
follows the knot-first lockstep. Nothing is tagged or pushed.

**2026-10-02: H10's per-interface mDNS fix has not been in effect.** mere's
patch for `iroh-mdns-address-lookup` (its fork at 0.4.0, carrying upstream
PR #7's per-interface multicast sockets) is `[[patch.unused]]` in the lock;
the live crate is crates.io's 0.5.0, required by `mere-p2panda-net` 0.7.4,
which has no per-interface sockets. This laptop is multi-homed (its WSL
adapter address appears in saved hints). Cargo has warned "patch was not
used" on every build; in chatelaine P1's verification I recorded that
warning as harmless and lock-wide, which was wrong. Ruling 25 brings the
fork to 0.6.0 with the repin. The `boa_engine` and `boa_gc` patches are
reported unused on the same line and were not examined here.

**2026-10-02: the fork lane's second round.** With a scratch-only rename of
stickleback's `StreamItem`, mere's whole workspace checks against the merged
fork, mere-transport, stickleback and djinn pass, and knot compiles through
djinn. The CBOR audit covers 73 calls: 39 strict (36 in mere, 3 in knot:
everything decoding a peer's data, a signature or a round-tripped canonical
form; signalman's shared control-frame decoder needs a strict variant for
its frames while its local snapshot stays lenient) and 33 lenient (local
data, tests, examples); the table is `C:\t\cbor-decode-audit-177b927c.tsv`.
Upstream's PR #7 is still unmerged (last activity 2026-07-28); ported onto
0.6.0 unchanged, the patch becomes used (`cargo tree` resolves the fork's
path, no `[[patch.unused]]`), and djinn then binds mDNS on the WSL adapter
too, but the port and the original PR both fail upstream's `mdns_subscribe`
test on this laptop (stock passes), Wi-Fi already holds the multicast route
here (metric 35 against WSL's 5000), and first contact was about 0.6 s
slower; hence ruling 26. The installed legacy resident already binds 5353
per interface, so some earlier build carried the fix.

**2026-10-02: iroh's abandon check, unchanged in 1.3.0.** The `connected`
lane traced the path gaps during simultaneous dials to iroh's
`NoqPathEvent::Abandoned` handler, which marks an address abandoned when the
closing connection's own paths no longer reach it, although its comment says
"once no connections have any path". The handler is byte-identical in iroh
1.2.0 (`remote_state.rs:595-613`) and 1.3.0 (`:600-618`), so the repin does
not remove it; ruling 30 keeps the old test one-sided and ruling 31 makes
gossip authoritative for peers on the overlay.

**Sibling pins at the release.** turnstone (root `Cargo.toml:342-349`),
cleromancy (`:97-102`) and isometry (`:112`, `:246-264`) pin the fork's
0.7.4 tag beside older mere revisions, which require `=0.7.4`; each moves to
0.7.5 at its own next mere repin, as the insigne proofs plan's phase C
handoff did.

**2026-10-02: rulings 32 to 34 landed in another session's merge.** They were
staged in the shared primary checkout while another session's merge was in
progress (`MERGE_HEAD` set, a conflict open in the physics catalog plan), so
that session's merge commit `8022cedd` ("Merge gpu-repulsion…") carries
them; a `git notes` entry on `8022cedd` says so. The content is as intended.
From here, staging in the shared checkout first checks that no merge,
rebase or cherry-pick is in progress.

**2026-10-02: ruling 31 built, and a gap after restarts.** The `connected`
lane built it as `8eb08a84` on ruling 30's `f212bd63`. "Subscribed" is read
from the address book's self record: the gossip manager tags this node with
the topic when it subscribes and untags it when it leaves (probed before,
during and after). While subscribed, `connected` is membership in the gossip
manager's neighbour set, the record LogSync reads; otherwise it is iroh's
active path. A stopped peer read not connected 92 to 398 ms after its close
(20 of 20), iroh still marking its path active each time; D1's ticketed
receipt passes, and gossip marked a killed resident down at 17.66 s against
the path's 74.7 s.

D1b's receipt failed 3 of 3 at its second restart. Measured over 12 runs
(`main` `177b927c` and `8eb08a84`, both restart orders, 3 each; logs in
`C:\t\pairing-d1b\halfstate`):

- At first contact both sides send `Join` and every `Neighbor` is answered.
- At the first restart the survivor answers the fresh `Join` with a
  `Neighbor`; the restarted side takes it as a request and answers with its
  own, which the survivor counts as the reply, so the restarted side keeps a
  pending entry that nothing clears.
- At the second restart the survivor is the side holding that entry, and
  `send_neighbor` (iroh-gossip 0.101.0, `proto/hyparview.rs:745-753`) sends
  only when its pending insert succeeds, so the fresh `Join` gets no answer
  and the restarted side never gains a neighbour. The failure follows the
  second restart in either order, not a name.
- Data from the restarted side reached the survivor 16.7 to 113.0 s after the
  restart, carried by whichever of the survivor's LogSync retries was
  accepted (one every 15 s: `RETRY_RATE`, 5 s, at
  `p2panda-net/src/sync/actors/topic_manager.rs:35`, plus a 10 s timeout);
  the restarted side starts no session without a neighbour. The other
  direction needs a runtime write the resident lacks (`--seed-node` runs only
  at start).
- `main` shows the same gossip trace in 6 of 6 while its directory read
  connected within 0.17 to 2.26 s, so the gap predates ruling 31 and the OR
  rule hid it. The survivor reading the restarted peer connected throughout
  fits `accept_conn` swapping the connection silently (`net.rs:772-801`;
  *reading, not instrumented*).

Also from the lane: the simultaneous-dial control flaked in 1 of 10 parallel
suites (ruling 37); and iroh's endpoint `close()` stalled past 10 s after
simultaneous dials three times in one suite run, the likely cause of an
earlier unexplained hang (the test now logs the stall and moves on).

**2026-10-03: 0.7.5 released; knot and mere repinned locally.**

- **The release.** Per ruling 33 it was rebuilt on GitHub's fork `main`:
  merge `91bafa2b` takes `94947fd1` without changing a file (its tree equals
  `293dcafa`'s), and release `1bec457e` carries `d91f748b`'s content (tree
  `6fc3c069`, the tested tree); the annotated tag `mere-p2panda-net-0.7.5`
  points there. Pushed without force, and `ls-remote` shows `main` and the
  tag at `1bec457e` (checked).
- **The repins**, both local:
  - Knot's is `562353aa`.
  - mere's is `a5543904`: 34 files, signalman joined (ruling 35), the 36
    strict sites, and a strictness control
    (`control_frame_refuses_non_canonical_cbor`) that fails when
    `GroupControlFrame::from_bytes` decodes leniently.
  - mere's lock waits for Knot's rev on GitHub. It moves 1651 to 1667
    packages: iroh, iroh-base and iroh-relay 1.2.0 to 1.3.0, iroh-metrics
    1.0.2, `iroh-mdns-address-lookup` 0.6.0, the eight p2panda packages to
    `1bec457e`, and signalman's 16. One copy each of iroh's and p2panda's
    packages.
- **Passing on that stack:** the portable gate (817 s), djinn 98,
  mere-transport 50, stickleback 91, signalman 22 in the workspace, and
  graphshell's library 191, among others. D1b's receipt also passes (first
  contact 2.32 s, reconnected 2.64 s after each restart), and so does D1b
  with M1 disabled, so F1 alone carries first contact.
- **D1 fails its stopped-peer control.** On iroh 1.3.0 a killed peer read
  connected for 199.5 s (600 s limit), against 73.5 s on 1.2.0 (ruling 40).
- **Also found:**
  - Knot's own tests need Knot's genet moved to mere's `bd3e8861`: 740
    type-split errors without it, 260 passing with it. Step (c) does that.
  - Cargo fetched `merely-made/mere.git` and `genet.git` from GitHub to load
    Knot's pinned revs.
  - Another session deleted shared target directories mid-run.
  - Retinue's desktop workspace, outside mere, will need `p2panda-core`
    patched to the 0.7.5 tag, since signalman now calls
    `decode_cbor_strict`.
  - Isometry decodes a peer's operation body leniently
    (`crates/isonetry/src/campaign_space/space.rs:77`, checked) and should
    go strict at its next repin (ruling 24).

**2026-10-03: ruling 31 landed; iroh 1.3 is not slower.**

- **The `connected` lane's last round** (`05fb3de2`): the simultaneous-dial
  budget went from 16 to 48 pairs (ruling 37). Ten parallel suites with the
  budget at 64 tripped after 3, 2, 2, 1, 2, 9, 4, 8, 3 and 3 pairs; ten
  more at 48 passed 10 of 10, tripping after 1, 12, 1, 8, 18, 4, 3, 2, 1
  and 1. Each pair's keys and topic are now numbered from its index, since
  the old scheme reused other tests' identities beyond about 16 pairs. The
  overlay wording is one constant, `NOT_CONNECTED_PATH_ACTIVE`, asserted by
  the card's test and by `djinn-devices`' first test (ruling 39); D1b's
  module note records its expected failure (ruling 36).
- **Merged** as `fdb02bd3`, after verification in the normal-depth worktree
  on `main` `39787d82` (merge `78d3245a`):
  - mere-transport passed 52 of 52 in three runs over two merges;
    stickleback 85 and 5; djinn's 15 test binaries (86 library tests).
  - D1 passed twice, the stopped peer reading not connected after 10.45 s
    and 14.02 s (73.5 s under the path rule).
  - D1b failed at its second restart both times, as ruling 36 expects.
  - The control: with the gossip branch disabled, the stopped-peer test
    fails at its 10 s limit. A first control, made in `peers_for_topic`,
    passed because the test calls the counting function directly; it proved
    nothing and was replaced.
  - The installed resident stayed on PID 53336 throughout.
  - The first djinn run failed to link (`0xc0000142`) while five other
    sessions were building; the rerun at `-j 2` passed.
  - `main` moved by four doc-only commits between verification and merge,
    so the merged tree differs from the verified one in those four files
    only.
- **iroh 1.3 (ruling 40)**, measured by the release lane with D1's harness,
  a 600 s limit and the same kill, seconds until the stopped peer read not
  connected:
  - quiet: iroh 1.2 74.9, 74.3, 74.7; iroh 1.3 with p2panda 0.7.4 73.9,
    81.5, 73.7; the repin 85.1, 74.1, 73.8;
  - under other sessions' builds: iroh 1.2 103.2, 237.3, 178.8; the repin
    311.8, 188.3, 173.3 (and the earlier 199.5); a pure CPU burner did not
    stretch it.
  - The cause, identical in both versions: `connected` read "any address
    Active" in `remote_info`, and it flipped within about 0.1 s of the
    peer's remote actor terminating. The actor lives until 60 s after its
    last queued message (`ACTOR_MAX_IDLE_TIMEOUT`, iroh 1.3.0
    `socket/remote_map/remote_state.rs:74`, reset at `:265-269`; checked),
    and the queued messages after the last close were a final sync dial's
    datagrams or our own `RemoteInfo` requests (406 in 70 s). Closing a
    connection does not mark its paths inactive (`:475-491`, checked;
    *reading* that nothing else does).
  - Gossip marked a killed neighbour down 9.9 to 16.7 s after the kill
    across 13 traced runs on iroh 1.3.
- **Retinue 0.2.0 on `main`**: another session's `f62581c7` moved the root
  and signalman to `fa4f925` and djinn's knot-site to Knot `ea3e99e`
  (ruling 44).
- **Reported by the physics session, not investigated here:** graphshell's
  `carrier::tests::p2panda_murm_grant_is_refused_before_projection_bytes`
  hit its 10 s timeout in several of its runs, twice when run alone, and
  passed single-threaded in its latest run; load is a candidate.

**2026-10-03: the liveness assessment (ruling 47) and the release merge.**

- **What iroh offers.** No per-remote connection count or connection event
  stream; `RemoteInfo` carries only addresses with their usage. The
  endpoint hook `EndpointHooks::after_handshake` (iroh 1.3.0
  `endpoint/hooks.rs:87-107`) runs in the single constructor for accepted
  and dialled connections, and `Builder::hooks` appends rather than replaces
  (`endpoint.rs:780-791`, checked). `WeakConnectionHandle::closed()`
  (`connection.rs:1352`) reports the close without keeping the connection
  alive. p2panda's builder appends hooks the same way
  (`p2panda-net/src/iroh_endpoint/builder.rs:85-94`, checked) and installs
  its own authoriser hook (*corrected 2026-10-03:* it installs none; see
  ruling 51). The hook code is identical in iroh 1.2.0 and
  1.3.0, and p2panda's builder and hooks are unchanged between 0.7.4 and
  0.7.5 (checked). p2panda hashes ALPNs with its network id, so a hook sees
  no protocol names.
- **The probe** (iroh 1.2.0, 4 runs, possibly load-affected): the hook saw
  both connections to a child-process peer, which closed 12.02 to 12.16 s
  after its kill. The path rule cleared at 72.19 and 72.12 s, the last close
  plus 60 s, whether polled every 200 ms or not. One poller did not extend
  it, so the release lane's long tails need messages to queue, which load
  supplies (*reading*).
- **Who reads `connected`:** in production only personal sync (djinn's
  directory and poll loop). Peers off the overlay today are the startup
  window before LogSync subscribes, transports without gossip, and topics
  left; a D3 ALPN-only peer would be the first deliberate one.
- **The release merge.** `259f2741` merges `c64b834b` into the repin:
  - Retinue comes through the workspace (ruling 44). outrider, postilion and
    radio-hand are pinned exactly in the root table (`=0.2.0`, `=0.2.0`,
    `=0.0.1`), as the brief wrote them, while `main`'s retinue row is caret
    `0.2.0` (*reading, not ruled*).
  - knot-site at Knot `ea3e99e` names no p2panda package.
  - The lock goes from 1653 to 1669 packages, with one retinue and one copy
    each of iroh's and p2panda's packages.
  - Passing: the gate, transport 52, stickleback 91, signalman 22, djinn
    100, and graphshell's library 191 (one first-run timeout of the carrier
    test, then 5 of 5 alone and the whole library). The CBOR control fails
    when lenient.
  - D1's stopped peer read not connected after 10.48 s; D1b fails at its
    second restart (ruling 36). PID 53336 throughout.

**2026-10-03: the open-connection count, first build.** The `connected` lane
built rulings 47 to 51 as `28292406` on `main` `277a911e` (branch
`pairing-liveness`), with no fork or lock change. `OpenConnections` counts
connections per remote from `after_handshake` and decrements when each weak
handle's `closed()` fires; it holds only a shared map. It is always
installed in `bind_inner`, with a test-only switch for the control. Off the
overlay the live rule never calls `remote_info`. Measured on a shared,
loaded machine:

- A killed peer off the overlay (a child process holding one own-ALPN
  connection, 12 runs): its connection closed 9.99 to 14.92 s after the
  kill, and the rule read not connected 0.01 to 0.11 s after that; iroh's
  path rule read connected at 59 s every time and cleared at 60.01 to
  65.08 s. With three pollers every 200 ms, the rule still followed its own
  close within 0.11 s, while the path rule did not clear within 150 s in any
  run.
- A connection opened through `protocol_endpoint()` and one opened by
  gossip were both counted; a graceful close read not connected in 10.5 ms;
  without the hook, the killed-peer test fails before the kill.
- The overlay is unchanged: the live-path test 50 of 50 alone, the
  dual-dial test with its control, the stopped-peer test; D1's stopped peer
  at 18.16 s and 14.39 s (the first likely load; that path does not go
  through the change); D1b failing only at its second restart.
- Gates: djinn, stickleback, clippy counts unchanged. mere-transport passed
  8 of 10 parallel suites, each about 155 s: one failed the polling check on
  absolute times (ruling 53), one the dual-dial delivery-gap check
  (ruling 54).
- Not yet built: ruling 50's wordings, which needed ruling 52.

**2026-10-04: the open-connection count landed.**

- **The second round** (`a44ee831`):
  - `KnownPeer` and `PairedDeviceV1` carry `on_overlay`; the directory
    stays at version 1, and an entry without the field reads as `false`.
  - The two wordings are constants (`NOT_CONNECTED_NO_NEIGHBOUR`,
    `NOT_CONNECTED_NO_CONNECTION`), chosen by one function both surfaces
    call, with both literals asserted on both.
  - The polling check compares each run's delay after its own close
    (ruling 53). In 10 suites the close came 9.90 to 10.02 s after the
    kill, and the rule followed within 0.12 s, polled or not. iroh's path
    rule read connected at 59 s every time, cleared at 60.01 to 65.10 s
    unpolled, and never within 150 s with pollers.
- **Ruling 54's A/B** is recorded in ruling 56. The lane's reading of the
  arms: A skipped both child-spawning tests; B's scratch switched the hook
  off except for the killed-peer scenario's own parent, since with it off
  everywhere that test fails in about 10 s and its load disappears.
- **Merged** as `005e27ad`, after verification in the normal-depth worktree
  on `main` `ce79a82f` (merge `cc697b8f`):
  - mere-transport 56 of 56 twice (about 155 s each); stickleback 85 and 5;
    djinn's 15 test binaries.
  - D1's stopped peer read not connected after 11.03 s. D1b failed at its
    second restart (ruling 36).
  - The control: with the hook never installed, both liveness tests fail.
  - The installed resident stayed on PID 53336.
  - `main` had moved by one doc file, so the merged tree differs from the
    verified one in that file only.
- **Open:** the dual-dial delivery gap, one event in 50 suites (ruling 56).

**2026-10-04: the repin on iroh 1.3 with liveness, and push (a).**

- **The merge.** The release lane merged `main` (`0292f369`) into the repin
  as `ea75dfdb`, without conflict. The lock is identical to the previous
  round's verified one: 1653 to 1669 packages, one retinue, one copy each
  of iroh's and p2panda's packages.
- **Checks on iroh 1.3:** the gate, mere-transport in 10 parallel suites
  (56 passed each, about 163 s; the delivery-gap check never tripped),
  stickleback 91, signalman 22, graphshell's library 191, djinn 101, and
  the CBOR control.
- **Liveness on iroh 1.3:**
  - a killed peer off the overlay closed 9.99 to 10.05 s after the kill and
    read not connected within 0.12 s of that, polled or not;
  - iroh's path rule read connected at 59 s every time;
  - a graceful close off the overlay read not connected in 10 to 50 ms, and
    a stopped peer on it in 2 to 38 ms.
- **Receipts:** D1's stopped peer read not connected after 10.42 s. D1b
  failed at its second restart in 2 of 3 runs and at its first in 1
  (ruling 57). PID 53336 throughout.
- **The push lists.** GitHub's mere `main` was `7587f0df`, and the local
  `main` held 21 unpushed commits from the physics session's Density branch
  besides this plan's four, so push (b) carries the verified repin branch
  only (ruling 59).
- **Push (a)** is done (ruling 58).

**2026-10-04: push (b), and (c)'s stop.**

- **Push (b)** (ruling 59). Once Knot's `562353aa` was on GitHub, the lock
  resolved with only the four Knot crates moving from the path patch to
  git (1669 packages before and after). It was committed alone as
  `031b3dcc`. Checks before the push:
  - the unmodified `cargo_mode.py verify` passed (1542 packages);
  - djinn 101 passed;
  - D1's stopped peer read not connected after 11.10 s, and D1b failed at
    its second restart (ruling 36);
  - PID 53336 throughout.

  GitHub's `main` was still `7587f0df`, and `7587f0df..031b3dcc` was pushed
  as a fast-forward of exactly the nine listed commits. The local `main`,
  with the physics session's unpushed commits and this plan's, has diverged
  from it; the physics session, told, merged `origin/main` into it as
  `fb12ce11`.
- **Push (c)** moved Knot's 40 mere rows to `031b3dcc` and its 14 genet rows
  to `bd3e8861` (no other rev left in any manifest; the lock has one mere
  and one genet). It stopped at the workspace check on the test helper
  (ruling 61). knot-desktop and its retinue check passed.
- **Push (c)** (rulings 61 and 63). With the helper setting `on_overlay:
  true`, Knot's checks passed: the workspace check, knot-editor 149,
  knot-document 47, the retinue check, and `cargo tree -d` with one copy
  each of mere, genet, iroh and p2panda. The exception is three of
  knot-desktop's four Collapse tests (290 others passed), which fail the
  same way on `ea3e99e`. Committed as `92367ec` and pushed as
  `eb934b4..92367ec`. Knot builds on its own again.
- **The 0.7.5 repin is complete:** the fork at `1bec457e`, Knot at
  `92367ec`, mere at `031b3dcc`. Turnstone, Cleromancy and Isometry move at
  their own next mere repins, Isometry with its strict site, and Retinue's
  desktop workspace needs the `p2panda-core` patch row (findings above).

**2026-10-04: ruling 36's assessment.** On the repinned stack (iroh 1.3.0,
iroh-gossip 0.101.0, p2panda 0.7.5), 13 two-resident runs were made with
gossip debug traces: mDNS-only and ticketed first contact, both restart
orders, and a seeded node on each restarted side.

- **The model held** (measured). Replaying each side's pending set from its
  log predicted all 26 restarts: a restart works exactly when its survivor
  holds no pending entry for it. Every first restart worked and every
  second failed. In the failures the survivor got the fresh `Join` 1.4 to
  5 s after the restart and sent no `Neighbor`, and the seeded node arrived
  109.1 to 112.4 s late, against 0.76 to 2.2 s when a restart worked.
- **Ruling 57's reading, corrected.** A first-restart failure was not
  reproduced (0 of 13). Its precondition was seen once, harmless in that
  run's order (`A-mdns-ba\2a0e5358`): a received b's `Join` and answered
  with `Neighbor`, which reached b before a's own queued `Join`s; b took it
  as a new request and replied, and a took that reply as the answer. So a
  stale entry forms on whichever side receives the other's `Neighbor`
  before processing the other's `Join`. One side joining alone is one route
  to that, and a `Neighbor` overtaking a `Join` in a symmetric contact is
  another. All 13 first contacts delivered both `Join`s, 2 to 96 ms apart,
  ticketed or not.
- **Fix locations.** iroh-gossip's API offers only `Broadcast`,
  `BroadcastNeighbors` and `JoinPeers` (`api.rs:376-382`, checked), so a
  re-join from the p2panda fork cannot help while the survivor's entry
  stands; that option reduces to closing connections, the mere-side nudge.
  The crate is about 8,000 lines. Knot restates mere's p2panda rows in its
  own patch table (checked), so a patch row is needed in both.
- **iroh-gossip 0.101.0's source** is upstream commit `2ce78afe`, which
  upstream's `v0.101.0` tag points to (`.cargo_vcs_info.json`, `git
  ls-remote`); upstream `main` is at `2885dd9f`.

**2026-10-04: upstream iroh-gossip (ruling 68).** Read through GitHub's API,
nothing downloaded:

- **`main` against our pin.** `2ce78afe...2885dd9f` is 3 commits ahead and 0
  behind: CI workflows, upstream's `Cargo.lock`, `deny.toml` and one line in
  `src/bin/sim.rs`. `proto/hyparview.rs` is untouched, so `main` does not
  fix it, and crates.io's newest release is 0.101.0.
- **Issue #172** (open, 2026-09-29, no replies) reports the same last step:
  a pending entry makes `send_neighbor` ignore a returning peer's `Join`.
  It reaches that state by another route, quitting one topic closes a
  connection the other topics share (`state.rs:322`), and its proposed fix
  would not cover a killed process on one topic. Its open question, about
  an old and a new connection overlapping after a restart, is our silent
  swap.
- **PR #159** (open, not draft, conflicting with `main`, no reviews; an
  outside contributor's, 2026-09-16 to 2026-10-04) carries A1 line for line
  in `on_join` ("the old request must not suppress our reply"), inside a
  2,435-line rework of dial ownership and reconnects across 11 files.
- **PR #163** (a maintainer's draft asserting HyParView's paper claims) and
  **PR #121** change other pending-entry paths, not `on_join`'s.

**2026-10-08: Retinue 0.3 at the root (rulings 42 and 44 followed).** The
root's four Retinue rows move to retinue `main` `773515f2` together:
retinue `0.3.0`, outrider `=0.3.0`, postilion `=0.3.0`, radio-hand
`=0.0.1`. seneschal `0.2.0` enters the lock under postilion and radio-hand
and needs no row of its own.
- **No source change was forced.** mere-transport (`--features reticulum`)
  and mere-signalman build, test and lint clean against the new API: neither
  calls `set_freshness_policy`, reads `address_book::Peer`,
  `SinglePacketReceipt` or the counters, and the public paths they use
  survived the file split. (Signalman's clippy also needed one
  `collapsible_if` in `head.rs`, which fails on the old pin too and is
  committed separately.)
- **Behaviour that reaches mere** (retinue `fd41131`, `e3468b9`, `f30f000`):
  routes now live a week, not 30 minutes, unless an interface mode caps them;
  links carry RNS keepalives and are torn down when stale, and a read on a
  dropped `LinkStream` fails with `TimedOut` instead of hanging; inbound
  links are bounded per endpoint and destination.
- **One Retinue no longer holds.** Knot's `knot-site` (at mere's Knot
  `eabd4434`, under djinn) names retinue `fa4f925` itself, so the lock now
  carries retinue 0.2.0 for knot-site beside 0.3.0 for everything else. No
  type crosses between them in mere (checked); the second copy goes when
  Knot repins knot-site.
- **Licence.** retinue and outrider are under the Reticulum License from
  0.3.0 (retinue `313198c`); postilion, seneschal and radio-hand stay
  MPL-2.0. mere vendors none of their source, so `LICENSES.md` is unchanged.

## 7. Progress

**2026-10-02.** Assessed by a read-only lane (Sonnet) and ruled in three
rounds (rulings 1 to 13); the load-bearing claims re-checked in code; the
machines' CAs read from their agents. Nothing built. Next: Mark's go to
start D1.

**2026-10-06, progress since 2026-10-02.** The tree recorded in one place,
since this section stopped at the assessment:
- D1, the paired-device directory route and CLI: `4963b489`.
- D1b's mere fix M1, mDNS-only first contact in mere-transport:
  `177b927c`.
- `connected` by gossip while subscribed (rulings 30, 31, 36 to 39):
  `fdb02bd3`.
- Open-connection liveness off the overlay (rulings 47 to 55):
  `005e27ad`.
- The 0.7.5 repin: mere's lock `031b3dcc`, Knot `92367ec`, the fork
  `1bec457e`.

*Reading, not ruled:* restarting D2 now touches decisions made since, in
the vault lock plan:
- its ruling 17 (djinn hosts the agent; the standalone `personae-agent`
  and its installers retire after D2);
- its ruling 22 (macOS joins with D2);
- its ruling 42 (a Linux resident starts locked and waits for a prompt).

So D2's assessment should start from the vault lock's state. The order
is Mark's to set.
