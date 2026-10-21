# Native co-op from Mac to ThinkPad

This packet qualifies one existing graph shared by an owner on an Intel Mac
with one invited editor on a physical ThinkPad running Fedora 44 and GNOME
Wayland. Each machine owns an isolated, passphrase-encrypted Djinn profile.
The owner has a second private graph before any co-op controls open.

The controller uses SSH for JSON control, empty synchronization markers and
public contact/invitation handoff. It forwards no data-plane ports and writes
no graph state into the guest. Graph reads, edits, acknowledgements, push and
revocation use the existing signed Mere service and Murm carrier. Carrier
observations report the actual active peer path.

The seven-phase scenario is the same sequence qualified between the two Macs:
choose an existing graph, invite one editor, add a guest source, switch the
owner to a private sibling, return deliberately, push an ordinary owner write
and relationship, revise the original source by UUID, return in a fresh guest
process, revoke, stop both apps and custodians, reopen both, retain exact saved
state and refuse the byte-identical old invitation. A final admitted read
compares the private sibling's graph, facets, session and Changes count.

`run.py` controls `machine.py` independently on each host. The worker launches
the actual native executable and asserts its exit status. On Linux it resolves
the active local graphical session through logind and GNOME's screensaver,
then uses the real Wayland socket with private test data/runtime directories.
The Radeon Vulkan ICD is selected explicitly. Owned-process DRM descriptors
and loaded driver-library observations accompany the guest's native receipt.
No virtual display, Xvfb, browser or SSH X forwarding is used.

The shared Mesquite lane activates retained native controls, enters text and
captures host-owned presented frames. This qualifies the declared native
workflow; it does not qualify physical keyboard/IME or OS assistive activation,
Windows, browser/PWA hosting, WAN/relay traversal, offline concurrent editing,
document bodies or power-loss durability. Cold reopen means new application
and custodian processes, not an operating-system reboot. Revocation removes
authority and clears the live view; already observed independent copies cannot
be recalled.

`freeze.py` records native executable hashes, compiler-declared and tracked
repository inputs, dependency trees, toolchain and Git-supplier audits. Mac
artifacts are ad-hoc signed and verified; Linux artifacts are native ELF.
The platform-specific compiler input sets may differ, so `verify.py` checks
each set against the same committed source and requires identical hashes for
every common input and the executed harness. It also checks the two machine
identities, process lifecycles, route, scope, attribution, revocation, cold
state and capture review.

Live stores, vaults, passphrases, keys, contact files, invitations and binaries
stay outside Git. Public evidence contains synthetic addresses, public test
identities, process/session observations, captures and hashes. Both primary
checkouts retain their separate work; qualification uses isolated task
checkouts and Rust/Cargo 1.99.1 with locked dependencies.

Build each native client and the owner's preparation example, then explicitly
build the test custodian before freezing:

```sh
cargo +1.99.1 build --locked --offline -j2 -p graphshell-mere --bin graphshell_mere
cargo +1.99.1 build --locked --offline -j2 -p djinn --example coop_existing_graph # owner only
cargo +1.99.1 build --locked --offline -j2 -p djinn --bin djinn --features passphrase-fd
```

The offline commands require cached locked dependencies. Use separate artifact
directories for the two native builds. `freeze.py --help` lists the repository,
artifact, harness and fresh output paths; add `--owner` on the Mac. Freeze probes
the custodian's explicit test feature before accepting the executable.
`run.py --help` lists the frozen directories, SSH control socket, remote task
checkout and worker. Both real desktops must be logged in and unlocked.
Choose fresh, short output paths to stay within macOS Unix socket limits.

After assembling a packet, verify its exact publication tree with:

```sh
python3 ports/graphshell/docs/receipts/coop_mac_thinkpad_20261020/verify.py --commit HEAD
```


## Qualified result

