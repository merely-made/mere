# Mere patch provenance

Published `burn-cubecl 0.22.0-pre.4`, Burn commit
`9147c11a19a5e2d46694c54d441eee0e006e096c`, rebased 2026-09-27.

Mere carries the existing BrowserWebGpu shared-input binary guard in the
numeric, integer and float launchers. A matching view binds its allocation
once, aliases the second input and writes a distinct output. Other inputs
retain upstream's in-place selection. The numeric and float guards stay
inside pre.4's `in_memory_order` closures, after its zero-output checks;
the integer launcher keeps its existing structure.

Rulings 355 and 377 move identity comparison into the private `same_view`
helper and add full service identity. Its six predicates compare allocation
descriptor ID, both optional offsets, stream, raw underlying size and service.
The same-device/different-service-type unit control calls this actual helper;
forged test handles are never sent to a GPU. Slice controls preserve distinct
offsets and the prior None-versus-Some(0) distinction.

Manifest-only additions isolate this patch as a workspace, apply Mere's
runtime persistence patch when checking it standalone, and directly name the
already-transitive common/environment crates for constructing test handles.
Upstream source beyond the three guards and private helper is preserved.

Root/product manifests remain on pre.2 at this bounded checkpoint. These
helper controls are not the full headed numerical or two-peer acceptance
receipts, which remain migration plan section 13 gates. Remove the launcher
patch when a released unpatched Burn/CubeCL row passes the headed shared-input
reproducer and its applicable controls.
