# Interfaces derived from linked records

A useful question to ask of a vocabulary is what someone can do when a second record uses the same term. A class can give a collection its membership. A predicate can give a reader a way to find evidence from either end of a claim. A selector can identify a passage precisely enough for another app to act on it. Those operations need different inputs.

| Capability | Facts required | Interface | Scope |
| --- | --- | --- | --- |
| Browse and filter | Stable record identity, selected type or predicate | Show observed types and counts. Select a type to see members, or a predicate to see comparable assertions. | Count within the indicated document or collection. Preserve the distinction between a missing value and an explicit negative. |
| Follow and return | Two named endpoints and a directed predicate | Pivot from either endpoint. Show the original direction. Allow return to the containing record or reading. | Looking up incoming links preserves the direction of the recorded assertion. |
| Compare accounts | Source, edition or revision, and the terms each account actually uses | Hold the subject fixed and compare accounts, or hold one account fixed and browse related records. | Keep withdrawals, approximate alignments, disagreements and transcription gaps visible. |
| Act on a passage | Source identity, selector, units and an available representation | Quote a passage, seek to a point, or crop a bounded interval. Offer actions whose inputs are available. | A recording URL alone cannot establish that two edits have the same clock. A point supplies no end for a crop. |
| Save a view across apps | A durable query, lens selection and an explicit set of owner-disclosed sources | Reopen a collection or ask which admitted records refer to the selected item. | This needs source enumeration, revision tracking and removal on loss of access. A document-local index establishes only what that document contains. |

## Product fit and open work

The comparisons below use the projects’ own references and design notes. The final column identifies a further capability and what would make it credible.

| App | Reference behavior | What the linked view contributes | Next missing piece |
| --- | --- | --- | --- |
| Turnstone | TheBrain’s focused neighborhood and the named sets discussed in the browser research | Browse a fetched document by type, follow incoming and outgoing statements, and inspect their source and vocabulary in Gloss. | Save a named query over explicitly retained sources. Reopening it should distinguish removed sources from zero matches. [Neighborhood design](../../../design_docs/mere_docs/research/2026-06-25_borrowed_ideas_brief.md) · [Turnstone collection gap](https://github.com/merely-made/turnstone/blob/47e59e62/design_docs/2026-11-02_user_agent_taxonomy_plan.md#o2-u4-bookmark-taxonomy) |
| Knot | Tinderbox and Roam expose relationships in context; Scrivener gives the writer multiple views of the same material | Filter personal assertions by selected predicate and compare the captured subject and object passages. Keep the author's predicate, direction and revision in view. | Reveal a cited passage in the current draft. Show when a revision has moved or removed it. [Writer references](https://github.com/merely-made/knot-editor/blob/69402ac/design_docs/2026-10-29_knot_writer_ergonomics_brief.md) |
| Cleromancy | Book T and Thierens attribute different signs to the same card | Browse from a card to an attribution to other cards in that same tradition, source and version. Return to the saved draw. | Compare the two traditions side by side for an axis across a deck, with explicit gaps where a transcription supplies no value. [Deck sources](https://github.com/merely-made/cleromancy/blob/f4e9b5e/src/content/tarot-decks-v1.json) |
| Redshank | PodNotes timestamp links and Margin voice annotations preserve an observation during listening | Listen shows text and voice notes that overlap the active transcript passage on the same recording. Sharing a note supplies its source, identity and point or span to another app. | Re-anchor across differently edited recordings. Require representation or alignment evidence before transporting the clock. [Listening references](https://github.com/merely-made/woodshed/blob/ba105986/design_docs/archive_docs/2026-11-03/2026-09-01_listening_annotation_port_plan.md) |
| Woodshed | Progression, rehearsal and looping views select related musical material for practice | Filter the Stage board by observed chord quality while keeping repeated occurrences and their original positions. A matched, bounded passage from another app can also become a portable WAV for practice. | Join practice occurrences to the relevant progression and recording. A chord catalog alone cannot establish what was practiced. [Musical projections](https://github.com/merely-made/woodshed/blob/ba105986/design_docs/2026-11-03_musical_projections_plan.md) |
| Hocket | Deeler’s variation slots and the additive four-track looper organize phrases differently | An immutable captured phrase makes a good citation target across later overdubs. | Map recording time to a particular phrase, track, bar clock and session revision before offering a slot or arrangement action. [Hocket model and references](https://github.com/merely-made/woodshed/blob/ba105986/ports/hocket/design_docs/PROJECT_DESCRIPTION.md) |

## Other consumers

The game wing has a different boundary. Cosmograph owns its world and simulation clock. A linked-data view could read the records disclosed to a particular player. These are possible consumers of the same browsing operations.

| App | Possible view | Evidence needed |
| --- | --- | --- |
| Isocosm (Isometry) | Select a rule in the compendium and find the abilities that use it. The Foundry PF2e comparison identifies rule elements as a useful grain. | Ruleset, edition, cited rule element and stable compendium record. Keep an ability description distinct from an executable game rule. [Wing design](https://github.com/merely-made/isocosm/blob/7dd244b9/design_docs/2026-09-18_wing_design_plan.md) |
| Mesocosm | Compare two generations of a lineage and follow an observed change back to the epoch where it appeared. | Lineage, epoch, observed trait, and the simulation snapshot behind each count. [Lineage and epoch review](https://github.com/merely-made/isocosm/blob/7dd244b9/mesocosm/README.md) |
| Eponym (Paredros) | Follow a character’s knowledge of a place to who told them and when. Two characters can have different views of the same place. | Speaker, listener, world event and the knowledge available to this character. Querying the entire simulation would reveal undiscovered information. [Character knowledge](https://github.com/merely-made/isocosm/blob/7dd244b9/eponym/README.md) |

## Shared implementation

Mere resolves vocabulary identity and localized labels from the selected pack. Its `RecordIndex` accepts named triples from a single source, indexes type membership and stores each directed relationship once with lookup from either endpoint. The caller keeps the index with the source admission. Changing the selection requires rebuilding the projection. A type or relationship label never grants permission to fetch a resource or run code.

Turnstone caches the parsed reading with its fetched response. Gloss pages through the index and reuses the current page until the admission or view changes. Knot retains the distinctions in its signed assertion model. Cleromancy reads the saved deck’s source, tradition and version. These product models supply evidence a generic RDF statement alone cannot recover.
