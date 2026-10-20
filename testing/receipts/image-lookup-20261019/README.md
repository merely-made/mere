# Renderer image ownership adoption

Adopts NetRender d92050146768357ead4e4e42967866be7061262e and Genet 83114ac2d3dd323e6d9af8bb5e763a39ffd7f41e in the workspace and standalone web consumers. Other locked suppliers retain their previous revisions.

NetRender now shares CPU image lifetime policy between its Classic rasterizers and borrows image lookup inputs rather than repeatedly building merged maps. Image override registration invalidates only dependent fragment lowerings. This also repairs the baseline-reproduced moving filtered-fragment retention failures. The supplier receipt contains actual image/pattern pixel comparisons, override replacement/removal and retirement checks. No GPU-upload performance improvement is inferred.

The local receipt records focused consumer tests and standalone Wasm compilation separately. Compilation does not qualify browser presentation. Gates use one job, one test thread, a shared machine lock, 5 GiB memory cap, no additional swap, 150% CPU quota and a 600-second timeout.

Results: 89 document-canvas tests with NetRender enabled and six canvas culling tests passed, including actual GPU pixel comparison. Both standalone Wasm checks passed with default features and locked dependencies. The smoke example first required an offline refresh of its ignored lockfile; the failed pre-refresh check is retained in the receipt. A concurrent documentation-only main update was incorporated without changing the tested code.
