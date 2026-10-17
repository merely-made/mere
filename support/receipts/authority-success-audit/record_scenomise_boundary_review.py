"""Checkpoint a read-only complete Scenomise source review and bounded diagnostics."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib
import json
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
load = lambda n: json.loads((HERE / n).read_text(encoding='utf-8'))
def save(n, value):
    (HERE / n).write_text(json.dumps(value, indent=2) + '\n', encoding='utf-8')
def upsert(items, values, key):
    for value in values:
        at = next((i for i, item in enumerate(items) if item.get(key) == value[key]), None)
        if at is None: items.append(value)
        else: items[at] = value

old = load('scenomise_review_receipt.json')
probe = load('scenomise_boundary_probe_receipt.json')
assert probe['test_exit'] == 0 and probe['tests'] == 10 and probe['generated_output_removed']
assert load('pending_read.json') == []
preserved = {**old['preserved_production_and_test_hashes'], **old['source_sha256']}
assert all(sha(ROOT / p) == h for p, h in preserved.items())
assert all(sha(ROOT / p) == h for p, h in probe['source_sha256'].items())
assert not Path('C:/t/cargo-targets/mere/authority-success-scenomise-boundary-probe.exe').exists()
assert not Path('C:/t/cargo-targets/mere/authority-success-scenomise-boundary-probe.pdb').exists()
package_log = (HERE / 'scenomise_boundary_package.log').read_text(encoding='utf-8-sig')
for count in [212, 7, 3, 4]:
    assert f'test result: ok. {count} passed; 0 failed;' in package_log

notes, authority = load('review_notes.json'), load('authority_map.json')
def finding(number, title, source, evidence, scope, repair):
    return {'id': f'Q{number:03}', 'status': 'documented from fully read source and native public API diagnostic; not repaired',
            'title': title, 'source': 'crates/cambium/scenes/scenomise/src/' + source,
            'evidence': evidence, 'scope': scope, 'verification': 'scenomise_boundary_review_receipt.json',
            'diagnostic': 'scenomise_boundary_probe_receipt.json',
            'repair_triage': {'disposition': 'documented; current continuation is read-only', 'reason': repair}}
findings = [
 finding(80, 'Unescaped relationship signatures hide actual text changes', 'history.rs:352 signature',
    'Strictly parsed HostDatasetV2 accepts label a\\u001f\\u001fb/explanation c followed by label a/explanation b\\u001f\\u001fc for the same relationship. Both fields change, but concatenating them with the same unescaped separator makes equal signatures. changes() reports that relationship and both incident occurrences Stable. An ordinary label-change control reports Updated; identical metadata reports Stable.',
    'Permitted C0 text in generated native JSON history, not a cryptographic collision, authorization bypass, source mutation, or executed product renderer.',
    'Compare structured typed fields or use unambiguous framing; share that comparison between relationship and incident-occurrence classification. Preserve legitimate input text.'),
 finding(81, 'Routing treats a weak generation number as a cross-dataset identity witness', 'host_dataset.rs:286 route_relationships; projection.rs:1456 stable_generation',
    'stable_generation hashes revision bytes and sorted occurrence IDs only. A projection compiled for owner-a accepts valid owner-b relationships when revision text r1 and occurrence IDs match; output retains owner-a scene sources and owner-b disclosure provenance. Same-source control succeeds and changed-revision r2 refuses. The route function promises to refuse projection/dataset mismatch but checks only the generation number.',
    'Native trusted in-process composition/provenance error. Compile itself checks the exact definition SourceBinding and revision; this is the later routing seam. No remote authorization exploit, network access, or source writes.',
    'Retain and check a suitable explicit compiled disclosure/source witness. A generation counter or weak hash must not substitute for dataset identity; decide the intended same-revision content contract before choosing the witness.'),
 finding(82, 'Coordinate admission validates nominal spacing instead of the actual scale override', 'projection.rs:1298 validate_occurrences; compile_inner/disclose',
    'A Geographic definition and dataset x=1e6 pass compile with default spacing16. A finite accepted units_per_coordinate override of 1e38 also passes current definition and occurrence admission, then ProjectionCompiler::compile returns Ok with infinite x. Coordinate range admission multiplies by arrangement.spacing rather than the resolved scale; built-in solve_via wraps solve in Ok without finite-output validation.',
    'Native actual ProjectionCompiler input/output with finite admitted definition and source values. This adds a compiler producer to Q069/Q074; no renderer, snapshot reopening, or product archive was executed in this diagnostic.',
    'Validate resolved scaling with checked/wide arithmetic and shared finite output admission. Apply the rule consistently to Geographic, Hulls and Embedding without claiming one nominal spacing bounds every actual transform.'),
 finding(83, 'Relaxation moves endpoints while retaining stale routed polylines', 'relax.rs:169 relax_roles',
    'A compiler-plus-route_relationships fixture starts with valid straight routed endpoint geometry. Zero steps leaves the complete Scene unchanged. One bounded repulsion step with spring0 moves both items but leaves RoutedRelation.points unchanged; the first route point differs from the moved from item.',
    'Exported native API behavior. Current Rust source search finds no non-test product caller of relax/relax_roles/relax_holding. No claim that a live Canvas currently renders stale edges. Arbitrary curved or source-owned route geometry needs an explicit ownership seam.',
    'Provide a reroute or invalidation seam after geometry mutation. Do not silently replace arbitrary domain routes with straight lines; this effect and ownership question should remain documented rather than prolonging the source audit.'),
 finding(84, 'Relaxation recomputes center-only bounds even when no geometry moves', 'relax.rs:92 placed_bounds; :326 final bounds',
    'One compiled item has footprint10x10 and initial scene bounds10x10. With steps1, dt0, repulsion0 and spring0, transform and footprint remain unchanged, but scene bounds become0x0. The same module already has a four-corner transformed footprint bounds helper for collisions, while final bounds independently unions centers.',
    'Native exported API and duplicated bounds implementation. Same absent product-caller qualification as Q083; no executed live clipping, viewport-fit or rendering failure.',
    'Share footprint-aware bounds in the declared coordinate space, preserving intentional content/backdrop bounds policy. Reuse a geometry seam rather than adding another center-only or transform-incomplete loop.'),
 finding(85, 'Checked history parsing skips occurrence identity admission and silently loses duplicate predecessors', 'host_dataset.rs:435 parse_host_history; :461 check_history; history.rs:187 classify_revisions',
    'A predecessor contains a,b and another a labeled replacement. ProjectionCompiler rejects it on occurrence_id, but JSON parse_host_history accepts it. With successor a labeled replacement,b, history classification maps predecessor IDs through a HashMap, keeps the last a, and reports every current occurrence Stable.',
    'Native checked history parser with malformed duplicate identity, independently rejected by the compiler. No source-store mutation or authorization behavior. Relationship validation uses an endpoint set and does not establish occurrence uniqueness.',
    'Run a shared structural dataset validator before history classification. Refuse ambiguous identity instead of arbitrarily deduplicating; keep relationship, compiler and history admission responsibilities explicit.'),
]
upsert(notes['findings'], findings, 'id')
def entry(number, domain, claim, implementations, assessment):
    return {'id': f'A{number:03}', 'domain': domain, 'claim': claim,
            'implementations': implementations, 'assessment': assessment,
            'state': 'all Scenomise source read; native boundary diagnostics recorded; no production repair',
            'verification': 'scenomise_boundary_review_receipt.json'}
entries = [
 entry(86, 'History comparison authority', 'Disclosed compared fields determine updates to relationships and incident occurrences.',
    ['ComparedFields selects typed metadata', 'history.signature flattens values with unescaped control separators', 'The same signature feeds relationship and incident-occurrence classification'],
    'Comparison reuse is real, but shared ambiguous encoding propagates the same false Stable result to multiple outputs (Q080). A single function is not necessarily faithful to the structured facts it represents.'),
 entry(87, 'Dataset identity versus scene generation', 'Routing refuses a projection from another dataset or revision.',
    ['Compiler checks exact definition SourceBinding and revision', 'stable_generation hashes revision plus sorted occurrence IDs', 'route_relationships treats generation equality as sufficient binding'],
    'Compilation has genuine source checks; the later routing seam loses that stronger authority. Q081 proves a different owner can pass the weaker witness. Generation reuse for animation/caching is not by itself a content or source identity contract.'),
 entry(88, 'Occurrence admission across history and compile', 'Checked history is safe to compare by occurrence identity.',
    ['Compiler.validate_occurrences rejects blank/duplicate IDs', 'History.check_history validates revisions/source/relationship constraints', 'Relationship validation builds an endpoint HashSet', 'classify_revisions builds a predecessor HashMap'],
    'Relationship validation is shared, structural occurrence admission is not. Q085 shows a checked parser admitting ambiguity that compile refuses; the later map silently chooses a predecessor.'),
 entry(89, 'Resolved options versus numeric/output admission', 'Validated finite input produces usable bounded projection geometry.',
    ['Projection reconstructs ProjectionDraft and runs shared definition validation', 'Built-ins resolve actual scale in catalog options', 'Coordinate admission uses nominal arrangement.spacing', 'Built-in solve_via has no shared finite output check'],
    'Draft bypass is ruled out at compile. Q082 instead demonstrates mismatch between nominal and resolved scaling, plus the existing Q069 output admission gap. Public finite configuration is not equivalent to finite computed output.'),
 entry(90, 'Geometry mutation, route and bounds ownership', 'A Scene retains usable bounds and relationship geometry after relaxation.',
    ['solve has placement bounds', 'relax.placed_bounds transforms all four footprint corners for collisions', 'relax final bounds unions centers only', 'relax mutates item transforms without route updates', 'opened_link and Graphshell grouped projection have further specialized geometry updates'],
    'Q083/Q084 show an exported scene mutation API updating only part of derived geometry. Current product callers do not invoke this relaxer. The Graphshell grouping consumer explicitly reroutes hosted straight relations and recomputes world footprint bounds; that caller is evidence of another ownership seam, not proof of a live relax defect.'),
 entry(91, 'Solver registry capability declarations and execution', 'Requires says each item must have a channel; capabilities support solver selection.',
    ['solve_via accepts a required channel when any item discloses it', 'Existing test accepts one of three items disclosing a required channel', 'recommended_max_items is explicitly advisory', 'is_deterministic is advertised without a replay-required mode', 'Named built-in families bypass the custom registry'],
    'Per-item requires wording is stronger than implemented admission. No custom solver failure or mandatory deterministic replay contract is established. The introductory eleven-family count is stale against fourteen catalog families. Do not turn advisory metadata into an invented enforcement requirement.'),
 entry(92, 'Fold visibility and entered-focus projection', 'Scene fold facts are authoritative for fold hiding.',
    ['HostDatasetFolds.project returns the full Scene plus an entered/context-filtered visible set', 'Scene.fold_effect applies fold hiding but not the entered focus set', 'Graphshell GroupedHostDataset.view explicitly consumes projected.visible'],
    'Fold hiding and entered focus are distinct operations. Native probe shows the difference; the inspected actual Graphshell consumer handles it. No portable export or remote divergence was demonstrated. Same-named FoldProjection types elsewhere are not automatically duplicate owners.'),
 entry(93, 'Custom solver identity and refresh cache', 'Unchanged score inputs can reuse previous placements.',
    ['ProjectionCompiler.refresh compares exact score inputs and occurrence mapping', 'Previous projection carries no registry implementation identity', 'Two registries can bind the same custom ID to different solvers'],
    'Native probe shows cross-compiler refresh reuses old geometry where a fresh second compile differs. Same custom ID may intentionally promise stable implementation, and previous geometry may contain user drags. Preserve as a qualified contract/caller question rather than asserting actual product stale cache.'),
 entry(94, 'Matrix derived source identity', 'Matrix cells keep row/column references and contributors beside their derived source.',
    ['Derived source IDs join row and column source.id strings', 'Adapters and axis authorities are omitted from that ID', 'Instances, row refs and contributor refs remain distinct', 'Relationship aggregate IDs use joined contributor IDs and the first adapter'],
    'Native probe shows distinct adapter-backed rows sharing one derived cell SourceRef. This may be intentional source sharing because occurrence/contributor identity survives. No failing source resolver/action is established; a consumer/identity contract is required before promoting it to a bug.'),
]
upsert(authority['entries'], entries, 'id')
upsert(notes['source_qualifications'], [
 {'area': 'Scenomise complete source boundary review', 'note': 'All forty inventoried source files are fully read at current hashes. This continuation adds nineteen files / 287896 characters. SceneDocument validates primary and linked projections by compiling them. Internal recipe row/column channels are refused from source and injected into a clone. Refresh compares exact input facts rather than relying only on generation.'},
 {'area': 'Scenomise boundary verification', 'note': '226 package tests passed (212 library,7 backdrop,3 document,4 template). Ten native boundary observation tests passed: six defects, one ordinary history control and three qualified cache/focus/matrix behaviors. No production repair, renderer/network/store/source-write/wasm or hazardous expansion execution. No current non-test Rust product caller found for the exported relaxer.'},
 {'area': 'Scenomise candidate narrowing', 'note': 'Compiler recreates ProjectionDraft and invokes shared definition validation; generic public-field bypass is not established. Generated Metre key ambiguity is already tested/refused. Entered focus is explicitly consumed via visible in the actual inspected Graphshell view. Template checked span/count/depth and parent-before-child validation refuse invalid structural input; template conformity is not inferred from mere placement.'},
], 'area')
upsert(notes['candidates'], [
 {'title': 'Cross-compiler refresh under changed custom solver registration', 'source': 'crates/cambium/scenes/scenomise/src/projection.rs:ProjectionCompiler::refresh', 'note': 'Native diagnostic reproduces old geometry under the same custom solver ID with a changed registered implementation. Contract/caller question: stable ID may promise implementation identity and retained user positions may be deliberate. Do not label an executed product stale cache.'},
 {'title': 'Entered focus and portable fold visibility are separate outputs', 'source': 'crates/cambium/scenes/scenomise/src/host_dataset/folds.rs:project', 'note': 'Native diagnostic shows unrelated item absent from visible but present/unhidden in the complete Scene. Graphshell GroupedHostDataset.view explicitly consumes visible. No broken live view, remote transport or export demonstrated.'},
 {'title': 'Matrix derived source aliases distinct adapters', 'source': 'crates/cambium/scenes/scenomise/src/matrix.rs:cell', 'note': 'Native diagnostic shows identical derived SourceRef for distinct adapter-backed row identities sharing the same id string. Distinct instances/row/contributor facts remain. Source-sharing intent and actual resolver/action effect are unqualified; retain as candidate.'},
], 'title')
inventory, coverage = load('source_inventory.json')['files'], load('read_coverage.json')
complete = sorted(p for p, v in inventory.items() if p.startswith('crates/cambium/scenes/scenomise/') and coverage.get(p, {}).get('sha256') == v['sha256'] and coverage[p]['through'] == v['chars'])
assert len(complete) == 40
assert all(sha(ROOT / p) == inventory[p]['sha256'] for p in complete)
stats = json.loads(subprocess.check_output(['python', str(HERE / 'read_coverage.py'), 'status'], cwd=ROOT))
assert not stats['changed_since_inventory'] and not stats['pending_chunks']
next_path = next(p for p in sorted(inventory) if p.startswith('crates/canvas/cartography/') and coverage.get(p, {}).get('through', 0) < inventory[p]['chars'])
next_read = {'path': next_path, 'start': coverage.get(next_path, {}).get('through', 0)}
if 'crates/cambium/scenes/scenomise' not in notes['completed_areas']:
    notes['completed_areas'].append('crates/cambium/scenes/scenomise')
notes['current_area'] = 'Scenomise complete; downstream Cartography next'
notes['next_area'] = 'crates/canvas/cartography'
notes['mode'] = 'Read-only full-source audit continuation; preserve prior authorized repairs and concurrent WIP.'
notes['checkpoint'] = {'classification': 'progress', 'goal_status': 'active; full-source objective incomplete', 'coverage': stats,
    'finding_count': len(notes['findings']), 'authority_entries': len(authority['entries']), 'new_receipts': ['scenomise_boundary_review_receipt.json'],
    'next_read': next_read, 'diagnostic_outcomes': 'Six new findings Q080-Q085; 226 package tests and ten native observation diagnostics pass. Three boundary candidates qualified separately; no repair claim.',
    'workspace': 'Stable C:/t/cargo-targets/mere reused; own probe exe/PDB removed; no isolated target, Cargo home or worktree.'}
notes['audit_continuation'] = {'previous_turn': 'progress: built-in solvers/catalog full reading and Q074-Q079',
    'classification_of_previous_turn': 'progress', 'production_edits_this_turn': False, 'coverage_checkpoint': stats, 'next_read': next_read,
    'user_instruction': 'Continue the full-source audit; preserve concurrent work; do not change production code.',
    'coverage_note': 'Nineteen more source files fully read; all forty Scenomise files complete. Mere full-source objective remains incomplete.'}
assert len({v['id'] for v in notes['findings']}) == len(notes['findings'])
assert len({v['id'] for v in authority['entries']}) == len(authority['entries'])
save('review_notes.json', notes)
save('authority_map.json', authority)

doc = 'design_docs/mere_docs/implementation_strategy/2026-10-07_scenograph_editor_plan.md'
addition = '''

### Read-only complete Scenomise boundary audit, 2026-10-16

All forty Scenomise source files are now fully read at recorded hashes. This
continuation adds nineteen files. Mere-wide coverage is 572/3352 source files;
the full-source audit remains active. Production code is unchanged this pass.

Six further findings Q080–Q085 have native reproductions. History joins compared
relationship text with unescaped control separators: two different permitted
label/explanation pairs produce the same signature, so both the relationship and
its incident occurrences falsely report Stable. Ordinary changes remain a
passing control. Checked history parsing also admits duplicate occurrence IDs
that the compiler rejects; its predecessor map silently retains the last one
and can report all current items Stable. Shared structural admission and typed
comparison are needed at these distinct boundaries.

Relationship routing checks a generation made only from revision text and sorted
occurrence IDs. A valid owner-b dataset routes onto an owner-a projection when
those weak inputs match, despite the documented different-dataset refusal.
Compile itself checks the stronger source binding; this is the later routing
seam. A changed-revision control refuses. This is an in-process provenance and
composition defect, not an executed remote authorization bypass.

A finite accepted units_per_coordinate override of 1e38 with finite x=1e6
produces infinite compiled geometry. Coordinate admission checks nominal spacing
instead of resolved scale, and built-in solve has no shared finite output check.
Definition draft validation does run; this is not a bypass of that validator.
The result extends Q069/Q074 with a real ProjectionCompiler producer.

Relaxation moves item transforms without moving their existing routed polylines.
It also recomputes bounds from centers, discarding a stationary 10x10 footprint
even with dt=0. The same module already computes transformed footprint corners
for collision bounds. Current Rust source search finds no non-test product
caller of this relaxer, so no live Canvas clipping or stale-edge claim follows.
Route ownership, invalidation and consistent content bounds need a named shared
seam; replacing arbitrary routes silently would not be an adequate local fix.

Three other probes remain qualified candidates rather than bugs. A different
custom registry under the same solver ID can reuse previous geometry, but stable
ID and retained-user-placement semantics need a caller contract. Entered focus
uses a visible set beyond Scene fold hiding; the actual inspected Graphshell
view consumes that set correctly. Matrix derived source IDs omit adapter identity
and can alias, while distinct instances and contributors remain; no failing
resolver or action is established.

The package suite passes 226 tests with authoring enabled. Ten native observation
tests pass: six reproduce defects, one controls ordinary history comparison,
and three qualify the boundaries above. Diagnostic success is not a repair or
product acceptance claim. No renderer, network, real store, source writes, wasm
or hazardous expansion was executed. The owned standalone executable/PDB were
removed; stable shared Cargo output was reused. Exact hashes, scope and logs are
in the [boundary audit receipt](../../../support/receipts/authority-success-audit/scenomise_boundary_review_receipt.json).
'''
destination = ROOT / doc
before = destination.read_bytes()
assert b'### Read-only complete Scenomise boundary audit, 2026-10-16' not in before
destination.write_bytes(before + addition.encode())
diff = subprocess.run(['git', 'diff', '--check'], cwd=ROOT, capture_output=True, text=True)
(HERE / 'scenomise_boundary_review_diff_check.txt').write_text(diff.stdout + diff.stderr, encoding='utf-8')
assert diff.returncode == 0
evidence = ['scenomise_boundary_probe.rs', 'run_scenomise_boundary_probe.py', 'scenomise_boundary_library_build.jsonl',
    'scenomise_boundary_package.log', 'scenomise_boundary_probe_build.log', 'scenomise_boundary_probe_output.log',
    'scenomise_boundary_probe_receipt.json', 'scenomise_boundary_review_diff_check.txt']
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(), 'basis_commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
    'kind': 'Read-only complete Scenomise source review; broader Mere incomplete', 'previous_turn_classification': 'progress', 'production_edits_this_pass': False,
    'new_findings': [v['id'] for v in findings], 'authority_entries': [v['id'] for v in entries], 'coverage': stats,
    'complete_scenomise_files': complete, 'next_read': next_read, 'source_sha256': {p: sha(ROOT / p) for p in complete},
    'preserved_production_and_test_hashes': preserved, 'package_tests': 226, 'package_test_exit': 0,
    'diagnostic_tests': 10, 'diagnostic_exit': 0, 'generated_output_removed': True, 'retained_isolation': [],
    'document': {'path': doc, 'sha256': sha(destination), 'preserved_prefix_bytes': len(before), 'preserved_prefix_sha256': hashlib.sha256(before).hexdigest()},
    'diff_check': 0, 'evidence_sha256': {p: sha(HERE / p) for p in evidence},
    'limits': ['No whole-source completion or production repair claim.', 'No actual product caller found for the exported relaxer.',
        'No renderer, wasm, remote authorization, live network, real store, source writes or hazardous expansion execution.',
        'Q073 prior client byte-identity failure remains open; the broader client/host suites were not rerun.']}
assert all(sha(ROOT / p) == h for p, h in preserved.items())
save('scenomise_boundary_review_receipt.json', receipt)
print(json.dumps({'coverage': stats, 'scenomise_complete': len(complete), 'findings': len(notes['findings']), 'authority_entries': len(authority['entries']), 'next_read': next_read}))
