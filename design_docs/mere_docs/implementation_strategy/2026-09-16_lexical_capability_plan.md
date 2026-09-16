# Lexical Capability Plan — senses, word parts, etymology, and writing checks

**Status (2026-09-16):** accepted; not started. New objective, raised by Mark
2026-09-16. Nothing implemented. The verified survey behind it, with licences,
sizes and formats per dataset, is the
[lexical and grammar resources brief](../research/2026-09-16_lexical_grammar_resources_brief.md).

## Objective

A shared Mere capability that answers, for a word in front of the reader or
writer: what it means, what it is made of, where it came from, and whether the
sentence around it is well formed. Knot and Turnstone consume it; neither keeps
its own copy.

## Decisions already taken (2026-09-16)

1. **Surface: a shared capability in Mere**, consumed by both apps, matching how
   Micron and forms were done and honouring the no-app-local-copies rule.
2. **Licensing: permissive core, share-alike as an optional add-on.** Open
   English WordNet (CC BY 4.0) ships in the product. The Wiktionary-derived
   etymology and word-parts data (CC BY-SA) is a separate artifact the user opts
   into, so share-alike never enters the shipped binary.
3. **Languages: English now, designed for more later.** The API takes a language
   tag from the first commit even while only English resolves, so a second
   language is additive rather than a rewrite.

## What the survey established

- The "Advanced English Dictionary" app is proprietary; the data under it is
  WordNet. Princeton's WordNet has been frozen since 2011. The live artifact is
  **Open English WordNet**, CC BY 4.0, roughly 10 MB gzipped, annual releases,
  120k synsets and 419k semantic relations. The senses layer is a solved problem.
- **Word parts**: MorphyNet gives 67k English derivational pairs plus
  segmentation as plain TSV (CC BY-SA 3.0). Wiktionary's affix templates are
  structured decomposition and come through the same extraction as etymology.
  EtymDB 2.1 (CC BY-SA 4.0, typed relations, static since 2022) is a candidate
  baseline for etymology, with a fresh extraction layered over it.
- **Etymology is the weak layer.** Everything open is machine-extracted from
  Wiktionary with no confidence signal. The purpose-built Etymological Wordnet's
  data snapshot is 2013; EtymDB is better typed but static since 2022. The only
  current path is extracting from the weekly Wiktionary dump.
- **Traps**: MorphoLex is non-commercial; GCIDE puts GPL on the data itself;
  Universal Dependencies licences are per-treebank and include non-commercial;
  LanguageTool's large n-gram data states no licence.
- **Writing checks**: Harper is Apache-2.0, native Rust, offline, small, English
  only. LanguageTool is the multilingual answer but is a JVM service with a
  paid cloud split. Decision 3 means Harper now, and no sidecar.

## Phases and done-conditions

### L1. Ingestion, offline and reproducible

A Rust conversion from each upstream artifact's published format (JSON for Open
English WordNet, JSON Lines for the Wiktionary extraction, TSV for MorphyNet) to
one compact queryable on-disk form, with the upstream version, URL and checksum
recorded per dataset, the way the Micron capture manifests record theirs.
Streaming readers, so the multi-GB extraction is never held in memory. No Python
or Java in the toolchain. Conversion runs on demand and is not part of an
ordinary build. No network access at runtime, ever.

Done when a named upstream release converts byte-reproducibly on two runs, the
manifest records version and checksum per dataset, and the converted artifact
loads in a test without network.

### L2. The query API in Mere

One crate owning senses, relations, morphology and etymology behind an interface
that takes a language tag and a word, and returns typed results with a
provenance marker per fact naming which dataset it came from. Absent layers
answer "not available" rather than guessing, which matters because the optional
add-on may be missing.

Done when the API resolves senses and relations from the permissive core alone,
reports the add-on layers as unavailable when absent, carries provenance on
every returned fact, and its tests run offline against a small committed sample
rather than the full dataset.

### L3. Writing checks

Harper behind a thin Mere seam, so the checker is swappable and the apps never
depend on it directly. Offline, English, no sidecar process.

Done when a passage with known errors yields stable, positioned findings through
the seam, and swapping the implementation out is a one-file change proven by a
second stub implementation in tests.

### L4. Consumers

Turnstone: look up a word on the page being read. Knot: the same lookup plus
writing checks while authoring. Both through the Mere API, no local copies.

Done when both apps show senses, parts and etymology for a selected word with
provenance visible, degrade correctly when the add-on is absent, and each app's
plan records the commands.

### L5. The optional add-on

Acquisition, verification and removal of the share-alike artifact as an explicit
user action, with its licence and attribution shown before download.

Done when the add-on installs, verifies by checksum, is removable, and its
absence or removal leaves the permissive core fully working.

## Decisions settled (2026-09-16, second round)

4. **The add-on is converted and compact.** Only English etymology and affix
   relations are extracted into a queryable file, expected in the tens of MB.
   Raw dumps are never kept; updating means re-downloading and re-converting.
5. **Etymology is shown as community-sourced, with provenance.** Everything we
   have is displayed, visibly marked as Wiktionary-derived with the source named.
   Nothing is presented as authoritative fact.
6. **Ingestion is Rust reading the published formats.** No Python or Java in the
   toolchain; streaming readers over JSON, JSON Lines and TSV.

## Out of scope

Anything multilingual beyond leaving room in the API; a LanguageTool sidecar;
retraining any model; spelling correction UI; and any runtime network access.

## Progress

- 2026-09-16: survey completed, six decisions taken across two rounds, plan
  accepted. Nothing implemented.
