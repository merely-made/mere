# Co-op in the ordinary native graph view

`graphshell_mere DOMAIN` opens the resident's ordinary graph view. Choose an
existing session in Sessions, then **Use this graph for co-op**. The window
keeps the ordinary MereView canvas, node selection, record details, session
switching and archive controls. Co-op edits use the resident's existing host
and journal for that session; no second GraphSession, copied graph or separate
store is opened.

The selected graph's control panel shows its id, access state and saved activity.
Choose **Guest contact file** and **Invitation folder**, then **Invite editor**
or **Invite reader**. The folder receives `invitation.json` and `contact.json`.
The invitation is recipient-bound and expires after 24 hours. Repeating its
export retains the grant and original expiry. Switching the ordinary view to
another graph does not change the invited graph. The panel clears its old
selection and requires an explicit choice before offering that graph's controls.

The guest exports a public, custody-attested contact and opens the invitation
in the same ordinary native viewer:

```sh
graphshell_mere --contact /absolute/path/guest-contact.json
graphshell_mere --invitation /absolute/path/invitation.json \
  --contact /absolute/path/contact.json
```

An editor can **Add address**, select a canvas node and **Save title**. The guest
sees the invited graph's addresses, titles, relationships and saved authorship.
It has no owner session list, archive, invitation or filesystem endpoint. The
owner can **End guest access**; the signed revocation is stored before success
is reported. A guest's memory-only graph and record selection clear when access
is lost. Information already observed cannot be recalled from independent copies.

Both people use their own unlocked Djinn. The daemon composes its own configured
custody door explicitly, including nondefault endpoint paths. Co-op controls
are not installed in embedded compositions that have not supplied that custody
boundary. Certificate and revocation signing stay inside Djinn. The owner-only
control route is registered on the local application door, never the peer mere
catalog. The peer service retains the existing signed graph, recipient and
permission scope, carrier identity binding and per-operation revocation checks.

The window's request acknowledgement sequence prevents an older background
reading from replacing a pending operation's status with “Saved.” Graph writes
still install only after the ordinary host's durable transaction acknowledges.
Guest attribution comes from admitted identity rather than an edit payload.

The existing-graph run also exposed a resident shutdown cycle: the application
catalog owned a reservoir factory whose route registrar owned the same catalog.
The registrar now holds a weak catalog reference and acquires it only while
registering routes. A regression drops a populated catalog and closes its
reservoir. Co-op listeners and their session waiters use the resident's existing
tracked-task group, so shutdown cancels and joins them before closing storage.

## Acceptance method

`run_native.py` starts two isolated, passphrase-encrypted Djinn profiles. The
preparation example creates two graphs through ordinary admitted resident APIs
before any window or co-op controls open. It never writes a store directly.
The native windows then exercise the existing graph and private sibling, an
editor invitation, guest edits, owner selection changes, ordinary owner edits
and a relationship pushed to the guest, guest process return, revocation,
application and custodian shutdown, cold reopen, refusal of the old invitation,
and a final private-sibling comparison.

The run uses the shared Mesquite headed lane and host-owned captures. Controls
are activated through the retained native semantics; text fields are populated
by the lane. This does not qualify physical keyboard/IME, OS assistive
activation, browser/PWA hosting, internet traversal, source/document bodies,
offline guest editing, or power-loss durability. Native resident DirectoryBackend
sessions are used; browser IndexedDB and standalone co-op Redb stores are not
migrated by this integration.

`run_native.py` is the reproducible controller; `verify.py` checks the evidence,
source hashes and committed publication tree. Keys, vaults, stores, live grants,
contact files and executables stay outside this packet. Public evidence contains
synthetic addresses, public test identities, process observations and captures.

## Qualified result, 2026-10-20

The accepted `gs-coop-view-20261020-08` cohort passes all seven phases with
21 reviewed captures, seven distinct native processes and four custodian
launches. Two different graphs and the first source exist before co-op starts.
The owner invites one editor to the chosen graph, the guest adds a source and
revises the original source by its durable UUID, and an ordinary owner write
pushes a third source and relationship to that guest. Switching the owner's
ordinary view to the private graph leaves the guest on the chosen graph.
The private sibling's graph, facets, session identity and Changes count match
before and after the complete run.

Ending access clears the live guest's graph and selected record. Both native
applications and both custodians then stop; all four custodian launch receipts
end with successful, ordinary stop intents. Fresh launches recover the exact
three saved nodes, relationship, authors and journal revision. The admission
epoch advances and the grant remains revoked. The owner saves a fourth source;
a fresh guest is refused as `Delegation(Revoked)`. Re-exporting connection
information retains the byte-identical original invitation, so this tests the
old grant against current authority rather than silently replacing it.

The frozen source is `523a86808d18ef85eb434570410931f8d370c780`, incorporating
main `d4756e0fb`, the shared site-model extraction and current native renderer.
`build-inputs.json` records 3,287 repository source and compiler-declared
input hashes, three executable hashes, runner hashes and Rust/Cargo 1.99.1.
Application/resident dependency trees and the supplier audit check
18 resolved Git checkouts against their pinned commits, including netrender
`ec15eb269855c798467c93a6d10031100fdfdf65`. The audit covers tracked
Rust/TOML/lock/WGSL/C/C++ code, including resolved but uncompiled packages;
it excludes untracked and asset-only changes.

The final selected software gates pass: 339 Graphshell library tests with six
existing ignores, 16 ordinary-view tests, all 30 resident Mere tests and the
focused custody-policy regression. Changed-file formatting and port boundaries
pass. Both selected Clippy runs complete with existing warnings retained in
logs. These are bounded package/module checks, not full-workspace or CI runs.
`software-gates.json` records commands and compressed logs;
`diagnostic-attempts.json` preserves the five rejected headed attempts and fixes.
`capture-review.json` records inspection of every accepted capture through three
contact sheets, supplemented by individual full captures for the existing-node
edit and cold guest refusal. The review sheets are retained in `review/`.

Earlier complete cohorts remain recoverable: `native05` is frozen in
`prior-cohort/native05`; the complete `native07` packet is retained at commit
`55c8d0f4db70368860222318dbb75945048faa39`. Main advanced during publication,
first adding a shared site-model dependency and then changing the renderer.
Each change produced new executable hashes, so complete headed and cold-reopen
qualification was repeated rather than transferring the earlier result.

Verify the current packet against the exact publication tree, and the retained
native05 packet against its original committed source:

```sh
python3 ports/graphshell/docs/receipts/coop_graph_view_20261020/verify.py --commit HEAD
python3 ports/graphshell/docs/receipts/coop_graph_view_20261020/verify.py \
  --packet ports/graphshell/docs/receipts/coop_graph_view_20261020/prior-cohort/native05 \
  --commit 1197db034
```
