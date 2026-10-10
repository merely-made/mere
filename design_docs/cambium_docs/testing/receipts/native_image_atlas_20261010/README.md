# Interleaved image atlas qualification, 2026-10-10

Maintained Vello `10f01d6d88e94eac087daf033b24895cf97b8e82` preserves the
resident image atlas when rendering a solid-only or empty scene. The resolver
begins the resource-cache pass before returning retained dimensions, clearing
prior upload records while leaving pending override refreshes intact.

The original `491c376c` same-device control renders an opaque four-color source,
registers it, renders the image, renders a patchless solid scene, then samples
the same image under two fresh raster identities. Both later samples lose all
65,536 pixels. Original raw copies, PNGs, exact comparisons, source/build
provenance and process timing are in `original-control`. The bounded process
exited in 0.859 seconds on Radeon Pro Vega 56 / Metal; mappings took 12–14 ms.

The CPU regression rejects the original resolver with extent `(0, 0)` instead
of its resident `(2048, 2048)` extent. With the fix, all 33 encoding tests pass,
including pending override refresh across the fast path. The new real-adapter
regression checks all pixels of 13 image/solid/empty frames through one Renderer
and passes in 0.81 seconds. The existing repeated clipped-preview test also
passes in 1.20 seconds, preserving the prior six-frame helper behavior. Logs
identify Radeon Pro Vega 56 / Metal with GPU computation enabled.

These receipts qualify this reproducible atlas defect. They do not identify
the exact kernel responsible for the earlier Pelt GPU reset, and do not replace
an application's native window, workflow or separate-process reopen acceptance.

The [earlier native reset analysis](original_reset_analysis.md) records the
archived fault pattern and the limits of kernel attribution separately.
