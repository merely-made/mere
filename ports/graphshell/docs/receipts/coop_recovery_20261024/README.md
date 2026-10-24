# Current-stack relay recovery and cold reopen

The accepted `gs-recovery-20261024-02` cohort ran on
`217d666a4570c2c2953068e76dfc683698765b2f`, with product runtime and root
lockfile inputs matching main `ea5c50966`. The Intel Mac owner
and physical Fedora ThinkPad guest used native windows; the guest used GNOME
Wayland and the Radeon Vulkan driver. Seven native clients, four Djinn
custodian lifetimes and 27 presented captures passed. Semantic controls and
presentation barriers drove the windows; physical input was not exercised.

The controlled outage stopped the owned relay before a guest title save.
The guest showed Working and then Unavailable without a save acknowledgement.
Complete owner reads before the outage and after the failed save were
byte-identical. After the same relay address and both endpoints were available,
the same guest window reconnected and recovered revision 11 unchanged.
Explicit retry received `Saved · owner acknowledged`, produced revision 12,
and appended exactly one guest authorship entry. This controlled outage proves
that this edit did not apply; it does not imply every timeout means no commit.

The chosen existing graph stayed isolated from the owner's private sibling
and roster. Owner and guest changes, the relationship and exact authorship
survived both custodians restarting. Live withdrawal cleared the guest graph,
selection and details. The byte-identical old invitation was refused as Revoked
after cold reopen; the owner retained the graph and could keep editing.

## Qualification repair and consolidation

The first cohort timed out waiting for the retried save after the relay
listener restarted. Its controller released Reconnect 7.4 seconds before the
last original endpoint registered again. Listener readiness did not establish
peer readiness. The shared controller now observes both original endpoints
on the restarted relay before releasing Reconnect, with a bounded observation
deadline. `accepted/relay-ready-for-reconnect.json` is checked against the relay
events. Duplicate, missing and unrelated endpoint registrations cannot satisfy
the barrier. The first failure, registration timing and process cleanup are
retained under `diagnostics/`; they are not counted as a passing cohort.

The successful fresh cohort used byte-identical executables and product
source files. No application timeout, automatic edit retry or authorization
behavior was changed. Active desktop, network, custody-lifetime, freezing and
locked relay-fixture helpers now live together in the
[shared harness](../../../../../support/graphshell-coop/README.md). Execution
no longer imports helpers from dated receipt directories. The historical
[relay](../coop_relay_20261020/README.md) and
[recovery](../coop_recovery_20261021/README.md) packets remain unchanged and
still pass verification; new packets add evidence rather than script copies.

## Stack and evidence limits

Both machines passed 39 native-view tests and 11 focused co-op tests, and
built Graphshell, Djinn and the existing-graph fixture. Exact commands, counts
and logs are in `software-gates.json`. Each freeze verified 21 Git suppliers;
both machines used identical workspace and harness source hashes. The native
app does not link Castellan; identity and durable owner writes stay behind
the resident. These are focused gates, not a full workspace suite or CI claim.

Every admitted guest capture reports the exact owned relay. PID-bound network
namespace evidence proves accepted-but-dropped guest UDP sends, a working
outside UDP control receiver and TCP relay reachability. The fixture uses
private-LAN HTTP with encrypted endpoint traffic. WAN/TLS relay deployment
and general NAT traversal remain outside this qualification.

All accepted client, custodian, proxy, relay and worker PIDs were independently
confirmed stopped. Temporary idle inhibitors were removed. Primary checkouts,
installed apps and personal vaults were preserved. A separate active worktree
advanced main with documentation-only commit `72b445c63` during qualification;
that checkout was left untouched and the final integration receives its commit. Public fixture artifacts
and hashes are included; executables, invitations, credentials and stores
remain in private task outputs.

Verify with:

```sh
python3 support/graphshell-coop/verify.py \
  --receipt ports/graphshell/docs/receipts/coop_recovery_20261024 \
  --commit 217d666a4570c2c2953068e76dfc683698765b2f
```
