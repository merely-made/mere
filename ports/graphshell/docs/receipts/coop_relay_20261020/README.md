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
