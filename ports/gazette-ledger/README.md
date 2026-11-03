# gazette-ledger

Gazette's Cambium side: the contact **Ledger** projection and the reading-room
article extractor.

- `ledger` projects contacts into a scene (`chirograph`, `incipit`, `sceno`).
  `tests/ledger_parity.rs` reproduces its receipts through `scenomise`'s matrix
  derivation and the frozen grid.
- `article_from_html` reads a readable article out of supplied HTML or XHTML
  through the static DOM and Fleece, without fetching or storing anything.

Gazette's directory domain, WebFinger discovery and the intake that hands what
it learns to `gaz`, is the `gazette` crate in dramatis (0.0.2) and links nothing
from mere. This crate was split from it on 2026-11-03 (dramatis repo plan, D13).
