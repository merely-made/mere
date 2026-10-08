# Mere stable CubeCL runtime patch

Source: published `cubecl-runtime 0.11.0`, CubeCL commit
`52c5086d3c5e73e56b3ee4cb711bdda495a360f0`.
License: MIT OR Apache-2.0, unchanged from upstream; both licenses are in-tree.

The only functional change is removal of `persistence` from the default
features in `Cargo.toml` and `Cargo.toml.orig`, carrying Mere ruling 375
forward to stable. Rust source is pristine upstream. Within-process caches
remain; cross-process autotune/throughput and compiled-kernel persistence
require an explicit feature request.

Remove this patch when upstream makes persistence optional without a forcing
default dependency edge. This source rebase is not a remote lifecycle receipt.
