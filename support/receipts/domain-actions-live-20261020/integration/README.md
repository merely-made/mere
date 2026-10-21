# Canonical integration qualification

Source `3b73f15c8` reconciles the live handler checkpoint `c2db56559` with main
`1797a2eed`. All 93 focused tests pass again with locked/offline Rust 1.99.1,
debug information and incremental compilation disabled, and four build jobs.
The native host, resident and preparation helper were rebuilt independently.

The native host and helper match the original live artifacts byte for byte;
the resident differs after the upstream retained-recall change. The complete
native acceptance was therefore repeated with frozen rebuilt binaries: one
actual default-browser visit, one durable owner access-history entry, cold
native/resident reopen without redispatch, and a disabled owner setting that
removes the offer without changing history. Four new presented captures were
reviewed. `accepted/qualification.json` identifies their processes and binaries.

All 50 original receipt files still match their original manifest. This packet
adds evidence rather than replacing the earlier source or claiming its checks
ran on the new source. Compiler-declared repository inputs and the lockfile
are hashed in `repository-inputs.json.gz`; binaries remain outside Git.
Runtime acceptance is limited to macOS, retained native controls and the local
admitted route. Physical input, screen readers, other platforms and general
page-load/application completion remain unqualified.
