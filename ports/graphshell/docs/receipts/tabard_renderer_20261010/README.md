# Tabard appearance with the guarded shared renderer

Chrome 152 on the real AMD gcn-5 adapter (no fallback) passes the production appearance modes, authored CSS/custom mode, corrupt-input preservation, fresh reopen, embedded-host ownership, saved-tree and main-control workflows. The fixed default Wasm build consumes Genet `7422e906` and Vello `491c376c`; acceptance.json records exact binary hash and adapter.

The three existing product scenarios pass 97 frames and 8 internal captures. Together with browser, tree and main-control screenshots, 23 original PNGs were visually reviewed and have nonblank pixel ledgers. Product-owned storage remains unchanged by appearance selection. No new GPU reset appeared. Original full scenario receipts and review contact sheets remain under tabard-app-receipts/2026-10-10/graphshell/fixed-renderer-01 in the repository-family workspace.

The original October 9 acceptance preceded `491c376c`; these October 10 receipts separately qualify that renderer. Browser paint and persistence evidence does not establish native Windows/Linux behavior or formal contrast conformance.
