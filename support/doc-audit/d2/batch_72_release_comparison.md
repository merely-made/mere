# D2 release trust and portable recovery comparison

## 2026-07-22_auto-update_brief.md
- disposition: current
- status line: "bounded comparison complete" - accurate: yes
- claims checked: 12 - holds: 12, stale: 0, unverifiable: 0

### Stale claims
Earlier dated Velopack/Luggage verdicts are historical pressure-test results.
The current section explicitly distinguishes successful update cycles and
basic stage persistence from authenticated fresh-process apply, recovery,
production signing, real delta qualification and P2P delivery.

### Contradictions
The Hocket plan says T3 landed and later says P2P delivery is unbuilt. The
current comparison reports this ceiling rather than treating the earlier
summary as a receipt. No product implementation was changed to resolve it.

### Recommended action
Keep wider adoption gated. Qualify TUF as the repository trust provider and
repair/qualify the selected apply backend's signed handoff and recovery before
reusing it across applications.

### Notes
2026-10-12: inspected Luggage signed feed, installed-version comparator,
stage record, Windows installer launch and Linux/macOS replacement source;
current Velopack 1.2.161 Windows/macOS/Linux source; and tough 0.24.0 trust APIs
and upstream public fixtures. Native probes demonstrate Luggage byte-only
refusal versus joint stage/digest acceptance, unsigned stage metadata
acceptance, signed offer replay, and edited/unsigned manifest refusal.
Velopack passes relocated portable/offline apply, preserves state outside
`current`, accepts a harmless staged asset edit, and loses current plus backup
under an injected replacement error. TUF passes expiry, timestamp-edit and
old-root refusal, valid root rotation and the deliberate unsafe-expiry
negative control. Receipts: `support/receipts/luggage-comparison/README.md`.
No product updater adoption, custody change, installation, publication,
native remote fault run, complete TUF deployment or power-loss proof is
claimed. Independent fixture dependency resolution is not an exact app-lock
qualification. Concurrent Mere and Woodshed work is preserved.