The accepted `gs-tp-20261020-05` cohort passes seven phases with 22 reviewed
nonblank captures, seven distinct native process/machine pairs and four real
custodian launches and ordinary stops. The owner is the physical Intel Mac
running macOS 15.8.1. The guest is the physical x86_64 ThinkPad, model
20U6S1CW00, running Fedora 44 and the active GNOME Wayland desktop. The guest
process's DRM descriptors identify the AMD device (vendor 0x1002), with the
installed Radeon Vulkan driver loaded. Salted identifiers distinguish the
machines without publishing their hardware or machine identifiers directly.

The existing Murm carrier reports an **active direct IPv6 path** to the Mac at
`[2603:6014:5d01:26d3:187a:6e60:eefa:9116]:52139`. The verifier checks that
this address belongs to the owner's interfaces. The earlier successful cohort
used a direct IPv4 LAN path. This is same-network evidence; neither result
establishes WAN or relay traversal.

The guest sees the chosen existing graph with no owner session list or archive,
adds a source, and revises the original source by its durable UUID. Both edits
reach the owner's signed authority and receive the visible **Saved · owner
acknowledged** status. An ordinary owner write pushes a third source and a
relationship to the guest. Journal authors retain the admitted guest identity.
Switching the owner to the private sibling leaves the guest scoped to the
chosen graph. The private graph, facets, session and Changes count match
before and after the complete workflow.

Ending access clears the live guest's graph, selected record and record data.
Both applications and both custodians then stop. Fresh processes retain the
exact three nodes, relationship, author history and journal revision. The
admission epoch advances and the grant remains revoked. The owner saves a
fourth source, then the fresh ThinkPad guest is refused with
`Delegation(Revoked)` using the byte-identical original invitation and fresh
owner connection information. This exercises the old grant against restored
authority.

The final source is `38bc62024bb998f5499178b52c1deb3f61e576b3`, integrating main
`80c938abc` and the qualified `d4a4429a1` runtime, the shared flattened rendering/semantics traversal, native Hest
pending-action routing, explicit-session material validation and immutable
receipt intake. Both native builds record matching hashes for all 3,313
repository/compiler inputs and the executed harness, plus 18 pinned Git
suppliers and their normal/build dependency trees. External path packages are
excluded from the actual compiled trees. Both clients and both custodians
were rebuilt before the accepted run; the earlier successful source and
qualification are retained separately in `diagnostics/`.

The task fixes one native control race: **End guest access** previously returned
silently whenever an unrelated ordinary graph read was pending. It now sends
the revocation independently of that read while requiring both the attached
graph and co-op panel to match the control's chosen graph. A deterministic
regression fails with the original guard and passes after the fix; another
rejects stale controls targeting a different attached graph or panel. The first
headed attempt timed out at owner revocation, but its pending value was not
instrumented, so this packet does not assign that timeout to the guard as an
established cause. A later diagnostic completed withdrawal and cold refusal.
The final standard controller completed repeatedly, including full repeats
after main advanced. Rejected and diagnostic attempts remain distinct in
`diagnostic-attempts.json`.

Selected gates on the final source pass: 339 Graphshell library tests with six
existing ignores on the Mac; 30 native-view and 30 resident Mere tests on each
machine; and ten focused co-op tests on the ThinkPad. Changed-file formatting
and port boundaries pass. Ordinary Mac Clippy completes with existing warnings;
an earlier strict warnings-as-errors probe stopped in shared dependencies and
is retained as a diagnostic. `software-gates.json` names exact commands,
source commits, environment and compressed logs. These are bounded gates,
not a complete workspace or CI qualification.

`capture-review.json` records all 22 visual inspections, backed by three
contact sheets and individual full-resolution checks of the guest's revised
source and cold refusal. `verify.py` checks the packet manifest, current
committed source, executable and harness provenance, real machine/process
observations, scope, author identities, saved cold state and withdrawal.

The final reconciliation adds release receipts and an unrelated inference
integration test. Rebuilding preserves byte-identical native clients,
custodians and owner helper on both hosts. `publication-invariance.json`
records the source delta and executable hashes. The final full controller run
uses that reconciled source; software gates retain their actual `d4a4429a1`
source and explicit reuse provenance. The changed integration test is absent
from the native compiler input set and is outside this acceptance's scope.
