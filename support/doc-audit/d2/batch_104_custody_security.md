# Batch 104 — custody isolation and security boundary audit

Audit date: 2026-10-21 (America/New_York). Adds current coverage for the
canonical security continuation; does not alter the immutable snapshot.

## dramatis_docs/implementation_strategy/2026-10-21_custody_isolation_and_security_hardening_plan.md

- disposition: current
- status line: "H1 production migration remains gated. Native11 passed 38/38 synthetic checks in the separate non-admin runtime account; the measured same-user signing attack failed. Native10 remains a failing counterexample for the signing account. Production Castellan, consent/policy and lifecycle integration remain open; no real consumer has migrated." — accurate: yes
- claims checked: 17 — holds: 17, stale: 0, unverifiable: 0

### Stale claims

None in the seventeen bounded claims checked here.

### Contradictions

The prior conversational claim that macOS supports only explicit locking was
too broad: fallback idle is unknown and the shared timeout treats it as idle.
The plan states missing native screen-lock/suspend hooks and preserves timeout
behavior. Its related-plan table distinguishes historical absolute language
from current mixed-release, control-route and persistence limits.

### Recommended action

Use H1–H4 as implementation gates; OS launch, confinement and key access are required in H1. A successful diagnostic witnesses
an unsafe current behavior, not a successful containment qualification.

### Notes

The seventeen checks are C01–C13's source-qualified boundary claims and W1–W4's
synthetic executed behaviors. Exact frozen source and lock identity, named test
results and limitations are retained in
support/receipts/custody-security-20261021/qualification.json.
Planned OS mechanisms and T01–T10 are explicitly not reported as measured.
No crypto, network exploit, all-platform isolation, installed-fleet qualification
or complete ordinary suite is inferred.

The full D2 command already fails on inherited coverage gaps and an immutable
snapshot digest mismatch at the audited main. This batch covers the new plan;
it does not fabricate judgments for unrelated documents or rewrite that snapshot.
Before/after output is retained, and the comparison must show no new D2 errors.


### Service-split follow-up, 2026-10-21

The user accepted security-motivated service extraction. The plan now proposes
ranked custody, compute, publication, ingress, private-data and short-lived import
roles under a small supervisor. Source modules, current library/GPU coupling and
blob ownership were checked; containment benefits remain proposed and require
the per-split hostile-worker gates. This adds no executed security claim to the
seventeen checks above. The original witness/code hashes remain unchanged.

### Enforcement review, 2026-10-22

The current plan supersedes the earlier service ranking with three authority
tiers and shared worker confinement/launch/IPC. macOS App Sandbox XPC + protected
launchd LaunchAgent + custodian-only data protection Keychain; Linux Landlock/
seccomp workers + dedicated-UID system service; Windows AppContainer workers +
SCM service identity/CNG key ACLs are explicit design defaults. Independent
custody launch and actual enforcement are H1 requirements. DPAPI/TPM and Unix
UID limitations are stated separately from application identity.

Sync activation, feature defaults and real transport binding were re-read:
configured ingress precedes the current loopback publisher. Installed fleet
configuration was not read. Platform documentation supports mechanism selection,
not a claim that this installation enforces it. The seventeen original checked
claims remain source/diagnostic claims, with no new measured security result.
Review references/source hashes and documentation deltas are retained in
support/receipts/custody-security-20261021/review-confinement-20261022.json.

### H1 synthetic implementation, 2026-10-22

The new §4.1 distinguishes measured native primitives from the remaining Rust
Castellan/consent/marker integration. Reviewed native07's owner/control/attacker
results, cold-reopen identity, signing-oracle key/root recovery and successful
synthetic-item deletion. The signing-authority escape is a failed H1 gate; a
valid profile and root-owned code do not remove it. Native10 subsequently
passed 34/35 checks: protected installation hashes/ownership, actual write/create
denials, enabled SMAppService registration and ServiceManagement launch records
are measured. Its remaining failure recovers the exact synthetic vault root
through same-user signing authority. Synthetic item deletion and service
unregistration succeeded; production integration remains open.

Source cuts and failed iterations are retained under
support/receipts/custody-h1-macos-20261022. The prior seventeen source/W1–W4
claims retain their exact original qualification. These additions do not alter
those production sources or assert that real consumers have migrated.

### Separate-account preparation, 2026-10-22

Native11 is a fresh prepared package for a separate standard local account, not
an executed receiving pass. Reviewed the root copy boundary (public pinned
files only; no root writes to a user home), intended-account/GUI/member guards,
exact attacker-payload signing positive control and 42 passing tests. The
builder account was rejected by the live preflight. H1-F1 remains open until
the unrelated app fails to use signing authority and recover the synthetic
root while authorized custody controls pass. Native10 source, documentation
and receipts remain frozen; the prepared follow-up is additive.

### Separate-account receiving result, 2026-10-22

Native11 passed 38/38 checks in the non-admin graphical UID 503 session. Reviewed
exact signer refusal, default-Keychain file EACCES, key lookup -34018, matching
owner/reopen public keys with new instance/PID, stdout/value agreement and the
frozen runner's recomputed evaluation. The root public payload and source
hashes match the prepared manifest. The receiving process reported exact-item
deletion and service unregister; both recorded custodian PIDs are absent.
Cross-account launchd inspection from the builder is denied and does not
independently qualify service absence. Production H1 remains gated by actual
Castellan/service/consumer, consent/policy, C04/C07 and lifecycle work. The
signing account remains outside the passing runtime receipt's scope.
