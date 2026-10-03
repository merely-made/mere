# Batch 24 — dramatis tier architecture (new document)

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---:|---:|---:|---:|---:|
| dramatis_docs/technical_architecture/2026-09-30_dramatis_tier_architecture.md | current | yes | 31 | 31 | 0 | 0 |
| **Totals** |  |  | **31** | **31** | **0** | **0** |

**Totals: 1 doc, 31 claims checked (31 holds, 0 stale, 0 unverifiable), 0 contradictions.**

Audit base: Mere `3fd2b147` (2026-09-30), which includes gaz M2's first intake
slice (`c388babb`); re-audited at `316e8217` (2026-10-01) after §7 changed
from open seams to rulings. `archive_docs/` is excluded.

This batch exists because the document is new. Its rulings (3 on 2026-09-30;
6 to 15 on 2026-10-01) are recorded in the document's §9 and carried the same
day into the documents they touch: the dramatis tier plan, the credential
port + gazette brief, the djinn resident services plan, the crate
consolidation plan, the standards survey's decision 5, the insigne proofs
plan, the gazette, castellan, chatelaine and dramatis READMEs, and smolweb's
home decision. They are not counted again here.

## dramatis_docs/technical_architecture/2026-09-30_dramatis_tier_architecture.md

- disposition: current
- status line: "Status: current. The tier's architecture of record … It synthesises the plans listed in §8 and does not replace them" — accurate: yes
- claims checked: 31 — holds: 31, stale: 0, unverifiable: 0

### Stale claims

- none.

### Contradictions

- none. The document's rulings amend the credential brief's "gazette carries
  discovery inward", gazette's README boundary "gazette reads what is already
  public", the tier plan's empty-facade checkbox, and the djinn plan's
  ownership table; each carries a dated amendment pointing here.

### Recommended action

- none for this record. Every 2026-10-01 ruling is marked unbuilt in the
  document.

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
"already public" boundary; the crate consolidation plan's 2026-09-23 limit
that castellan's sealed OTP store stays in castellan.

Added 2026-10-01: djinn's site service binding loopback Gemini only
(`ports/djinn/src/resident_site.rs:1007`); the djinn plan's split of lifetime
and scheduling from domain policy, and its scheduler's first two consumers;
finger-protocol's `webfinger` feature compiling for wasm32-unknown-unknown
(checked from smolweb's workspace, exit 0); reqwest 0.12.28 compiling
`blocking` out on wasm32 through `if_hyper!` (its `src/lib.rs`, the version in
mere's lock); finger-protocol's `Jrd` and `Link` deriving `Serialize` and
`Deserialize` with RFC 7033 §4.4's fields; gazette's `WebFingerDocument`
deriving `Deserialize` only, with a required `subject` and a link of `rel`,
`type` and `href`; errand taking finger-protocol without `webfinger`
(`crates/system/errand/src/finger.rs`); CXF v1.0's 17 credential types
(Proposed Standard, 2026-03-09, read from the FIDO Alliance specification);
castellan's `ImportSshKeyNativeIntentV1` (`authority.rs`) and its `OtpItemId`
and `OtpItem` (`otp/item.rs`); and the sibling pins, eight repos at four mere
revisions, read from their manifests.
