# cubek-matmul 0.3.1: packed Q4 unit-stage memory

Upstream: `tracel-ai/cubek`, commit
`245851cc83e95e179b91bc145c2b3a1e305bfdba`, `crates/cubek-matmul`.
The registry archive SHA-256 is
`11362e390ac15e89a9dad87645a0869e0fa73998c26fc62dfffbca62893b0f28`.
Sources and tests come from that archive. The licenses come from that commit's
repository root. Cargo.toml adds a standalone workspace for its patch
regression gate. CubeCL now resolves upstream; its defaults forwarding is
qualified separately. Its lockfile pins that standalone gate. Embedding hosts must restate this patch at their
own Cargo root; dependency manifests do not propagate Cargo patches.

Mere changes `src/multi_level/routines/selector/unit.rs`. The original selector
shrinks register partitions until its stage fits shared memory. Packed Q4 can
require an eight-wide tile. At 16 by 16 working units, even partitions of one
need 69,632 bytes, exceeding the 49,152-byte Vulkan limit on the qualified
RTX 4060 Laptop GPU. The selected backend then panics before generation.

After exhausting register partition reductions, the patch halves the larger
unit-stage axis until the footprint fits. It retains the tile required by
vectorization, whole GPU planes, and the existing unit-to-partition mapping.
The mapper derives its cube dimensions from the adjusted stage. Matrices stay
packed and their scales are applied in the kernel.

The pure selector regression pins the failing footprint and its reduction to
128 working units. ESP also tests 960-wide Q4 projections at 30-token prefill
and one-token decode shapes, alongside the full SmolLM2 format fixture.
Remove this patch when upstream selection passes these same rows and limits.

## 0.22.1 cohort rebase (2026-10-24)

Released archives were checksum-verified before a three-way rebase. Existing
local source and regression deltas are preserved, including changes added
after the original stable-release receipt. New upstream source changes are
retained. Selected execution gates pass in the Burn migration plan, section 13.48.
The bounded macOS receipt and remaining platform/model limits are recorded
there; the source rebase alone is not a wider acceptance claim.
