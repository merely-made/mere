"""Read-only complete Cartography source review; preserve production and earlier receipts."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib
import json
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
load = lambda n: json.loads((HERE / n).read_text(encoding='utf-8'))
def save(n, value): (HERE / n).write_text(json.dumps(value, indent=2) + '\n', encoding='utf-8')
def upsert(items, values, key):
    for value in values:
        at = next((i for i, item in enumerate(items) if item.get(key) == value[key]), None)
        if at is None: items.append(value)
        else: items[at] = value

old = load('scenomise_boundary_review_receipt.json')
probe = load('cartography_boundary_probe_receipt.json')
assert probe['test_exit'] == 0 and probe['tests'] == 7 and probe['generated_output_removed']
assert load('pending_read.json') == []
preserved = {**old['preserved_production_and_test_hashes'], **old['source_sha256']}
assert all(sha(ROOT / p) == h for p, h in preserved.items())
assert all(sha(ROOT / p) == h for p, h in probe['source_sha256'].items())
assert not Path('C:/t/cargo-targets/mere/authority-success-cartography-boundary-probe.exe').exists()
assert not Path('C:/t/cargo-targets/mere/authority-success-cartography-boundary-probe.pdb').exists()
assert 'test result: ok. 57 passed; 0 failed;' in (HERE / 'cartography_package.log').read_text(encoding='utf-8-sig')
notes, authority = load('review_notes.json'), load('authority_map.json')

def finding(number, title, source, evidence, scope, repair):
    return {'id': f'Q{number:03}', 'status': 'documented from fully read Cartography source and native diagnostic; not repaired',
        'title': title, 'source': 'crates/canvas/cartography/src/' + source, 'evidence': evidence, 'scope': scope,
        'verification': 'cartography_review_receipt.json', 'diagnostic': 'cartography_boundary_probe_receipt.json',
        'repair_triage': {'disposition': 'documented; current continuation is read-only', 'reason': repair}}
findings = [
 finding(86, 'Ordered affinity lookup returns the reverse value when it appears first', 'signals.rs:AffinityScores::lookup',
    'The type expressly permits asymmetric pairs and lookup promises forward-first/reverse-only fallback. With [(b,a)=0.2,(a,b)=0.8], lookup(a,b) returns0.2; reordering the same two facts returns0.8. The predicate finds either orientation in one pass rather than exhausting the forward lookup first. Reverse-only fallback control returns0.2 as intended.',
    'Public native contract with generated asymmetric data. Current inspected structural-affinity producer is symmetric and physics consumes pair lists directly; no product ranking or force error is claimed.',
    'Contained no-brainer when production edits resume: forward lookup first, then reverse fallback. Keep directed facts intact rather than normalizing them into an unordered pair.'),
 finding(87, 'Portable scene lowering drops arrangement regions while preserving only overlay halos', 'scene_out.rs:scene_from_projection',
    'PartitionAdapter returns nonempty ProjectedRegion contours/members/labels with no signal fault. scene_from_projection keeps items but emits zero regions because it only visits Overlay::ClusterHalo. Adding one halo produces exactly that halo, confirming the arrangement regions were ignored rather than merged or refused.',
    'Native actual Partition adapter-to-portable Scene chain. Current source search found no non-test product caller of scene_from_projection, so no live export/render failure is alleged. Generic public scene conversion loses existing supplied geometry.',
    'Contained conversion repair when production edits resume: lower projection.regions with instance remapping and preserve contours/labels alongside halo overlays. Qualify member omissions without inventing geometry.'),
 finding(88, 'Generic score conversion replaces valid footprint bounds with center-only bounds', 'adapters/score.rs:project_score; projection_from_positions; bounds_of; scene_out.rs:scene_from_projection',
    'A valid one-item 10x10 Score solves to10x10 bounds. project_score extracts only positions and projection_from_positions recomputes0x0 center bounds. scene_from_projection with measured10x10 extent keeps the footprint but copies0x0 bounds. The specialized Spiral conversion instead retains scene.bounds.',
    'Native public compiler-independent solve/conversion chain, no renderer or live clipping execution. Different producer from Q084: loss occurs after correct solver bounds, even without relaxation. Content-bounds policy for regions and later importance transforms needs explicit ownership.',
    'Reuse solved bounds where converting a solved Scene, then define consistent footprint/region/emphasis bounds admission during later lowering. Avoid another independent center-only bounds implementation.'),
 finding(89, 'ViewIntent filters are accepted but never materialized by Cartography', 'request.rs:NodeFilter; ViewIntent::with_filter; adapters/score.rs:score_from_request',
    'The NodeFilter contract assigns materialization to Cartography before strategy projection. GridAdapter.project with an explicit one-node filter, three-node graph and complete order places all three with no fault or coverage limit. Supplying a one-node order separately places one as a control. Full package read finds no filter materializer.',
    'Exported request/strategy API capability defect. No current non-test product NodeFilter use found; actual Pictograph subgraph projection constructs its own induced graph and remaps IDs explicitly. Do not allege that the existing scoped canvas leaks filtered nodes.',
    'Documented broader seam: choose a real shared request/filter materializer or explicit unsupported-filter refusal; ensure graph, order and relation endpoints agree. Preserve the actual host induced-subgraph behavior rather than adding another siloed filtering algorithm.'),
 finding(90, 'Partition and Mosaic add a second isolation offset to degree-plus-one signals', 'adapters/bounded.rs:PartitionAdapter::project; MosaicAdapter::project',
    'WEIGHT_DEGREE is documented as degree plus one. Pictograph producers::degree_weights actually emits (degree+1) and its registry discloses it to Partition dispatch. Bounded adapters apply max(0)+1 again. A native Partition fixture given valid channel weights1/2/2 exactly matches a direct solve with2/3/3 and differs from a solve using the disclosed1/2/2; isolated share changes1/5 to2/8. Mosaic repeats the same base offset before user scale.',
    'Cartography public native diagnostic with documented channel values; actual Pictograph producer/registry/dispatch inspected as source excerpts, not executed. Partition effect reproduced; Mosaic duplicate implementation source-qualified. No product renderer or saved user layout modified.',
    'Contained arithmetic repair when edits resume: consume the channel according to its already-offset semantics, with explicit fallback for absent/invalid weights. Keep zero-degree handling in the producer owner so adapters cannot independently boost it again.'),
]
upsert(notes['findings'], findings, 'id')

def entry(number, domain, claim, implementations, assessment):
    return {'id': f'A{number:03}', 'domain': domain, 'claim': claim, 'implementations': implementations,
        'assessment': assessment, 'state': 'all Cartography source read; seven native diagnostics recorded; no production repair',
        'verification': 'cartography_review_receipt.json'}
entries = [
 entry(95, 'Actual placement reuse versus repeated graph conversion', 'Nothing in Cartography places items; Scenomise owns placement.',
    ['Generic adapters build Score and call scenomise::solve', 'Spiral builds its own Score and calls the same solver', 'Both rebuild node/edge projections separately', 'Generic edge builder and Spiral repeat missing-position origin defaults and weight1.0'],
    'Placement reuse is real, replacing the retired origin trick. Post-solve conversion remains duplicated and diverges in bounds, signal membership handling and node ordering. This is derived representation duplication, not a second durable graph truth owner.'),
 entry(96, 'Region preservation across portable lowering', 'Projection carries arrangement regions and scene lowering makes portable region geometry.',
    ['project_score translates solved polygon regions into ProjectedRegion', 'scene_from_projection lowers ClusterHalo overlays only', 'ProjectedRegion values are never read in that lowerer'],
    'Q087 proves a lossy conversion despite successful native return. Existing overlay migration is real but does not imply complete Projection-to-Scene migration. Product call path remains unexecuted and no non-test caller was found.'),
 entry(97, 'Bounds authority after solve and re-lowering', 'content_bounds denotes total projected content.',
    ['Scenomise solve computes footprint-aware bounds', 'Generic projection conversion discards them and calls bounds_of on centers', 'Spiral retains solved bounds', 'Portable lowering copies the supplied projection bounds after adding measured footprints/emphasis'],
    'Q088 exposes duplicate bounds authorities with different answers for the same valid content. Related Q084 is a separate mutation seam. A shared coordinate-aware content policy should cover footprints/regions/transforms instead of spreading local loops.'),
 entry(98, 'Filtering responsibility and actual host scoping', 'Cartography materializes NodeFilter before handing a filtered graph to strategies.',
    ['ViewIntent carries filter and with_filter builder', 'No Cartography source consumes filter', 'Grid reads graph plus order directly', 'Pictograph project_canvas_subgraph builds an induced graph, computes its own facts and remaps stable IDs'],
    'Q089 is an unrealized advertised API, not evidence that current host scoping is broken. ViewIntent is also not literally the only call-site configuration because adapter structs expose configuration. Config ownership claims should match executable seams.'),
 entry(99, 'Directed affinity semantics and reverse fallback', 'AffinityScores permits asymmetry and prioritizes an exact ordered pair.',
    ['Vec stores ordered pairs', 'lookup ORs forward and reverse predicates in a single find', 'Structural affinity currently supplies symmetric scores', 'Physics constructors consume the raw pair list'],
    'Q086 is concrete exported lookup behavior; symmetric producers mask it. Do not infer an executed physics/ranking effect where lookup is not used.'),
 entry(100, 'Zero-degree share ownership', 'weight.degree already includes the plus-one that gives isolated nodes a share.',
    ['Pictograph producer computes degree+1', 'Registry publishes that value under WEIGHT_DEGREE', 'Partition and Mosaic independently apply another plus-one', 'Radial consumes supplied weights directly'],
    'Q090 shows two layers owning the same offset with different share ratios. The repeated adapter policy is semantically active duplication, not just a repeated helper. Native Partition effect proven, live host execution not claimed.'),
 entry(101, 'Signal membership preconditions versus typed faults', 'Adapters report absent or mistyped channels; one host registry serves one graph.',
    ['IntelligenceSignals typed readers check presence and variant only', 'Generic score builder skips absent graph keys', 'Spiral expects all supplied order keys to exist and panics otherwise', 'Pictograph registry caller explicitly pairs graph and registry'],
    'Native stale-key diagnostic proves inconsistent unchecked membership handling, but the live host precondition was not violated. Keep as qualified candidate, not a reproduced current host panic. Duplicate order keys/numeric ranges likewise lack generic structural admission.'),
 entry(102, 'Package status claims versus current implementation', 'Root docs say v0 contract-only, sibling strategies, no canvas integration.',
    ['Manifest correctly describes Sceno/Scenomise adapters and retired graph-canvas-adapters feature', 'lib.rs status and LayoutStrategy docs retain contract-only/sibling-crate text', 'Adapters and Pictograph dispatch are implemented', 'Graph-only strategy list and dispatch have a parity test', 'Advisory/default channel table differs from optional weight fallback'],
    'Stale status understates implemented code; manifest is more current. Shared graph-only dispatch is genuine. channels_read names default producer needs but Partition/Mosaic intentionally allow absent weights; this weaker optional contract should not be confused with the blanket all-channel-fault prose.'),
]
upsert(authority['entries'], entries, 'id')
upsert(notes['candidates'], [
 {'title': 'Missing projection endpoints are represented by origin geometry before portable lowering', 'source': 'crates/canvas/cartography/src/adapters/score.rs:build_positioned_edges; spiral_score.rs:edges', 'note': 'Native generated one-endpoint projection emits a path to origin. with_graph_coverage reports omission and scene_from_projection drops that edge. Full-order normal producer preconditions may prevent this in a live host; no live phantom-edge rendering demonstrated. Shared conversion seam should expose/refuse absent endpoint geometry.'},
 {'title': 'Spiral requires stronger signal membership than generic score construction', 'source': 'crates/canvas/cartography/src/spiral_score.rs:project_spiral_score_for_view', 'note': 'Native stale-key probe catches Spiral panic while generic Grid skips that same absent key. Real host registry explicitly serves its paired graph and refreshes disclosures; actual host contract violation unproved.'},
 {'title': 'Empty typed histories and registries can bypass checked constructors', 'source': 'crates/canvas/cartography/src/adapters/changes.rs:read_changes; reading.rs:GraphReadingRegistry::default_profile', 'note': 'Public data structures are deserializable/constructible empty; read_changes subtracts1 and default_profile expects nonempty. Checked host history parser refuses empty revisions. No parsed-valid input or real caller failure established; qualify constructor preconditions before promoting.'},
 {'title': 'Importance scale area wording versus linear transform', 'source': 'crates/canvas/cartography/src/overlay.rs:ImportanceScale; scene_out.rs:importance', 'note': 'Overlay says scale screen-area by factor, lowerer uses factor directly as uniform linear scale. No current non-test product lowerer caller found and surrounding consumer expectations unreviewed; do not infer a live two-axis scale bug without contract qualification.'},
], 'title')
upsert(notes['source_qualifications'], [
 {'area': 'Cartography full source review', 'note': 'All20 package source/manifest files fully read at current hashes (194673 chars). Actual placement is shared Scenomise code; post-solve conversion duplicates geometry/metadata handling. No native production edits this pass. Source-only Pictograph excerpts are not counted as full file reads.'},
 {'area': 'Cartography verification and runtime limits', 'note': '57 package tests and seven native diagnostics pass. Five diagnostics reproduce exported contract defects Q086-Q090; two explicitly qualify absent endpoint/stale signal behavior. Native Partition weighting executed; live Pictograph producer/render/network/export not executed. No non-test source caller found for scene_from_projection or NodeFilter.'},
 {'area': 'Cartography scene identity mapping narrowing', 'note': 'Reviewed built-in Scenomise solve places every ordered item; no per-item omission causes zip shifting in normal adapter construction. Generic builder emits fresh contiguous ordinals; Mosaic reorders items and keys together. Arbitrary reordered public Score callers must satisfy project_score documented ordinal/key preconditions; no normal adapter source-identity shift is established.'},
], 'area')

inventory, coverage = load('source_inventory.json')['files'], load('read_coverage.json')
complete = sorted(p for p, v in inventory.items() if p.startswith('crates/canvas/cartography/') and coverage.get(p, {}).get('sha256') == v['sha256'] and coverage[p]['through'] == v['chars'])
assert len(complete) == 20
assert sum(inventory[p]['chars'] for p in complete) == 194673
assert all(sha(ROOT / p) == inventory[p]['sha256'] for p in complete)
stats = json.loads(subprocess.check_output(['python', str(HERE / 'read_coverage.py'), 'status'], cwd=ROOT))
assert not stats['changed_since_inventory'] and not stats['pending_chunks']
next_path = next(p for p, v in sorted(inventory.items()) if p.startswith('crates/canvas/pictograph/') and coverage.get(p, {}).get('through', 0) < v['chars'])
next_read = {'path': next_path, 'start': coverage.get(next_path, {}).get('through', 0)}
if 'crates/canvas/cartography' not in notes['completed_areas']: notes['completed_areas'].append('crates/canvas/cartography')
notes['current_area'] = 'Scenomise and Cartography complete; Pictograph next'
notes['next_area'] = 'crates/canvas/pictograph'
notes['checkpoint'] = {'classification': 'progress', 'goal_status': 'active; full-source objective incomplete', 'coverage': stats,
    'finding_count': len(notes['findings']), 'authority_entries': len(authority['entries']),
    'new_receipts': ['scenomise_boundary_review_receipt.json', 'cartography_review_receipt.json'], 'next_read': next_read,
    'diagnostic_outcomes': 'Eleven findings Q080-Q090 this continuation. 226 Scenomise and57 Cartography tests pass; seventeen native observation diagnostics pass. Qualified cases remain separate; no production repair.',
    'workspace': 'Stable C:/t/cargo-targets/mere reused; both owned native probes removed; no isolated target, Cargo home or worktree.'}
notes['audit_continuation'] = {'previous_turn': 'progress: built-in solver/catalog full reading and Q074-Q079', 'classification_of_previous_turn': 'progress',
    'production_edits_this_turn': False, 'coverage_checkpoint': stats, 'next_read': next_read,
    'user_instruction': 'Continue the full-source audit; preserve concurrent work; do not change production code.',
    'coverage_note': '39 additional complete files this continuation, 482569 additional characters. All40 Scenomise and20 Cartography source files read. Mere full-source objective remains incomplete.'}
assert len({v['id'] for v in notes['findings']}) == len(notes['findings'])
assert len({v['id'] for v in authority['entries']}) == len(authority['entries'])
save('review_notes.json', notes)
save('authority_map.json', authority)

doc = 'design_docs/mere_docs/implementation_strategy/2026-10-07_scenograph_editor_plan.md'
addition = '''

### Read-only complete Cartography audit, 2026-10-16

All twenty Cartography source/manifest files are now fully read at recorded
hashes. Combined with the Scenomise completion above, this continuation adds
39 fully read files and 482,569 characters. Mere-wide coverage is592/3352 files;
the full-source objective remains active. Production code remains unchanged.

Q086–Q090 document five further defects. AffinityScores permits asymmetric facts
but lookup finds either orientation in one pass, so reverse-first storage wins
over a present forward value. Portable scene lowering ignores ProjectedRegion:
Partition contours/members/labels disappear while a ClusterHalo control remains.
The generic score converter discards valid footprint-aware Scene bounds and
recomputes center-only bounds; a10x10 item becomes0x0 content. Lowering restores
the footprint but copies those zero bounds. Spiral uses a separate converter
which retains the solved bounds. Placement itself is genuinely shared through
Scenomise; this is duplicated, lossy conversion after solving.

ViewIntent.filter has a builder and assigns materialization to Cartography,
yet no implementation reads it. An explicit one-node filter places three nodes
with no fault. Actual Pictograph scoped layout independently builds an induced
graph and remaps stable IDs, so no live scoped-canvas leak is claimed. Partition
and Mosaic also add another plus-one to weight.degree, although the reviewed
Pictograph producer and registry already supply degree+1. A native Partition
probe given1/2/2 matches a direct solve of2/3/3: isolated share changes1/5 to2/8.
Producer/registry/dispatch were inspected as excerpts, not executed or counted
as full Pictograph reading; Mosaic's duplicate offset is source-qualified.

Two native observations remain qualified: raw projection edges default a missing
endpoint to origin, but coverage reports omission and portable lowering drops
the edge; Spiral panics on a stale order key where the generic builder skips
it, but the actual inspected host pairs a registry with its graph. Current
source search finds no non-test scene_from_projection or NodeFilter caller.
No live export, phantom edge, ranking/physics or stale-signal host failure is
alleged. Generic scene identity shifting was also narrowed out: built-in solve
places every ordered item and adapter ordinals/keys remain aligned.

The package root's contract-only/no-integration and sibling-strategy text is
stale against its own manifest and implemented adapters/Pictograph dispatch.
The graph-only dispatch table and its parity checks are actual shared code.
Missing/mistyped channel faults are real; optional Partition/Mosaic weight
fallbacks are weaker than the blanket all-required-channel wording.

All57 Cartography package tests and seven native observation diagnostics pass.
Five diagnostics reproduce defects; two qualify the boundaries above. Passing
diagnostics do not mean those defects are fixed. Bounds/filter/route ownership
decisions remain documented so they do not consume the broader source audit.
The owned executable/PDB were removed; no isolated build/worktree was created.
Exact source hashes and evidence are in the
[Cartography audit receipt](../../../support/receipts/authority-success-audit/cartography_review_receipt.json).
'''
destination = ROOT / doc
before = destination.read_bytes()
assert b'### Read-only complete Cartography audit, 2026-10-16' not in before
destination.write_bytes(before + addition.encode())
diff = subprocess.run(['git', 'diff', '--check'], cwd=ROOT, capture_output=True, text=True)
(HERE / 'cartography_review_diff_check.txt').write_text(diff.stdout + diff.stderr, encoding='utf-8')
assert diff.returncode == 0
evidence = ['cartography_boundary_probe.rs', 'run_cartography_boundary_probe.py', 'cartography_library_build.jsonl',
    'cartography_package.log', 'cartography_boundary_probe_build.log', 'cartography_boundary_probe_output.log',
    'cartography_boundary_probe_receipt.json', 'cartography_review_diff_check.txt']
extra_sources = ['crates/canvas/pictograph/src/signals/producers.rs', 'crates/canvas/pictograph/src/signals/registry.rs', 'crates/canvas/pictograph/src/canvas/cartography_scene.rs']
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(), 'basis_commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
    'kind': 'Read-only complete Cartography source review; broader Mere incomplete', 'production_edits_this_pass': False,
    'new_findings': [v['id'] for v in findings], 'authority_entries': [v['id'] for v in entries], 'coverage': stats,
    'complete_cartography_files': complete, 'next_read': next_read, 'source_sha256': {p: sha(ROOT / p) for p in complete},
    'preserved_production_and_test_hashes': preserved,
    'targeted_source_excerpt_hashes_not_complete_reading': {p: sha(ROOT / p) for p in extra_sources},
    'package_tests': 57, 'package_test_exit': 0, 'diagnostic_tests': 7, 'diagnostic_exit': 0,
    'generated_output_removed': True, 'retained_isolation': [],
    'document': {'path': doc, 'sha256': sha(destination), 'preserved_prefix_bytes': len(before), 'preserved_prefix_sha256': hashlib.sha256(before).hexdigest()},
    'diff_check': 0, 'evidence_sha256': {p: sha(HERE / p) for p in evidence},
    'limits': ['No whole-source completion or production repair claim.', 'Current non-test product use of scene_from_projection/NodeFilter not found.',
        'Pictograph producer source inspected; no live renderer, network, store, remote authorization, wasm or user archive execution.',
        'Prior Q073 client byte-identity failure remains open; this package pass does not rerun its broader suites.']}
assert all(sha(ROOT / p) == h for p, h in preserved.items())
save('cartography_review_receipt.json', receipt)
print(json.dumps({'coverage': stats, 'cartography_complete': len(complete), 'findings': len(notes['findings']), 'authority_entries': len(authority['entries']), 'next_read': next_read}))
