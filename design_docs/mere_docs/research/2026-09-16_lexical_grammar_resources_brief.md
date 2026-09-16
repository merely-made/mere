# Lexical data + grammar tooling — Assess-phase survey

**Status (2026-09-16):** research complete. Its open questions were answered the
same day and the resulting decisions are recorded in the
[lexical capability plan](../implementation_strategy/2026-09-16_lexical_capability_plan.md);
read the stacks and questions below as the options that were weighed, not as live
questions. The chosen shape is closest to Stack 3 plus Stack 2's data as an
optional add-on.

**Date of research: 2026-09-16.** Every licence, version and size below was checked
against a primary source (project site, repo, licence file, package registry API)
on this date unless the row says otherwise. Rows marked **[unverified]** are things
I could not confirm from a primary source today and should not be relied on.

Two pages that would normally be primary sources are behind Cloudflare and refused
automated fetch: `wordnet.princeton.edu/license-and-commercial-use` and
`wordnet.princeton.edu/download/current-version` (HTTP 403, browser UA included).
The Princeton licence text below is taken verbatim from `WNDB_License.txt` as
redistributed inside the Open English WordNet tree, which is the same text.

---

## The one factual correction up front

The **"Advanced English Dictionary"** Mark is thinking of is a *consumer app*, not a
dataset. Two different vendors ship one:

- MobiSystems, "Advanced English Dictionary & Thesaurus" — Android package id is
  literally `com.mobisystems.msdict.embedded.wireless.wordnet`, and MobiSystems runs
  a blog post titled "WordNet lexical database for MSDict". Its store copy describes
  WordNet's own structure (synsets, hypernyms, hyponyms, meronyms, ~140k entries).
- Cloudbit d.o.o., "English Dictionary + Thesaurus" (iOS), whose App Store
  description states outright: "Our app leverages the esteemed WordNet 3 database
  from Princeton University."

So: **the app is proprietary; the data under it is Princeton WordNet, which is open.**
There is nothing to license from the app vendor. What is open is WordNet itself — and
the live, maintained version of that is **Open English WordNet**, not Princeton 3.1,
which has been frozen since 2011.

---

## HALF A — English lexical data

### A1. Source dictionary (definitions / senses)

