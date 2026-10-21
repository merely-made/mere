# Co-op relay loss and explicit recovery

The accepted `gs-recovery-20261021-01` cohort ran on source
`b58967c7926d0321c700eb225b6de783769830d9`, after refreshing both isolated
checkouts to current main `8942e3050`. The Intel Mac owner and physical Fedora
ThinkPad guest used real native windows, with the guest on GNOME Wayland and
Radeon Vulkan. Seven native processes, four ordinary custodian lifetimes and
27 presented captures passed. Semantic controls and presentation barriers drove
the windows; physical input was not exercised.

The owned relay stopped before a guest title-save attempt. The guest showed
Working and then Unavailable, without an owner save acknowledgement. Complete
owner reads before the outage and after the failed save were byte-identical.
The same guest window reconnected through the restarted relay at its original
address and recovered revision 11 with its original saved title. Explicitly
retrying the save received `Saved · owner acknowledged`, produced revision 12,
and appended exactly one guest authorship entry. This controlled outage proves
that edit was not applied; it does not claim every network timeout means the
owner did not commit an edit.

The chosen graph stayed isolated from the owner's private sibling and roster.
Owner and guest writes, the relation, exact authorship and revision survived
both custodians restarting. Live withdrawal cleared the guest canvas, selected
record and details; the byte-identical old grant was refused as Revoked after
cold reopen. The owner retained the saved graph and could keep editing.

All admitted guest captures reported `active relay
http://192.168.4.32:40895/`. Each guest process has PID-bound evidence of
successful-but-dropped UDP sends, an outside UDP control receiver and TCP relay
reachability. Both relay lifetimes, three Pasta forwarders, clients and
custodians stopped; independent process inspection found none remaining. The
fixture uses private-LAN HTTP with encrypted endpoint QUIC traffic. WAN/TLS
relay deployment and general NAT traversal remain unqualified.

Active execution and verification now live in the [shared harness](../../../../../support/graphshell-coop/README.md).
This packet adds no receipt-specific script copies. Historical providers and
receipts retain their exact bytes; freeze metadata records every executed
provider. The shared verifier also reproduced the prior published relay
packet's 22-capture result before this run. Both hosts passed 39 native UI tests
and 11 focused co-op tests on the refreshed runtime. The test logs and actual
provenance are in `software-gates.json`; these are bounded gates, not a full
workspace suite or CI result. No product behavior change was needed.

Verify this packet with:

```sh
python3 support/graphshell-coop/verify.py \
  --receipt ports/graphshell/docs/receipts/coop_recovery_20261021 --commit COMMIT
```

Private stores, passphrases, contacts, invitations and executables remain in
private task outputs. Only public fixture evidence and source/binary hashes are
included here.
