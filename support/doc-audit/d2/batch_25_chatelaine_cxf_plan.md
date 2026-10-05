# Batch 25 — chatelaine and CXF plan (new plan)

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---:|---:|---:|---:|---:|
| dramatis_docs/implementation_strategy/2026-10-01_chatelaine_cxf_plan.md | current | yes | 12 | 12 | 0 | 0 |
| **Totals** |  |  | **12** | **12** | **0** | **0** |

**Totals: 1 doc, 12 claims checked (12 holds, 0 stale, 0 unverifiable), 0 contradictions.**

Audit base: Mere `cdbeeba6` (2026-10-01). `archive_docs/` is excluded.

This batch exists because the plan is new. Its rulings (16 to 61) are
recorded in its §2 and pointed to from the dramatis tier architecture's §7
and §9 and the crate consolidation plan's C5 log; they are not counted again
here.

## dramatis_docs/implementation_strategy/2026-10-01_chatelaine_cxf_plan.md

- disposition: current
- status line: "Status (2026-10-04): in progress. Shape ruled by Mark on 2026-10-01 (rulings 7 and 10 to 15 in the dramatis tier architecture; rulings 16 to 61 below). P0 met; P1 landed on `main` (`da3c50bc`); P2 landed (`3e4992ec`); P3 landed (`ff68e86c`), meeting the Mere 0.4 baseline's chatelaine condition. The review stop ended 2026-10-04 (ruling 51). P4a's first build signs RSA through `ring` and ECDSA, proven on the ThinkPad; its second round carries rulings 54 to 58 (no re-import overwrite, P-521 refused, unsignable keys refused, and the RSA key built once by the `rsa` crate, ring signing: rulings 58, 59)." — accurate: yes
- claims checked: 12 — holds: 12, stale: 0, unverifiable: 0

### Stale claims

- none.

### Contradictions

- none. The plan makes C5's chatelaine condition include the Secret Service
  move, by Mark's ruling 17; the crate consolidation plan's C5 log records
  the same.

### Recommended action

- none for this record. The two forks first recorded as Mark's (the
  per-kind metadata line, RSA/ECDSA SSH keys) were ruled the same day as
  rulings 23 to 25. Rows of §1's table marked *(reading)* remain open to his
  correction before P1 lands.

### Notes

The claims checked: `OtpItem` and `OtpItemId` as secret-free metadata
(`ports/castellan/src/otp/item.rs`); `OtpAlgorithm`, `OtpKind` and
`OtpCodeStyle` without serde (`otp/mod.rs`); the `castellan/otp/v1` record
directory and its legacy format version; the Secret Service store's
collection and item types and its catalog/collection/item records
(`secret_service/store.rs`, `store/persistence.rs`); `SecretServiceLimits`;
the D-Bus server's Linux-only gating; the two resident HOTP tests by name
(`ports/castellan/src/resident.rs`); 240 references to the moved types in
18 files, all in castellan; personae's fingerprint-keyed, idempotent SSH
slots and `ssh-key` built with `ed25519` only in personae and castellan;
`credential-exchange-format` 0.4.0 as published 2026-06-11 under MIT (the
crates.io API); and CXF's Header, Account, Collection and Item structure
from the 2026-03-09 specification.

Added 2026-10-04 with rulings 51 to 53, two claims read: RUSTSEC-2023-0071
in the RustSec advisory database (`crates/rsa/RUSTSEC-2023-0071.md`,
`patched = []`, 0.9.10 and 0.10.0-rc.18 named affected), and `rsa` 0.9.10
already in mere's graph through `ssh-key` 0.6.7 (`cargo tree -i rsa`).

Added 2026-10-04 with rulings 54 to 58, three claims read: ssh-key 0.6.7's
`rsa` conversion passing `p` twice (`private/rsa.rs:195-203`), its ECDSA
scalar decoder (`private/ecdsa.rs:38-55`), and crates.io's index listing ssh-key
up to `0.7.0-rc.11` with `ssh-agent-lib` 0.6.0 requiring `^0.6`.

Added 2026-10-04 with ruling 59: aws-lc-rs 1.18.1's `KeyPairComponents`
requiring dP, dQ and qInv (`src/rsa/key.rs:103-125`) read in the registry.