| Resource | What it gives | Licence | Size | Offline | Rust story | Caveat |
|---|---|---|---|---|---|---|
| **Open English WordNet (OEWN) 2025 / 2025+** | Synsets with glosses + examples; full semantic relation set. 2025+ = 120,564 synsets / 161,875 words / 419,226 semantic relations; core (no proper nouns) 107,519 synsets / 135,969 words | **CC BY 4.0** (its `WNDB_License.txt` carries the historical Princeton notice alongside) | Gzipped: WNDB 9.2 MB, WN-LMF XML 10.8 MB, RDF/Turtle 16.9 MB, JSON 9.5 MB (2025+: 10.5/12.3/19.4/10.8 MB) | Yes — flat file | No serious crate. Parse WN-LMF XML or JSON yourself into SQLite. `wordnet-lmf` crate exists but is v0.1.0 / 2021 | Annual cadence: 2019…2024-11-01, 2025 + 2025+ both 2025-12-31. Glosses are *short definitional glosses*, not lexicographic prose |
| **Princeton WordNet 3.1** | Same shape, 2011 vintage | Princeton WordNet licence — BSD-ish permissive: "Permission to use, copy, modify and distribute … for any purpose and without fee or royalty is hereby granted", attribution required on ALL copies, Princeton's name may not be used in advertising | ~10 MB WNDB | Yes | Several hobby crates (see A-Rust note) | **Frozen since 2011.** OEWN is the maintained successor and is a strict superset in practice. No reason to start here |
| **Kaikki.org / wiktextract (English)** | Full Wiktionary entries as JSONL: senses, glosses, forms, pronunciation, translations, linkages, **etymology_text + etymology_templates** | Data is Wiktionary-derived → **CC BY-SA 4.0** (dual GFDL) per Wikimedia ToU. Kaikki's own download page states no licence; wiktextract *code* is MIT | **23.1 GB uncompressed / 2.7 GB gz**; 1,390,507 distinct words. Extracted 2026-09-09 from the 2026-09-02 enwiktionary dump | Yes | No crate. It is JSONL — stream with `serde_json` into SQLite. Straightforward but you are writing the schema | Weekly-ish refresh, genuinely current. Huge. Quality is Wiktionary quality: excellent coverage, uneven register, share-alike |
| **DBnary** | Wiktionary as RDF/ontolex-lemon: lexical entries, senses, translations, lexical relations. 26–27 Wiktionary editions; ~7.9M lexical entries / 6.5M senses / 12.3M translation pairs across all languages | **CC BY-SA 3.0** (dataset); extractor is MIT | RDF/Turtle dumps per language edition | Yes | None. You would ingest Turtle — no good pure-Rust RDF story at this size; realistically convert with Python/Java once, ship SQLite | Alive: site shows July/August 2026 extracts, "evolves twice a month". Best *multilingual* option. **[unverified]** whether it extracts etymology — I found no etymology module documented |
| **GCIDE (Webster's 1913 + WordNet additions)** | Full lexicographic definitions in prose — the thing WordNet glosses are not | **GPL** (GNU project) | v0.54, released **2024-12-31**, 18 MB tar.gz / 14 MB tar.xz | Yes | None. Custom SGML-ish markup, needs a bespoke parser | **GPL on the data is the hard trap** (see traps). Also: it is 1913 English. Charming, frequently archaic, and wrong about the modern world |

**Rust note for A1:** the crates.io WordNet ecosystem is thin and hobbyist. The
plausible-looking family — `wordnet-db` (memory-mapped reader), `wordnet-types`,
`wordnet-morphy`, all MIT OR Apache-2.0, all v0.1.3, all last published 2026-01-01 —
turns out to belong to one user (`johanneswd`) and point at a repo called
`crosswordsolver`. Download counts are ~2k lifetime. `wordnet` (2017), `wn` (2021),
`wn-parser` (2025), `thesaurus`/`thesaurus-wordnet` (2022, ~14k downloads) round it
out. **There is no maintained, serious Rust WordNet crate.** The mature library is
Python `wn` (MIT, SQLite-backed, `python -m wn download oewn:2025+`) — which is a
perfectly good *build-time* tool: use it once to produce a SQLite file and ship that.

---

### A2. Word parts (morphology — roots, affixes, segmentation)

| Resource | What it gives | Licence | Size | Offline | Rust story | Caveat |
|---|---|---|---|---|---|---|
| **MorphyNet** | Derivational + inflectional morphology + morpheme segmentation, 15 languages. English: 396,772 lemmas, 649,594 inflected forms, **67,412 derivational pairs** | **CC BY-SA 3.0** per README; GitHub detects no SPDX licence file | TSV per language, small (tens of MB) | Yes | None needed — plain TSV, trivial to ingest | Repo static since **2023-04-02**. Wiktionary-derived, so share-alike and Wiktionary-quality. This is the best open *derivational* data for English |
| **MorphoLex-en** | 68,624 English words with root/prefix/suffix decomposition + 9 psycholinguistic variables, from the English Lexicon Project | **CC BY-NC-SA 4.0 — NON-COMMERCIAL** | ~6.4 MB repo, single `MorphoLEX_en.xlsx` | Yes | Would need xlsx→csv at build time | **Unusable in any product you might sell or even arguably commercialise.** GitHub reports the licence as NOASSERTION; the LICENSE.md file is plainly CC BY-NC-SA 4.0. Repo static since 2022-01 |
| **UniMorph** | Inflectional morphology only (lemma + feature bundle), **169 languages** | Mostly **CC BY-SA 3.0**; some languages LGPL-LR | English repo ~20 MB | Yes | Plain TSV | Inflection, *not* derivation and *not* segmentation. English repo static since **2023-02**. No decomposition into roots/affixes |
| **Hunspell affix tables (.aff/.dic)** | Affix-stripping rules; Hunspell can do morphological analysis and generation (`-m` / stem output), 64k affix classes | Engine: **LGPL/GPL/MPL tri-licence**. *Dictionaries are separately licensed per language* | Per-language, small (en_US ~1 MB) | Yes | **Best Rust story in this whole survey** — `spellbook` (MPL-2.0, pure Rust, `no_std`, a Rust rewrite of Nuspell, only depends on hashbrown), `zspell`, or FFI via `hunspell-rs` | The English dictionary comes from **SCOWL**, which is permissive (Kevin Atkinson's BSD-style grant + public-domain Moby components). Affix tables are a *spellchecker* artefact — they approximate morphology, they do not encode etymological roots |
| **Morfessor 2.0** | Unsupervised/semi-supervised morphological segmentation — an *algorithm*, not a dataset | **BSD-2-Clause** | Python package | Yes | None. You would run it offline to generate data | Gives you statistical segments, not linguistically-named roots and affixes. Useful as a fallback for OOV words, not as a source of truth |
| **CELEX2** | The gold standard: orthography, phonology, morphology, syntax, frequency for English/Dutch/German | **NOT OPEN.** LDC-distributed under a CELEX User Agreement, fee-based (LDC membership or member purchase) | — | — | — | Rule it out and stop thinking about it. It is the resource everyone in the literature uses and nobody can ship |
| **OEWN derivational relations** | WordNet's own `derivationally related form` sense pointers | CC BY 4.0 (comes free with A1) | included | Yes | included | Not decomposition — it links *pairs* (`decide`↔`decision`), it does not tell you `de-` + `-cide`. Still, it is permissive and you get it for nothing |

---

### A3. Etymological relations

Honest framing first: **this layer is the weakest of the three by a wide margin.**
Everything open is machine-extracted from Wiktionary, and Wiktionary etymology is
prose written by volunteers in wildly varying depth. Extraction turns "from Latin
*decidere*, from *de-* + *caedere*" into edges; it drops nuance, misparses, and has
no confidence signal. Coverage skews hard toward Latin/Greek/Germanic roots of
"interesting" words and thins out fast.

| Resource | What it gives | Licence | Size | Offline | Rust story | Caveat |
|---|---|---|---|---|---|---|
| **Etymological Wordnet** (de Melo) | Word-origin edges across many languages (`rel:etymology`, `rel:derived`, etc.) | **CC BY-SA 3.0** | 26.5 MB archive | Yes | Plain TSV | **The data snapshot is `etymwn-20130208` — February 2013.** Paper is LREC 2014. It has not been refreshed in ~13 years. The canonical homepage `etym.org` did not respond today. This is a fossil; useful as a baseline, not as a product dependency |
| **EtymDB 2.1** (Sagot & Fourrier) | 1.8M lexemes, **>700,000 fine-grained etymological relations**, 2,536 living and dead languages, typed relations (inherited/borrowed/derived) | **CC BY-SA 4.0** (repo LICENSE), with LGPL-LR also referenced | ~26 MB repo, CSV (`etymdb.csv` + split lexeme/link files) | Yes | Plain CSV | Repo static since **2022-01-04**. Better typed and bigger than etymwn, still Wiktionary-derived, still a snapshot. The best of the pre-built etymology sets |
| **wiktextract / Kaikki `etymology_text` + `etymology_templates`** | Raw etymology prose *and* the structured templates behind it — including `{{affix}}`/`{{prefix}}`/`{{suffix}}`/`{{compound}}`, which is **also a word-parts source** | CC BY-SA 4.0 (Wiktionary) | part of the 2.7 GB gz | Yes | You build the extractor | **The only genuinely current option** (2026-09-09 extraction). You do the etymology-graph construction yourself, which is real work, but you control quality and you can refresh weekly. This is where the `etymology_templates` field quietly doubles as morphological decomposition |
| **Wikidata Lexemes** | Hand-curated lexemes, forms, senses, with `derived from` / `combines` statements | **CC0** — the only public-domain option anywhere in this survey | Full dump is large; lexeme subset is extractable | Yes | JSON dump; you build it | **Coverage for English etymology is thin.** CC0 is extremely attractive and the data is genuinely curated rather than scraped, but volume is nowhere near Wiktionary's. Worth a coverage spike before betting on it. **[unverified]** — I did not pull current English lexeme counts today |

---

## HALF B — Grammar and other languages

### B1. Open-source Grammarly alternatives

| Resource | What it gives | Licence | Size | Offline | Rust story | Caveat |
|---|---|---|---|---|---|---|
| **LanguageTool** | The real Grammarly competitor. **42 languages** (v6.6 docs), English 6,074 XML rules + 686 confusion pairs; French 6,984; Catalan 8,251; German 5,224 | **Core is LGPL-2.1-or-later** ("The LanguageTool core – is distributed under the LGPL"). Weak copyleft: linking it does not infect your app | JVM app; + **~8 GB optional n-gram data** | Yes, fully self-hostable | Java. Call it over its HTTP API as a sidecar, or shell out | See the split below. Latest tag **v6.8, 2026-05-05** |
| — LanguageTool n-gram data | Confusion-pair disambiguation (their/there) | **[unverified]** — the docs page does not state a licence; it says the set comes "from Google" (Google Books Ngram-like). Google Books Ngrams themselves are CC BY 3.0 | **~8 GB total**, four languages only: en, de, fr, es | Yes | — | Docs warn it needs an SSD or LanguageTool slows badly. Licence needs pinning down before shipping |
| — LanguageTool Premium | AI paraphrasing, "picky" style rules, stronger en/de detection | **Proprietary.** Open-core model; premium is a separate closed sister product running server-side models | cloud | No | — | **Also closed: every official integration except the OpenOffice/LibreOffice plugins.** Browser extensions and mail plugins are proprietary. The self-hosted server gets the same *core* rule sets for all 25+/42 languages; what you cannot replicate locally is the AI rewriting |
| **Harper** (Automattic) | Offline, privacy-first grammar checker. Claims <1/50th LanguageTool's memory, millisecond lints | **Apache-2.0** | `harper-core` crate; static dictionary compiled into the binary; small enough to ship as WASM | **Yes, by design — nothing leaves the machine** | **Native Rust.** `harper-core` 2.10.0 published **2026-09-10**; release v2.10.0 **2026-09-09**; ~15.4k recent / 112.8k lifetime downloads. Also `harper.js` (npm), `harper-ls` (LSP), CLI | **English only.** README: "the core is extensible to support other languages, so we welcome contributions." Rule coverage is far below LanguageTool's 6,000+ English rules. **[unverified]** — dictionary provenance/licence is not documented |
| **nlprule** | Rust port of LanguageTool's rule engine, en/de (+experimental es) | Code **MIT OR Apache-2.0**; the compiled `*.bin` rule data is **LGPL-2.1**, derived from **LanguageTool v5.2** | crate | Yes | Native Rust | **Effectively dead: last push 2023-05-23, 27 open issues.** Its rule data is pinned to LT 5.2 while LT is now 6.8. Do not build on it; it is a useful *reference implementation* of how to compile LT rules into Rust |
| **Vale** | Markup-aware *prose style* linter — terminology, passive voice, house style | **MIT** | Go binary, v3 | Yes | Shell out | Style, **not grammar**. Different product. Genuinely good at enforcing a style guide |
| **write-good** | Naive English prose linter: passive voice, weasel words, wordy phrases, clichés, E-Prime | **MIT** | tiny npm module | Yes | Reimplementable in an afternoon | Naive by its own admission. ~157 commits; maintenance level low |
| **proselint** | 100+ prose checks: uncomparables, archaisms, clichés, malapropisms, jargon, sexist language | **BSD-3-Clause** | Python | Yes | Shell out | Style/usage, not grammar. Actively maintained (CI, pre-commit at 0.16.0) |
| **textlint** | Pluggable natural-language linter, ESLint-shaped. 100+ community rules; md/txt built in, html/rst/asciidoc/org via plugins | **MIT** | Node ≥20 | Yes | Sidecar | **No bundled rules** — it is a framework, you assemble the ruleset |
| **Hunspell** | Spelling + morphological analysis/generation; 64k affix classes, twofold affix stripping | **LGPL/GPL/MPL tri-licence** (pick one) | small | Yes | `hunspell-rs` FFI | Dictionaries licensed separately per language. The tri-licence means you can take MPL and avoid copyleft concerns entirely |
| **Nuspell** | Modern C++17 rewrite of Hunspell, claims up to 3.5× faster, reads Hunspell dictionaries | **LGPL-3.0-or-later / GPL-3.0** | small | Yes | FFI, or use `spellbook` instead | **LGPL-3/GPL-3 is stricter than Hunspell's tri-licence.** If you want Nuspell's behaviour without its licence, `spellbook` is the Rust rewrite under MPL-2.0 |
| **Morfologik** | Finite-state-automaton morphological dictionaries + stemming | **BSD-3-Clause** | Java | Yes | Java | Polish-centric origin. **[unverified]** — I could not confirm from its own repo that LanguageTool uses it, though LanguageTool's spellchecking is widely said to be Morfologik-based |

### B2. Multilingual grammar / morphology infrastructure

| Resource | What it gives | Licence | Size | Offline | Rust story | Caveat |
|---|---|---|---|---|---|---|
| **Universal Dependencies** | Consistent POS + morphological features + dependency syntax annotation across languages. **v2.18, released 2026-05-15**; 200+ treebanks, 150+ languages, 600+ contributors | **Not uniform — per-treebank.** The set mixes CC BY-SA, CC BY, **CC BY-NC-SA**, GPL and others | large (CoNLL-U text) | Yes | CoNLL-U is trivially parseable | **The per-treebank licence patchwork is a genuine trap.** Some treebanks are non-commercial. You must audit treebank-by-treebank for the languages you actually ship, and that audit does not generalise across releases |
| **spaCy models** | Tokenisation, POS, dependency parse, NER; **25 languages** with trained pipelines, language data for 70+ more | Library MIT. **Models carry their own licence** — `en_core_web_sm` 3.8.0 is **MIT**, 12 MB | 12 MB (sm) up | Yes — installable from a URL or local dir, fully offline-deployable | Python. Build-time only, or sidecar | **The interesting bit:** `en_core_web_sm`'s declared sources are OntoNotes 5 (*"commercial (licensed by Explosion)"*), ClearNLP conversion, and WordNet 3.0. The *weights* ship MIT; the *corpus* behind them is not redistributable. Fine for use, but you cannot retrain from the same data |
| **Stanza** (Stanford) | Neural pipeline: tokenise, lemmatise, POS, depparse, NER. **70+ languages**, UD formalism | Code **Apache-2.0**; the site does not state a distinct model licence | PyTorch models, per-language | Yes after model download | Python + PyTorch. Too heavy to embed | Models are trained on UD treebanks, so the UD per-treebank licence question arguably propagates. **[unverified]** — model licensing is not separately documented and deserves a direct question to the maintainers if it matters |
| **Grammatical Framework** | Multilingual *generation and parsing* from an abstract syntax. Resource Grammar Library covers **40+ languages**. v3.12 released 2025-08-08 | **Compiler GPL**; libraries LGPL and BSD | — | Yes | **No Rust runtime.** Runtimes: Haskell, Java/Android, JavaScript, C, Python | A different kind of tool entirely — it is for *building* correct multilingual text, not for correcting user text. GPL compiler matters only if you distribute the compiler |
| **Apertium** | Rule-based MT platform; **51 stable released language pairs**. `lttoolbox` = FST morphological analysis/generation (`cats` → `cat<n><pl>`), usable standalone | `lttoolbox` is **GPL-2.0** | per-pair dictionaries | Yes | C++ API / CLI; GPL | **GPL-2.0 on lttoolbox.** The morphological dictionaries are the valuable part and they are excellent for less-resourced languages, but the toolchain is copyleft |

### B3. What is actually Rust-native

Short list, because it is short:

- **`harper-core` / `harper-ls` / `harper.js`** — Apache-2.0, English grammar checking,
  active this month (2.10.0, 2026-09-09/10). The only credible Rust-native grammar checker.
- **`spellbook`** — MPL-2.0, pure Rust, `no_std`, one dependency, Hunspell-dictionary
  compatible, Nuspell-equivalent suggestions. The right spelling primitive.
- **`zspell`** — pure Rust Hunspell-format spellchecker, alternative to spellbook.
- **`hunspell-rs`** — FFI bindings; cannot change dictionary at runtime.
- **`nlprule`** — dead since 2023, LGPL rule data pinned to LT 5.2. Reference only.
- **`cargo-spellcheck`** — dev tool, uses hunspell and/or nlprule.
- WordNet crates — all hobbyist, none maintained, none worth adopting.

Everything else in this survey is Java, Python, Go, C++ or Haskell, and the realistic
integration pattern is **convert once at build time, ship a SQLite/FST artefact,
read it from Rust.**

---

## Licence traps (the section to read twice)

1. **Wiktionary is share-alike, and it is the substrate under almost everything.**
   Kaikki/wiktextract, DBnary, MorphyNet, Etymological Wordnet and EtymDB are *all*
   Wiktionary-derived and all land on CC BY-SA (3.0 or 4.0). If share-alike is
   unacceptable, **you lose the entire etymology layer and most of the word-parts
   layer at a stroke** — and you are left with WordNet's derivational pointers, which
   are pair-links rather than decomposition. This is the single structural fact that
   shapes the whole decision.
2. **MorphoLex-en is CC BY-NC-SA 4.0 — non-commercial.** It is the nicest-looking
   morphological decomposition dataset for English and it cannot go in a product.
   GitHub's API reports its licence as `NOASSERTION`, so a naive licence scan will
   miss this. Verified by reading `LICENSE.md` directly.
3. **GCIDE is GPL on the data.** Webster 1913 the *text* is public domain, but the
   GCIDE markup and corrections are a GNU project under GPL. Embedding GCIDE in a
   distributed application raises a copyleft question about the application, not just
   the data file. This is materially worse than CC BY-SA, which reaches the data and
   adaptations of it but not the program that reads it.
4. **Universal Dependencies has no single licence.** Per-treebank, and the mix
   includes CC BY-NC-SA. Any claim of the form "we use UD, it's open" is unsound
   until audited per language and per release.
5. **spaCy model weights are MIT but their training corpora are not.** `en_core_web_sm`
   lists OntoNotes 5 as "commercial (licensed by Explosion)". Using the model is fine;
   reproducing or retraining it is not.
6. **nlprule's rule data is LGPL-2.1**, separate from its MIT/Apache-2.0 code, and is
   frozen at LanguageTool 5.2. Mixing licences within one crate is exactly the thing
   that gets missed.
7. **Nuspell is LGPL-3/GPL-3, Hunspell is LGPL/GPL/MPL tri-licensed.** If you FFI to
   a spellchecker, Hunspell-under-MPL or Rust `spellbook`-under-MPL are the clean
   choices; Nuspell is not.
8. **Apertium's `lttoolbox` is GPL-2.0.** Its morphological FSTs are the best open
   resource for many smaller languages and they come with copyleft attached.
9. **LanguageTool's n-gram data has no stated licence on its own docs page.** ~8 GB,
   described only as "this data set from Google". Pin this down before shipping it.
10. **Princeton WordNet's licence forbids using Princeton's name in advertising** and
    requires the notice on *all* copies including internal modifications. Trivial to
    comply with, easy to forget. OEWN's CC BY 4.0 is the cleaner vehicle anyway.

---

## What I would pick and why

Three coherent stacks. These are options to choose between, not a recommendation —
the choice turns on the open questions below, which are Mark's to answer.

### Stack 1 — Permissive-only (ships in a closed or permissively-licensed product)

- **Senses:** Open English WordNet 2025+ (CC BY 4.0). Convert WN-LMF XML → SQLite once
  at build time; ship the SQLite.
- **Word parts:** OEWN's own derivational relations (free, permissive) + **Hunspell
  affix tables with SCOWL English dictionaries** (permissive) read via **`spellbook`**
  (MPL-2.0, pure Rust).
- **Etymology:** **Wikidata Lexemes (CC0)** only — and accept that coverage is thin.
- **Grammar:** **Harper** (Apache-2.0, Rust-native, offline).
- **Cost:** the etymology layer is weak-to-absent and word parts are approximated by
  spellchecker affix rules rather than real morphological decomposition. You get two
  of the three layers Mark named, properly, and the third barely.

### Stack 2 — Share-alike tolerant (the data layers are actually good)

- **Senses:** OEWN (CC BY 4.0) as the spine, **Kaikki/wiktextract** (CC BY-SA 4.0)
  as the coverage layer for everything WordNet lacks.
- **Word parts:** **MorphyNet** English (CC BY-SA 3.0) — 67,412 derivational pairs +
  segmentation — plus Wiktionary's `etymology_templates` (`affix`/`prefix`/`suffix`)
  from the same Kaikki dump you already have.
- **Etymology:** **EtymDB 2.1** (CC BY-SA 4.0) as a ready-made baseline, with your own
  extraction from the current Kaikki dump layered over it for freshness.
- **Grammar:** **LanguageTool self-hosted** (LGPL-2.1, 42 languages) as a sidecar,
  with Harper for the fast local path.
- **Cost:** everything derived from these datasets is share-alike. Acceptable if the
  data ships as a separately-identified artefact; a real problem if the data is fused
  into a proprietary index you do not want to redistribute.
- **This is the only stack that actually delivers all three layers Mark named.**

### Stack 3 — Offline-embedded, minimum footprint (Rust end to end)

- **Senses:** OEWN → SQLite, ~10 MB.
- **Word parts:** SCOWL/Hunspell `.aff`/`.dic` via `spellbook` (`no_std`, one dep).
- **Etymology:** omit, or ship EtymDB as an optional downloadable pack so the
  share-alike obligation attaches to a clearly separable file.
- **Grammar:** **Harper** only — Apache-2.0, WASM-capable, English only, milliseconds.
- **Cost:** English only, thin grammar rules relative to LanguageTool, no etymology in
  the base install. Buys you: no JVM, no Python, no network, one binary.

**The honest summary across all three:** senses are a solved problem (OEWN, permissive,
maintained, small). Word parts are adequate but share-alike. Etymology is the weak
link — nothing open is both current and permissive, and the best available data is a
2022 snapshot of a 2013-era approach unless you build your own extractor over the
weekly Wiktionary dump.

---

## Open questions — only Mark can answer these

1. **What product surface is this for?** A reader/lookup tool, a writing assistant, a
   graph/canvas exploration surface, a library for other repos in the workspace? The
   answer changes which of the three layers is load-bearing and whether the grammar
   half is even in scope for v1.
2. **Is share-alike acceptable?** This is the fork in the road. "Yes" gives you
   MorphyNet + EtymDB + Kaikki and all three layers properly. "No" collapses the
   etymology layer to CC0 Wikidata Lexemes with thin coverage, and reduces word parts
   to affix-table approximation. There is no middle path that preserves quality.
3. **Does it need to work fully offline, and at what footprint budget?** ~10 MB (OEWN
   only), ~100 MB (OEWN + MorphyNet + EtymDB), or multi-GB (full Kaikki, or
   LanguageTool's 8 GB n-grams)? The n-gram set alone would dominate any install.
4. **Which languages beyond English?** English-only makes Harper viable and keeps the
   data small. Multilingual forces DBnary (CC BY-SA) for lexical data and LanguageTool
   (LGPL, JVM sidecar) for grammar, and drags in the UD per-treebank licence audit.
5. **Is a non-Rust sidecar acceptable at runtime, or must everything be in-process
   Rust?** LanguageTool is the only mature multilingual grammar checker and it is a
   JVM process. Harper is the only Rust-native one and it is English-only with far
   fewer rules. There is no third option today.
6. **Is a build-time conversion pipeline acceptable?** Every good dataset here has a
   mature Python/Java ingestion tool and no Rust one. Converting once to SQLite/FST at
   build time is by far the cheapest path, but it means the repo carries a
   non-Rust build dependency.
7. **How much etymology quality is enough?** Machine-extracted Wiktionary etymology is
   patchy and has no confidence signal. Is "roughly right for common words, silent for
   the rest" acceptable, or does this need to be defensible? If the latter, the real
   answer is that no open dataset meets that bar and the layer needs either curation
   effort or a licensed source.
8. **Does this belong in an existing repo or a new one?** Out of scope for me to
   propose, and it interacts with question 1.
