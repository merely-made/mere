# Native Graphshell co-op through an explicit relay

This receipt reuses the complete seven-phase Mac-to-ThinkPad controller and
native machine owner. The wrappers add one temporary, owned Iroh 1.3 relay on
the ThinkPad, persisted `coop_relay_urls` in the Mac's isolated Djinn profile,
and an isolated user/network namespace for each native guest process. The
installed rootless Pasta forwarder allows TCP and drops UDP; IPv6 forwarding
is disabled. A successful UDP send, an empty outside receiver, a working
outside control receiver and TCP relay reachability are checked before each
native exec. Neither machine's firewall policy is changed. The native PID is
preserved through exec and bound to both the namespace proof and headed receipt.

The shared native workflow qualifies one existing chosen graph, one owner and
one invited editor: private sibling isolation, owner/guest writes and saved
authorship, live withdrawal, both native clients and both custodians stopping,
exact saved state on cold reopen, and refusal of the byte-identical old grant.
The guest worker rejects any active direct path. The final verifier additionally
requires that every admitted guest capture uses the exact selected relay and
that the relay records actual forwarded bytes and packets.

The test relay uses plain HTTP on the private LAN; the carried Iroh QUIC
connection remains authenticated and encrypted between endpoints. This is a
controlled relay-path qualification, not an internet deployment, TLS relay
configuration, general NAT traversal, or a public relay service. No personal
profile, key, passphrase, live invitation, binary or store is published.

`run.py`, `machine.py` and `freeze.py` reuse the existing receipt modules and
hash every executed shared module. The small relay is an isolated, locked
Cargo fixture rather than a new product dependency. Its native build and
registry source checks accompany the final packet.

## Qualified result

Accepted cohort `gs-relay-20261020-04` ran on source
`166d0aefdb520599612c137c63764427ab51cb54`, integrating current main
`1797a2eed` before rebuilding both hosts. All seven real native processes and
four ordinary custodian launches passed, with 22 presented captures. The
owner was the Intel Mac; the guest was the physical Fedora ThinkPad, using
GNOME Wayland and its Radeon Vulkan render node. Retained semantic controls
and presentation barriers drove actual native windows. This does not qualify
physical mouse or keyboard input.

The guest added a source and renamed the original source by its existing
identity. An ordinary owner edit and relation arrived in the guest view.
Owner and guest authorship survived the cold reopen exactly, as did the
three-node graph, relation and revision. The private sibling stayed unchanged
and never appeared in the guest's graph, sessions or archives. Ending access
cleared the live guest canvas, selection and record details. After both native
clients and both custodians stopped, the byte-identical invitation was refused
with `Delegation(Revoked)`; the owner retained the graph and could keep editing.

Every admitted guest capture reported exactly
`active relay http://192.168.4.32:32867/`. The relay accepted five connections
from exactly two endpoint identities, including both endpoints after the cold
restart, and recorded 108,327 received / 106,800 sent bytes and 187 received /
183 sent packets. All three guest native processes have PID-bound network
proofs: UDP sends succeeded without reaching the outside control receiver,
TCP reached the relay, and the network namespace differed from the host's.
Pasta, native clients, custodians and relay all stopped; independent process
inspection found none of the accepted subjects remaining.

## Stack and configuration

The product change composes the existing Murm/P2panda/Iroh relay builder in the
owner's resident activity. Persist `coop_relay_urls` in the existing Djinn
profile settings, then restart Djinn; for example, add
`"coop_relay_urls": ["https://relay.example/"]` alongside the profile's other
settings. An absent or empty list preserves LAN-only behavior. URL validation
uses the shared host-policy parser. Before freezing a contact, the activity
waits for an explicitly selected relay to become reachable; failure closes the
transport and returns an error. Relay reachability does not grant graph access.
No transport fork, dependency pin, default public relay or alternate graph writer
was introduced. The guest learns the owner's relay through the existing contact.

The fixture's relay executable is separate from product dependencies. Its
locked registry archives and all 10,002 files in 274 compiled registry packages
were checked against the installed source. Native provenance records 3,325
workspace files, 18 resolved Git suppliers, dependency trees, executable hashes,
and the exact reused controller modules. The final rebuilds produced the same
native executable hashes as the software-tested implementation commit;
`software-gates.json` records the actual gate provenance and unrelated source
snapshot delta. The Mac passed 340 core tests (six existing ignored), 36 native
UI tests, 23 settings tests and 30 resident tests. Both hosts passed 11 focused
co-op tests, including unreachable-relay refusal; the ThinkPad also passed 36
native UI tests. Clippy completed with existing warnings; boundary and selected
format checks passed. These are bounded gates, not full-workspace or CI results.

`diagnostic-attempts.json` preserves rejected runs and diagnostics. Earlier
macOS deny-UDP sandbox runs were intermittent; they do not establish a
lower-stack root cause and are not acceptance evidence. The final run uses
successful-but-dropped guest UDP with Pasta's TCP forwarding instead. Shared
prior receipt modules remain unchanged. `artifact-sha256.json`, `verify.py`,
`capture-review.json`, and `cleanup.json` bind the reviewable public packet.
Run `python3 ports/graphshell/docs/receipts/coop_relay_20261020/verify.py
--commit COMMIT` to verify it against a committed tree. Live stores, invitations,
contacts, passphrases and native executables remain in private task outputs.
