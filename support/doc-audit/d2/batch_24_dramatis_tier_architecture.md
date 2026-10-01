# Batch 24 — dramatis tier architecture (new document)

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---:|---:|---:|---:|---:|
| dramatis_docs/technical_architecture/2026-09-30_dramatis_tier_architecture.md | current | yes | 20 | 20 | 0 | 0 |
| **Totals** |  |  | **20** | **20** | **0** | **0** |

**Totals: 1 doc, 20 claims checked (20 holds, 0 stale, 0 unverifiable), 0 contradictions.**

Audit base: Mere `3fd2b147` (2026-09-30), which includes gaz M2's first intake
slice (`c388babb`). `archive_docs/` is excluded.

This batch exists because the document is new. Its one ruling (castellan
issues, gazette announces) is recorded in the document's §9 and carried the
same day into the dramatis tier plan's progress log, the credential port +
gazette brief, and the gazette, castellan and dramatis READMEs; it is not
counted again here.

## dramatis_docs/technical_architecture/2026-09-30_dramatis_tier_architecture.md

- disposition: current
- status line: "Status: current. The tier's architecture of record … It synthesises the plans listed in §8 and does not replace them" — accurate: yes
- claims checked: 20 — holds: 20, stale: 0, unverifiable: 0

### Stale claims

- none.

### Contradictions

- none. The document's ruling amends the credential brief's "gazette carries
  discovery inward" and gazette's README boundary "gazette reads what is
  already public"; both carry a dated amendment pointing here.

### Recommended action

- none for this record. The open seams in §7 are named as open by ruling.

### Notes

The claims checked, against source at the audit base: djinn owning one
`CastellanResident` (`ports/djinn/src/resident.rs`); personae's `Issue`
(`delegation.rs`) and `AttestationKeys` and `IdentityProvider`
(`provider.rs`); castellan's `PersonaeHost` (`authority.rs`), `OtpReleaseGate`
(`otp/release.rs`), `DeviceGrantView` (`view.rs`) and `sealed_backend`
(`sealed_storage.rs`); insigne's `CheckedAttestation`, `CheckedCertificate`,
`CheckedRevocation` and `CheckFault`; gaz's `Anchor`, `root`, `keys_for`,
`verify_scope`, `alarms`, `is_alarming`, `intake_addresses` and the
`jscontact` `PublicCard`; gaz's manifest (insigne as the only tier runtime
dependency, personae and insigne `verify` as dev-dependencies); gazette's
`WebFingerIntake::from_import`, its blocking `reqwest`, and its README's
"already public" boundary; the djinn resident plan naming only Personae and
Castellan in the tier; the crate consolidation plan's chatelaine row ("ruled:
design first"); and the `dramatis` crate having no dependent outside the
workspace root's table.
