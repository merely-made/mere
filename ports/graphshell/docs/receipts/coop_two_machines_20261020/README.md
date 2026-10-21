# Graphshell co-op across two physical Macs

This packet qualifies one existing graph shared by an owner on an Intel Mac
with one invited editor on an M4 Mac. Both run native Graphshell and their own
isolated, passphrase-encrypted Djinn profile. The owner has a second, private
graph before co-op begins. The guest has no owner session list or archive.

The controller uses SSH for JSON control, empty synchronization markers and the
public contact/invitation handoff. It forwards no data-plane ports and injects
no graph state into the guest. Graph reads, edits, acknowledgements, push updates
and revocation travel through the existing signed Mere service and Murm carrier.
The new runtime observation reads that carrier's existing peer paths and records
which route is active; it establishes no extra connection or authority.

`machine.py` owns native processes, local custody and captures on each machine.
`run.py` orchestrates the seven phases. It uses the shared Mesquite headed lane
and retained native semantics. The preparation helper creates the existing
graphs and performs an ordinary owner write through admitted resident APIs;
it never writes the store directly. `freeze.py` records executable hashes,
compiler inputs, dependency trees and pinned Git suppliers. `verify.py` checks
the public evidence and its exact source tree, including a committed tree.

Keys, vaults, stores, passphrases, live invitations, contact files and executables
stay outside this packet. Public evidence contains synthetic source addresses,
public test identities, process and route observations, and owned captures.

The acceptance is bounded to native macOS, one owner and one editor, online
co-op and orderly cold reopen. It does not qualify Windows/Linux, browser/PWA
hosting, physical keyboard or IME interaction, internet/WAN traversal, offline
concurrent editing, document bodies or power-loss durability. Revocation removes
authority and clears the live guest view; it cannot recall independent copies
of information already observed.

## Reproduction

Build Graphshell Mere, Djinn with `passphrase-fd`, and the owner's
`coop_existing_graph` preparation example using Rust/Cargo 1.99.1 and the locked
dependencies. Freeze each architecture with `freeze.py`. Both desktop sessions
must be logged in and unlocked. Supply the two frozen binary directories, the
remote task checkout and worker, an existing SSH control socket and fresh short
output paths to `run.py --help`. Short output paths avoid macOS Unix socket
length limits. Each machine generates and retains its own random test
passphrase only in worker memory.

Verify the assembled public packet with:

```sh
python3 ports/graphshell/docs/receipts/coop_two_machines_20261020/verify.py --commit HEAD
```

## Qualified result, 2026-10-20

The accepted `gs-c2-20261020-04` cohort passes all seven phases with 22
reviewed nonblank captures, seven distinct machine/native-PID pairs and four
successful custodian launches and ordinary stops. The owner runs x86_64 macOS;
the guest is the physical M4 iMac running arm64 macOS 26.5.1. Salted hardware
identifiers distinguish the machines without recording hardware UUIDs.

The guest's existing carrier reports **active direct 192.168.4.68:56242**,
matching the owner's LAN interface; the guest return also reports a direct
IPv6 path to the owner. SSH carries control and handoff only;
there are no forwarded data-plane ports or guest graph-state writes.

The guest adds a source and revises the original source by its durable UUID.
The owner's ordinary resident write pushes a third source and relationship.
Guest activity retains the admitted guest identity. Switching the owner to the
private sibling leaves the guest scoped to the chosen graph. The private
sibling's graph, facets, session identity and Changes count match before and
after the complete run.

Revocation clears the live guest's graph and selected record. Both native
apps and both custodians then stop. Fresh launches retain the exact three
nodes, relationship, saved authors and journal revision; the admission epoch
advances and the grant remains revoked. The owner saves a fourth source.
A fresh M4 guest is refused as `Delegation(Revoked)` using the byte-identical
original invitation, with fresh owner connection information. Thus the cold
refusal exercises the old grant against restored authority.

The current runtime source is `262fe3631acec3620020c08628bf567514d8ece6`,
based on main `cf0dc82f27d2d7036ec48eac01c3d63baa4d8446`. It includes the
current shared site/contributor stack. Both builds record identical hashes for
3,289 repository/compiler-declared inputs and the executed harness, plus
18 pinned Git suppliers and normal/build dependency trees. Resolved external
path packages are excluded from the actual compiled trees. Rust/Cargo 1.99.1
is explicit on both machines.

The two-machine run exposed a packaging prerequisite that the earlier
same-machine run could not: macOS dropped incoming flows for the unsigned
Intel executable because it lacked a designated code requirement. The
per-app firewall query still reported permitted. A minimal Murm probe isolated
the failure below Graphshell admission; ordinary bidirectional UDP worked.
The frozen candidate now ad-hoc signs its owned bundle and executables and
verifies them before launch. No personal signing identity is used. The user
allowed the test app through macOS's native prompt; the agent did not change
firewall policy. The controller also owns the actual native process directly,
avoiding `open -W` returning before the M4 native PID was available.
`diagnostic-attempts.json` and `diagnostics/` retain the rejected attempts and
public diagnostic evidence, separate from the accepted cohort.

Selected software gates pass: 339 Graphshell library tests with six existing
ignores, 16 native-view tests, 30 resident Mere tests and one custody-policy
regression on Intel; 16 native-view tests and ten focused co-op tests on M4.
Changed-file formatting and port boundaries pass. Both selected Intel Clippy
runs complete with existing warnings. Commands and compressed logs are in
`software-gates.json` and `software/`; these are bounded checks, not a complete
workspace or CI qualification. `capture-review.json` records all 22 visual
inspections, backed by three contact sheets and individual critical captures.

During final freshness checking, main advanced to `2764c84895ba7bca564113eb14aecf7894cf1592`
with six Markdown-only description updates. They were merged at
`33171fa2811af2fb9b0111f6c0bff599a9afefa1`. All recorded compiler and harness
input hashes remain identical; the packet verifier checks those inputs against
the final committed tree, so the earlier runtime result is not relabelled as a
new binary build. Both primary checkouts retain their separate active work.
