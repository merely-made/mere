# Accepted browser R6 shared recovery

These are compact copies of the accepted 2026-10-14 browser R6 receipts and all
twelve raw-derived PNGs. No test or browser was rerun to create this directory.
[Provenance](provenance.json) binds copied hashes, absolute external raw paths,
source/build/runtime identities and the larger artifacts retained outside Git.
The 29,388,144 raw RGBA bytes and the large Wasm/glue package are not copied here.

The qualified code is Genet `5618326d287e7f7ef0c1cc3545587d64cedb22a7`,
Mere `4682ba99040c234c6410d73125e5dc9b1f91cc03`,
Netrender `c74b7cb28090c28b1c6d80b08dcdea18aff48d3e` and
provider `2eae6243881d5b24c984d5ee35dde509ca5db782`. The R6 process-ownership driver
uses the unchanged R5 package and exact reused R4 Wasm/glue, under recorded
compiler-input equivalence. The original candidate-local Genet override and
lock remain recorded; later Git-only pin/merge integration is a separate gate.

Two actual full Workshop mounts share one core and use identical local preview
keys with distinct physical image identities. Actual Device::destroy emits
Destroyed for generation 2, both hosts detach, and the async owner adopts one
shared generation 3. The second host is application-hidden within a visible
page. Authored state, runner/Reader/Stylesheet Rc identity, focus/selection/zoom
and same host/canvas survive. Three armed captures refuse exactly once, queued
readback returns DeviceLost then AlreadyCollected, and later logical input plus
resize change state/pixels and extent. Actual canvas and mirror synthetic DOM
keyup Tab controls exercise owned listeners and preserve focus/application
authority; they are not OS keyboard or assistive-technology actions.

[Independent verification](independent-verification-r6.json) checks all twelve
complete raw images and decoded PNG equality, nonblank 13-color admission and
six full before/after comparisons. The maximum observed RGB difference is 1,
alpha 0 and mean absolute RGBA 1.9501730778606602e-5. Every pair passes unchanged
uniform RGB 3 / alpha 2 / mean 1 limits; these images are not byte-identical. Four
presented workspace copies and eight full actual physical Reader/Stylesheet
copies include clipped producer content without reconstruction, crop or matte.
[Root visual review](root-visual-review-r6.json) inspected all twelve PNGs; the
[coordinator review](coordinator-visual-review.json) independently inspected six
representative complete images and confirmed visible light/dark text/content.

The fixture deliberately maps the checked-in 168,260-byte Roboto face
SHA256 319cff6e7a31f0f2a41c475dca42890aa5d19fe16017e2290f8c1d4e14f76481 to all
four generics: sans-serif, system-ui, serif and monospace. Presentation seals
require actual outer materialized fonts/zero registration errors and positive
Reader/Stylesheet glyphs with exact font bytes. The five
[CPU font controls](font-cpu-qualification-r4.json) qualify explicit mapping,
atomic replacement and retained reconstruction. This proves resource delivery
and retention, not proper serif, fixed-width, script, icon or locale coverage.
Production consumers must choose and qualify suitable actual faces by role.

Both active mounts finish disposal with no listeners, RAF, pending startup or
cleanup. The exclusively owned replacement device and separate late-acquisition
control device are explicitly destroyed and their matching notices observed.
Rust weak handles expire after the observation handles are dropped. The
[inner](inner-runtime-release.json) and [outer](outer-runtime-release.json)
receipts independently close only owned Chrome/Node/server resources within the
original 60-second runtime/10-second cleanup/150-second outer bounds. The outer
completed in 50.38809433 seconds. The actual R6 run had zero page/console errors.

Browser admission observes AMD/gcn-5/nonfallback on the real current browser
adapter. Browser-exposed core identity may be redacted; the immutable native
Vega 56/Metal supplement is separately scoped and is not an exact browser model
claim. This is queue-presentation and raw-image visual evidence, not an OS
compositor screenshot, sustained performance or physical driver-reset proof.

Current-frame async shader quality, async GPU error scopes and final canvas
clear/blit scoping remain unqualified. R6 honestly records pending quality
Some((3, 96)); it does not claim zero counters. Both current-frame qualification
flags, physical_driver_reset and os_accessibility_actions remain false. The
separate detached QC controls have not been applied, compiled or executed.
Earlier R2/R4 input/font/console/cleanup negatives remain external and unchanged.
