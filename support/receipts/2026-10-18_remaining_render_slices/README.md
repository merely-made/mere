# Completion of interrupted rendering test runs

Pictograph: **535 passed, 18 existing ignored, zero failures** on Mere `dd9d5c523c092baf72be31cdbc1d28c4ae179fa0`. Tabard Workshop: **14 passed, zero failures/ignores**. The [receipt](receipt.json) records exact commands, all test dispositions, source hashes, log hashes, and resource measurements.

The checkout was fast-forwarded to current main and the original package/feature graph rebuilt before testing. The spiral probe passed alone in 55.82 seconds. The remaining tests ran in serial slices of at most 30 tests, reduced to eight for simulation tests. The GPU culling comparison performed actual draw/readback and checked that removing a crossing edge changed the pixels. No Mere source repair was required.

Every gate kept the shared machine lock, 5 GiB memory cap, zero swap allowance, 150% CPU quota, nice 10, one Cargo job/test thread and 600-second timeout. No timeout recurred. Existing ignored probes remain outside this qualification; headed/browser and other platform execution are not added. Earlier completed suites retain their original source provenance, rather than being relabeled as tested on newer main.

Genet's interrupted scripted suite also has all 117 tests covered across its recorded runs. Two failures came from child-frame fixtures using the unsupported default inset border. Five rendering fixtures now author solid borders and assert successful paint; the existing refusal/recovery regression still passes. Genet commit `6f00922bb422d4449f55713b5c1e2fcd178c5e75` publishes that test-only correction and its receipt. NetRender remains the border-admission owner; no consumer-specific fallback or duplicate policy was added.
