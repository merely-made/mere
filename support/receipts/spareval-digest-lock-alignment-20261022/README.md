# Spareval digest lock alignment — 2026-10-22

The current Web product build failed in spareval 0.2.7: md-5 and sha1 resolved to digest 0.11 while its sha2 edge resolved to 0.10.9/digest 0.10.7. Spareval imports the md5 Digest trait for all hash evaluators, so SHA-256/384/512 failed to type check before reaching Graphshell UI code.

Only the spareval sha2 edge changes in the root and standalone Web locks, to the already-resolved sha2 0.11.0/digest 0.11.3. The cached upstream manifest accepts `>=0.10,<0.12`; package/version/checksum sets are unchanged, with no manifest, supplier pin or registry edits. This prerequisite is separate from Schedule authoring.

The locked/offline native `mere-linked-data --features query --lib` gate passes all 104 tests. The locked/offline Web build gets through spareval after the correction; final product/UI qualification is retained with the Schedule pass. Rust 1.99.1, debug0, incremental0, four jobs. Red Web build, native green gate and exact two-edge diff are retained here; full output is under `/Users/markik/Code/output/mere-seiche-three-passes-20261022/schedule`.
