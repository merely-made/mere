# cubek-matmul 0.3.0: packed Q4 unit-stage memory

Upstream: `tracel-ai/cubek`, commit
`b6ac1f2ee54c773f345c0bfbd9f330c9022cc3a8`, `crates/cubek-matmul`.
The registry archive SHA-256 is
`59816f7d5baf51ca06f417899ee194d46d76f633541907c94091163b4ade20ff`.
Sources and tests come from that archive. The licenses come from that commit's
repository root. Cargo.toml adds a standalone workspace and the same local
cubecl-runtime patch used by Mere, for the patch regression gate.

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
