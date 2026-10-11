# Earlier native GPU resets: evidence and limits

The earlier Pelt acceptance resets and the image-atlas defect qualified by this
receipt are separate findings. The reset reports establish recurring compute
queue and memory-fault symptoms; they do not identify an exact Vello kernel or
establish an infinite shader loop. This note summarizes a read-only comparison
of five archived macOS GPU reset reports, without publishing full OS reports or
private process paths.

## Archived reset pattern

Report-local timestamps and reported fault pages:

| Timestamp | Restart channel | GPU fault page |
| --- | --- | --- |
| 2026-10-09 22:18:40 | computeUQ6 | `0x405129000` |
| 2026-10-09 23:08:00 | computeUQ2 | `0x40512a000` |
| 2026-10-10 00:17:33 | computeUQ2 | `0x40507a000` |
| 2026-10-10 01:23:35 | computeUQ2 | `0x40507a000` |
| 2026-10-10 01:46:41 | computeUQ2 | `0x40507a000` |

All five reports associate the first pending Metal command buffer with the
acceptance product, across different process IDs. Each reports the same hash,
`f2d7d0c7d0d5d99c6327793f165377d [main_]`, and submission trace ID `0`.
Each also records a read protection fault with VMID `12`, failing protection
`VALID, READ`, and memory client `8 (TCP)`.

The hardware snapshots show the compute command processor busy while the
unified shader processor, texture pipe, and render backend are idle. In the
last report, graphics and DMA channels wait on the stalled compute channel;
the accompanying WindowServer watchdog record reports an unhealthy display.
These observations support the downstream display failure. They do not
establish whether the memory fault initiated the stall or which resource was
responsible.

Although the reports label a section "Shader Info," they provide no source
mapping, named WGPU pipeline, or usable shader disassembly. The associated
80-word dumps differ despite the common hash. Neither the anonymous `main_`
name nor that hash can defensibly be assigned to fine rasterization, coarse
rasterization, or another kernel from these reports alone.

## Repair timeline and qualification boundary

The latest archived reset occurred at 01:46:41 on October 10. Maintained Vello
`491c376cf2b01fc11132cf8f86419dec114ae032`, committed at 02:29:57 that day,
enabled normal shader runtime checks and workgroup memory initialization. It
also bounded fine-command operand reads, forward jumps, clip endings, and fill
segment spans. The archived resets therefore precede this guard repair. Its
deliberate malformed-command GPU regression passed on Radeon Pro Vega 56 /
Metal; that evidence does not retroactively identify the original failing
kernel.

The later atlas repair, `10f01d6d88e94eac087daf033b24895cf97b8e82`, preserves
those guards unchanged. Its separate defect has a pixel-exact reproduction:
a correctly rendered image becomes transparent after a solid-only frame
discards the persistent atlas while image residency remains clean. See the
[atlas qualification](README.md) and [original control](original-control/receipt.json).
This is evidence for an image-loss repair, not a demonstrated reset repair.
No new reset was observed in the qualified runs recorded by this receipt;
that finite-run observation does not establish the cause of the older resets.

## Diagnostic limits and smallest next step

Production NetRender submits coarse and fine dispatches together through
`render_to_texture`. Genet raster spans measure CPU preparation and submission,
not GPU completion. Vello consumes allocation-counter readbacks internally;
these receipts contain no durable counter or submission ledger. Optional
`wgpu-profiler` queries have shader labels, but no exported profiler results
identify a stalled stage here. A shader-validation run containing zero tests
is compilation evidence only.

The smallest useful future addition is an opt-in durable ledger recording
frame/raster identity, named dispatch labels, direct or indirect launches,
resource sizes, and submission/completion boundaries. Existing profiler
queries could supplement completed-stage timings. **This ledger is not
implemented.** A separate coarse/fine completion control would change
scheduling and would require deliberate bounded diagnostic qualification;
re-running older unchecked shaders is not necessary to preserve this evidence.
