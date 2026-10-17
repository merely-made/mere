"""Read-only partial Pictograph checkpoint; never replay preceding recorders."""
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

old = load('cartography_review_receipt.json')
probe = load('pictograph_boundary_probe_receipt.json')
assert probe['test_exit'] == 0 and probe['tests'] == 13 and probe['generated_output_removed']
assert 'test result: ok. 13 passed; 0 failed;' in (HERE / 'pictograph_boundary_probe_output.log').read_text()
assert load('pending_read.json') == []
preserved = {**old['preserved_production_and_test_hashes'], **old['source_sha256']}
assert all(sha(ROOT / p) == h for p, h in preserved.items())
assert all(sha(ROOT / p) == h for p, h in probe['source_sha256'].items())
notes, authority = load('review_notes.json'), load('authority_map.json')

def finding(number, title, source, evidence, scope, repair, size='contained'):
    return {'id': f'Q{number:03}', 'status': 'documented from fully read Pictograph source and native diagnostic; not repaired',
        'title': title, 'source': 'crates/canvas/pictograph/src/canvas/' + source,
        'evidence': evidence, 'scope': scope, 'verification': 'pictograph_review_receipt.json',
        'diagnostic': 'pictograph_boundary_probe_receipt.json',
        'repair_triage': {'disposition': 'documented; active goal forbids production edits',
            'size': size, 'reason': repair}}

findings = [
 finding(91, 'Degree face size and height ignore Resource topology counted by the shared channel',
    'cartography.rs:node_size; node_height',
    'Resource Cites contributes one projected relation and gives both endpoints weight.degree=2, versus isolated=1. node_size with degree enabled stays36 and node_height stays0 because neighbors_undirected reads Surface topology only. A Surface UserGrouped control gives44 and16.',
    'Public native Canvas size/height and channel APIs. Shared producer source was inspected separately; no headed comparison. Physics mass.degree deliberately counts visible controller topology, unlike arrangement weight.degree; that distinction is not this defect.',
    'Choose a shared projected degree reader with explicit multiplicity/self-link semantics, then use it in these encodings. Do not add another local graph traversal or collapse intentionally different mass/weight channels.', 'shared semantics; document broader repair'),
 finding(92, 'Programmatic zoom clears a physically held Ctrl modifier',
    'camera_host.rs:CameraHost for Canvas::wheel_zoom_px',
    'With Ctrl already true, wheel_zoom_px zooms1 to1.15 and unconditionally writes Ctrl=false. The next raw wheel no longer zooms until the host reasserts Ctrl=true. The helper synthesizes raw input by mutating persistent modifier state.',
    'Native public camera APIs and released/reasserted modifier controls; OS key delivery was not executed.',
    'Use a semantic zoom operation, or preserve and restore the previous modifier state. A command must not change physical key state.'),
 finding(93, 'Explicit Pan command becomes zoom while Ctrl is held',
    'camera_host.rs:pan_step; controls.rs:CanvasCommand::Pan',
    'CanvasCommand::Pan {dy:40} changes zoom1 to1.15 with Ctrl held; the released-key control remains1. pan_step routes through modifier-sensitive wheel instead of a pan operation.',
    'Native command reducer and Canvas APIs. User pan inertia may still be desired; no headed keyboard binding exercised.',
    'Share semantic pan arithmetic independently of raw wheel interpretation, preserving the configured motion behavior.'),
 finding(94, 'Retired field continues to emit visual coupling paint',
    'coupling_paint.rs:visual_overlays; fields.rs:rebuild_coupling_forces; seiche_bridge.rs:coupling_force_from_graph',
    'A scalar Const1 visual/halo coupling emits one overlay before retirement and an identical overlay after RetireField makes is_active false. The visual path reads every held definition without an active check. Physics coupling admission and field overlay/picking do check lifecycle.',
    'Native public visual painter, not GPU pixels. Fully read frame/underlay source connects the painter to the main Canvas underlay. Exact frame output for this fixture was not executed.',
    'Apply shared lifecycle eligibility before resolving visual couplings. Retired definitions may remain held for history; their presence must not imply active effects.'),
 finding(95, 'Fold boundary routes ignore canonical link and exact assertion hides',
    'fold_paint.rs:fold_routes; selection.rs:set_hidden_link_keys; set_hidden_assertion_ids',
    'A folded a/b group with b->c Resource Cites has one boundary route and one controller row. Canonical link hide or exact assertion hide removes that controller row, but fold_routes still returns and counts the route. It checks legacy EdgeCell hides only. Ordinary appearance geometry uses all three hide sets.',
    'Two native generated-fold diagnostics and controller introspection. Main frame calls fold_overlay in fully read source; headed fold paint was not executed. Source relation remains intact as intended.',
    'Route shared visible source eligibility into fold bundling, before display aggregation. Preserve directed source accounting and the distinction between canonical links and displayed fold bundles.', 'shared visibility policy; document broader repair'),
 finding(96, 'Pointer cancellation leaves a pending self-link click active',
    'input.rs:pointer_down; pointer_cancel; pointer_up',
    'Pressing the Resource self-link arch at(9,-34), then pointer_cancel, returns false as though nothing changed. A subsequent release still selects the LinkKey because self_link_press was never cleared. Cancellation promises not to activate click selection.',
    'Native public pointer API with default36px body and self-Cites fixture; no native window focus/capture stream executed.',
    'Clear self_link_press as part of cancellation and include it in changed-state accounting. Review graph replacement separately for other pending gesture state.'),
 finding(97, 'Size mutation restarts physics while Canvas reports paused',
    'lifecycle.rs:resettle_for_size; input.rs:settle_physics; fields.rs:delete_field; strategy.rs:sync_affinity_force',
    'A paused Canvas initially has no settle budget. set_node_size80 leaves physics_paused=true but is_settling=true with a positive budget. One CPU Canvas frame advances FieldClock elapsed while paused. resettle_for_size calls physics.settle directly, bypassing the documented single pause gate. Field deletion and affinity sync also contain direct ordinary-mutation settle calls.',
    'Size path and one CPU frame reproduced. Field deletion/affinity calls are source-qualified, not executed diagnostics. Paused placement may still mask body motion; no visible-position movement claim. AtRest home return has a separate intentional budget and should not be mechanically removed.',
    'Route ordinary settle requests through the pause gate, then qualify intentional home/preview exceptions. The broader pause/backend/tick-demand contract needs review; do not treat the paused boolean alone as evidence the backend is stopped.', 'shared pause semantics; document broader repair'),
 finding(98, 'Chrome-facing node colors use a stale duplicate of the canonical palette',
    'palette.rs; cartography.rs:node_color; node_state_color; build.rs:NODE_SHEET',
    'palette.rs claims one table for every representation and build.rs generates CSS from it. node_color repeats old hex constants: selected#f7a440 versus canonical[232,150,40], idle#5a8fc8 versus canonical[54,92,156], with similarly stale open/closed colors. Native exported accessors return the old values.',
    'Native Canvas accessors against public palette constants; CSS generation/source reuse read, no product screenshot comparison.',
    'Format/read the shared palette instead of maintaining a second literal table. Keep product theming policy separate from this duplicate default-data repair.'),
 finding(99, 'Dynamics inspector reports visible input rows that its controller has hidden',
    'dynamics_inspection.rs:inspect_dynamics; relationship_topology.rs:inspect_pair_with_link_hides; seiche_bridge.rs:visible_relation_edges_with_assertion_hides',
    'Before exact assertion hide, pair_controller_edges and pair_inputs.visible_rows both equal1. After hide, the same report returns controller0 and visible_inputs1, with CellInput.hidden=false. The advertised shared exact topology seam omits hidden_assertions on the inspection branch.',
    'Native inspection of one generated Resource Cites and exact hide. Limits correctly disclaim measured forces, but do not disclose inconsistent visibility eligibility. No stored-source revision or force magnitude inferred.',
    'Use one assertion-aware visibility reader for controller and inspection while retaining held versus visible counts. Grouped cell summaries may need partial hidden counts rather than a single boolean.', 'shared grouped visibility semantics; document broader repair'),
 finding(100, 'Pair Unrelate leaves Resource semantic statements intact',
    'selection.rs:retract_selected_relation; retract_semantic_between',
    'Selecting the endpoints of a live Resource Cites and calling retract_selected_relation returns0 and leaves it live. The same Cites is retractable through retract_relation_between_members with Semantic(Cites), returning1 and removing its projected contribution. retract_semantic_between enumerates sub-kinds only through the Surface edge lookup.',
    'Native public selection and sanctioned graph-delta paths. Retraction retains its source bucket/tombstone; initial harness wrongly expected bucket deletion and was corrected to assert live projected rows. Original failing harness/output/receipt retained. Do not equate held history with active relation.',
    'Read all relevant projected semantic owners and dispatch sanctioned retractions with explicit author/scope policy. Do not enumerate only the old Surface store or delete history buckets.', 'source-owner and scope policy; document broader repair'),
 finding(101, 'Isolate selection excludes Resource neighbors',
    'view.rs:isolate_selection',
    'With selected a and Resource Cites a->b, isolate_selection scopes to a alone and excludes b. Adding Surface UserGrouped over the same endpoints makes the control scope contain both. The immediate-undirected-neighbors promise still calls Surface-only neighbors_undirected.',
    'Native Canvas scope APIs and Surface control, with generated graph. This is distinct from Cartography NodeFilter Q089: actual Canvas neighborhood construction itself excludes the Resource endpoint.',
    'Use an explicit shared projected neighborhood traversal with the intended hide/direction policy. Keep exact authored roster scoping and its empty-roster semantics separate.', 'shared topology semantics; document broader repair'),
]
upsert(notes['findings'], findings, 'id')

def entry(number, domain, claim, implementations, assessment):
    return {'id': f'A{number:03}', 'domain': domain, 'claim': claim, 'implementations': implementations,
        'assessment': assessment, 'state': 'partial Pictograph source review; native observations recorded; no production repair',
        'verification': 'pictograph_review_receipt.json'}

entries = [
 entry(103, 'Topology queries after the two-stratum migration', 'User graph operations and degree encodings describe the displayed graph.',
    ['Face degree/height and isolate_selection use Surface neighbors_undirected', 'Pair Unrelate enumerates Surface edge semantic_data', 'Arrangement weight.degree uses projected incoming/outgoing', 'Explicit selector retraction routes Resource statements correctly'],
    'Q091/Q100/Q101 reproduce incomplete adoption of projected topology. A shared graph owner exists; old consumers still read a narrower storage authority. Repair the consumer contract rather than adding another independent topology model.'),
 entry(104, 'Semantic camera commands versus physical input state', 'CameraHost gives hosts explicit pan and zoom verbs.',
    ['Canvas pan_step calls raw wheel', 'wheel_zoom_px writes Ctrl true then false', 'Raw wheel interprets held modifiers', 'CanvasCommand delegates the shared CameraHost trait'],
    'Command reducer reuse is real, but the shared implementation mixes semantic intent with physical event interpretation. Q092/Q093 show both wrong behavior and key-state corruption.'),
 entry(105, 'Field lifecycle across force and paint paths', 'One held field/coupling layer drives physical and visual effects.',
    ['Coupling force bridge/rebuild filters active fields', 'Field interaction/extent painter filters active fields', 'Visual coupling painter evaluates held retired definitions', 'Main Canvas underlay invokes the visual coupling painter'],
    'Q094 is divergent lifecycle admission over the same source definition. Numen evaluator reuse is real and does not provide host lifecycle policy automatically.'),
 entry(106, 'Visibility before fold/display aggregation', 'Hidden source cells stay excluded from fold boundary routes.',
    ['Ordinary link geometry accepts legacy/canonical/exact hides', 'Controller uses assertion-aware adapter', 'Fold routes independently walk projected relations and only legacy hides'],
    'Q095 reproduces two missing eligibility inputs. Share source admission before bundling; a fold route is still a derived display object, not a new graph assertion.'),
 entry(107, 'Interaction termination state', 'pointer_cancel ends transient interaction without click activation.',
    ['Many gesture fields are individually reset', 'self_link_press survives cancellation', 'pointer_up later consumes it', 'set_graph has another manually maintained gesture-reset list'],
    'Q096 exposes cancellation drift. Graph replacement omissions remain a separate unexecuted candidate. A single interaction-state cancellation seam should own termination rather than several incomplete field lists.'),
 entry(108, 'Pause and scheduling admission', 'settle_physics is the single gate suppressing settle triggers while paused.',
    ['Most mutations call settle_physics', 'Size/delete-field/affinity paths call physics.settle directly', 'Frame advances backend before paused placement overlay', 'AtRest owns an intentional home-return budget'],
    'Q097 proves the blanket single-gate claim false. Distinguish mutation triggers from intentional at-rest transitions before consolidation; paused rendering alone does not establish backend inactivity.'),
 entry(109, 'Palette ownership', 'palette.rs is the one table used by every node representation.',
    ['CSS custom properties are generated from palette.rs', 'node_color and node_state_color retain independent stale literals', 'Selected/open/closed/idle values differ'],
    'Q098 is actual conflicting default data despite the one-table claim. This is a contained shared-data repair, not a reason to redesign host theme settings.'),
 entry(110, 'Inspector and controller topology evidence', 'The exact relation-cell input seam is shared by controller and inspector.',
    ['Controller consumes hidden_assertions', 'Pair inspector accepts canonical and legacy hides only', 'One DynamicsInspection publishes both independently computed counts', 'Measured-force/source-storage limits are otherwise explicit'],
    'Q099 demonstrates a diagnostic authority split. The report must distinguish held and admitted rows with the same eligibility policy before claiming exact input correspondence.'),
 entry(111, 'Portable spec, host binding and execution', 'Scenograph stores the recipe; Seiche derives it; Pictograph binds catalog and channel inputs.',
    ['DynamicsSlot remains durable opaque versioned storage', 'dynamics_spec calls shared derive then host binding', 'PhysicsChoice is a flat picker view with the full authored recipe retained', 'ProjectionDynamics and Canvas use shared Seiche execution', 'Unsupported weighted root is explicitly refused'],
    'This ownership separation is implemented reuse. Do not conflate compatible portable syntax with a supported host binding, or report weighted-root silence after the refusal control passed.'),
 entry(112, 'Board, Canvas and dynamics presentation', 'Board reuses graph-free law inputs and Seiche physics without owning graph truth.',
    ['PhysicsBoard accepts scene items and site labels', 'Canvas/Board use LawInputs and shared Physics backend', 'Board inline/actor paths and shared AtRest are real reuse', 'Occurrence preview creates disposable body IDs while retaining original source refs'],
    'Separate board and graph adapters are justified owners. Catalog enum labels and picker arrays repeat metadata but no current mismatch established. Board stage damping/target application remains a qualified candidate, not a proved runtime defect.'),
 entry(113, 'Meaning cache and provider contract', 'One revision/generation-tagged Meaning run feeds shared channels.',
    ['MeaningJob uses shared ESP embedding/Louvain', 'Inline and actor paths execute that job', 'Canvas rejects stale revision/generation results', 'MeaningEngine promises one vector per text but job does not validate count'],
    'Shared computation and stale-result rejection are real. Short/extra provider output could stall, panic or publish incomplete similarity, but shipped-provider contract breach has not been executed. GPU/shared-device feature was read only, not built or qualified.'),
 entry(114, 'Position authority versus obsolete migration prose', 'Underlay docs still claim committed graph-position fallback and a Seiche-free producer.',
    ['projection_from_graph explicitly puts every node at origin', 'projection_from_positions uses unwrap_or_default for missing live positions', 'Graph positions are no longer durable truth', 'Pictograph manifest and Canvas own a Seiche dependency'],
    'Old prose survives the extraction and can misdescribe structural-only output. Main Canvas supplies live positions; no claim that its ordinary nodes all render at origin. Repeated conversion/bounds concerns remain linked to Q088 rather than inflated into an unexecuted renderer bug.'),
 entry(115, 'Stable identity across graph and gesture replacement', 'Described item keys stay stable for a Canvas life; graph switches reset transient view state.',
    ['Accessibility described keys use NodeKey index', 'Durable member IDs remain separate UUIDs', 'set_graph manually clears some gesture/role state', 'Orbit/empty/self-link/context pending state is not in the same reset list'],
    'Source-qualified identity/replacement candidates. Need actual host event and NodeKey reuse qualification before assigning findings. Durable UUID ownership is real and should not be replaced by numeric presentation handles.'),
 entry(116, 'View reconstruction and territory bounds', 'Historical snapshots are disposable presentations; territory cost is bounded by the node cap.',
    ['SourceTimeCanvas preserves live Canvas while selecting history', 'snapshot_canvas manually copies selected subsets of geometry/roles/view state', 'Territory filters node visibility but row count follows unbounded aspect ratio', 'Camera/ground geometry use independent projection helpers'],
    'Historical graph truth separation is implemented. Full presentation fidelity, territory work bounds and isometric rectangles need follow-up; no live history mutation, hang or pixel comparison executed. Document these larger seams without extending this source pass into a rewrite.'),
]
upsert(authority['entries'], entries, 'id')

upsert(notes['candidates'], [
 {'title': 'Meaning provider vector-count admission', 'source': 'crates/canvas/pictograph/src/canvas/meaning_job.rs', 'note': 'Custom MeaningEngine contract promises one vector per text; job trusts count. Short/empty slice results may stall/repeat, surplus vectors can overrun next slice or be discarded by zip. Shipped provider correctness not disproved. Follow-up should use a bounded malformed-provider diagnostic before changing shared scheduling.'},
 {'title': 'Graph replacement gesture and numeric-handle reuse', 'source': 'crates/canvas/pictograph/src/canvas/lifecycle.rs; input.rs; reader.rs', 'note': 'set_graph does not clear every pending gesture field; described item key is NodeKey.index despite stable-for-Canvas-life claim. Need generated replacement/reuse diagnostics plus actual host cancellation contract before assigning exact wrong-target effects.'},
 {'title': 'Scoped overlays and minimap membership', 'source': 'crates/canvas/pictograph/src/canvas/frame.rs; build.rs; gloss.rs', 'note': 'Community/bridge rings and minimap loop view positions independently of ordinary scoped/folded node visibility. Territory explicitly filters visibility as a control. Need product lens semantics and a bounded CPU paint-list fixture; no hidden-node pixel leak claim.'},
 {'title': 'Territory work grows with unbounded aspect ratio', 'source': 'crates/canvas/pictograph/src/canvas/territory.rs:territory; territory_runs', 'note': 'MAX_TERRITORY_NODES caps nodes but rows=ceil(height/(width/48)) has no cap. Even two finite far-separated vertical nodes can create excessive cells/commands. Current Canvas adds finite margin but does not bound span/aspect. No hanging workload executed. Review viewport clipping, maximum cells and policy before a broader repair.'},
 {'title': 'Historical presentation reconstruction omits some live view state', 'source': 'crates/canvas/pictograph/src/canvas/source_time.rs:snapshot_canvas', 'note': 'Live graph remains separately held, but refresh_live replaces its Canvas with a manually copied subset. Current method copies viewport/sizing/images/faces/selection/ordinary scope/strategy/roles; it does not copy full dynamics, hides, folds, seed/regions/mosaic or all view toggles. Selective fidelity needs an explicit contract and native source-history controls; no graph truth rewrite alleged.'},
 {'title': 'Board stage parameters need application qualification', 'source': 'crates/canvas/pictograph/src/canvas/physics_board.rs:set_stage', 'note': 'Stage binding uses law/composition/sources/seed but damping/target application needs comparison with the documented graph-free board scope and existing tests. Do not report ignored dynamics policy before qualifying that API.'},
], 'title')
upsert(notes['source_qualifications'], [
 {'area': 'Pictograph partial full-reading checkpoint', 'note': '96 of163 package source/manifest files fully read, 1409628 chars. All immediate canvas/*.rs implementation files except canvas/tests.rs are read; the tests directory, signals and other package files remain. Targeted producer/query excerpts are not complete-file coverage.'},
 {'area': 'Pictograph native evidence scope', 'note': 'Thirteen native observation tests pass: twelve observations cover eleven defects Q091-Q101; one confirms weighted-root refusal. One CPU Canvas frame advances its backend/clock. No package suite, GPU feature, headed rendering, network, store or source writes. Passing defect observations do not mean fixes.'},
 {'area': 'Pictograph harness correction', 'note': 'Initial12-case diagnostic had11 pass and1 harness failure: explicit retraction retains a source bucket/tombstone, so expecting resource_relations count0 was wrong. Corrected to projected live count0; subsequent12 then13 cases pass. Initial source/output/receipt retained; no production change was needed.'},
 {'area': 'Pictograph actual reuse and ownership', 'note': 'Shared Scenomise solve, Seiche derive/Physics/AtRest, Numen field evaluation and ESP Meaning work are real reuse. Canvas view positions, occurrence bodies, ambient simulations and graph source facts have distinct roles. Physics mass.degree and arrangement weight.degree intentionally differ in input/offset semantics; no duplicate-authority allegation for that distinction.'},
], 'area')

inventory, coverage = load('source_inventory.json')['files'], load('read_coverage.json')
complete = sorted(p for p, v in inventory.items() if p.startswith('crates/canvas/pictograph/') and coverage.get(p, {}).get('sha256') == v['sha256'] and coverage[p]['through'] == v['chars'])
assert len(complete) == 96 and sum(inventory[p]['chars'] for p in complete) == 1409628
assert all(sha(ROOT / p) == inventory[p]['sha256'] for p in complete)
stats = json.loads(subprocess.check_output(['python', str(HERE / 'read_coverage.py'), 'status'], cwd=ROOT))
assert not stats['changed_since_inventory'] and not stats['pending_chunks']
next_path = next(p for p, v in sorted(inventory.items()) if p.startswith('crates/canvas/pictograph/') and coverage.get(p, {}).get('through', 0) < v['chars'])
next_read = {'path': next_path, 'start': coverage.get(next_path, {}).get('through', 0)}
notes['current_area'] = 'Pictograph partial: immediate Canvas implementation read; tests/signals/package remainder pending'
notes['next_area'] = 'crates/canvas/pictograph'
notes['checkpoint'] = {'classification': 'progress', 'goal_status': 'active; full-source objective incomplete', 'coverage': stats,
    'finding_count': len(notes['findings']), 'authority_entries': len(authority['entries']),
    'new_receipts': ['pictograph_review_receipt.json'], 'next_read': next_read,
    'diagnostic_outcomes': 'Eleven findings Q091-Q101; thirteen native diagnostics pass. No production repair or whole-package test claim.',
    'workspace': 'Stable C:/t/cargo-targets/mere reused; owned probe executable/PDB removed; no isolated target, home or worktree.'}
notes['audit_continuation'] = {'previous_turn': 'progress: Scenomise/Cartography completion and Q080-Q090',
    'classification_of_previous_turn': 'progress', 'production_edits_this_turn': False,
    'coverage_checkpoint': stats, 'next_read': next_read,
    'user_instruction': 'Full source audit; document larger repairs, preserve concurrent work; active goal forbids production edits.',
    'coverage_note': '96 additional complete files and1409628 characters. Pictograph96/163 complete; Mere688/3352 complete. Full source goal remains active.'}
assert len({v['id'] for v in notes['findings']}) == len(notes['findings'])
assert len({v['id'] for v in authority['entries']}) == len(authority['entries'])
save('review_notes.json', notes)
save('authority_map.json', authority)

doc = 'design_docs/mere_docs/implementation_strategy/2026-10-07_scenograph_editor_plan.md'
addition = '''

### Read-only Pictograph implementation audit, 2026-10-16

This checkpoint adds96 fully read Pictograph source/manifest files and
1,409,628 characters, bringing Mere coverage to688/3352 files. Pictograph is
96/163 complete. Its immediate canvas implementation files are read; the
Canvas test root/directory, signals and other package files remain unread.
The full-source goal remains active. Production code is unchanged.

Q091-Q101 record eleven further defects. Three consumers still read only the
Surface store after Resource topology was introduced: degree size/height,
pair Unrelate and Isolate selection. Native Resource Cites fixtures demonstrate
each omission; Surface UserGrouped controls show the older path still works.
The shared projected graph owner exists, but adoption remains incomplete.

Camera commands reuse the raw wheel interpreter: explicit Pan zooms with Ctrl
held, and programmatic zoom clears a physically held Ctrl flag. Cancellation
misses self_link_press, so a canceled self-link click still selects on release.
The documented single pause gate also has bypasses: resizing a paused node
starts a backend settle budget and one CPU frame advances its field clock while
Canvas still reports paused. This does not establish visible body movement.
Field deletion and affinity sync have related direct settle calls in source;
AtRest home return owns an intentional budget and needs separate qualification.

The same lifecycle/visibility facts are interpreted differently by separate
derived paths. Retired fields still paint visual coupling halos although force
admission and field picking/extent painting filter lifecycle. Fold boundary
routes ignore canonical link and exact assertion hides that the controller
honors. DynamicsInspection itself reports zero controller rows beside one
visible input row after an exact hide. Its shared exact-topology claim therefore
overstates the actual shared eligibility. Grouped partial-hidden inputs need
explicit representation rather than another independent visibility loop.

palette.rs supplies real shared CSS data, but exported node_color and
node_state_color repeat stale literals. Selected and all three state colors
disagree with the table advertised as the one palette. This is a contained
duplicate-data issue. Camera verbs and cancellation are also contained repair
candidates. Projected-neighborhood semantics, grouped visibility, scoped source
retraction and the pause/backend contract are documented for a broader repair
so they do not prolong the source audit into a redesign.

Shared Scenomise solve, Seiche derive/Physics/AtRest, Numen field evaluation and
ESP Meaning computation are real reuse. The weighted-root negative control
confirms explicit refusal, not silent acceptance. Arrangement weight.degree
and physics mass.degree intentionally have different inputs/offsets; that
distinction is not a duplication defect. Disposable occurrence bodies preserve
their original source references and do not become graph truth.

Larger qualified follow-ups are recorded without promoting unexecuted effects:
Meaning provider vector-count validation; graph replacement gesture/handle
reuse; scoped overlays/minimap membership; full historical presentation copying;
Board stage parameter application; and territory cells growing with unbounded
aspect ratio despite its node cap. Underlay prose still describes committed
position fallback, while missing live positions actually default to origin.
The ordinary main Canvas supplies live positions; no all-origin rendering claim
is made.

All13 native diagnostic cases pass: twelve observations reproduce eleven
defects, and one qualifies refusal. These are observations of unfixed behavior,
not repaired regressions. The initial12-case run had one harness mistake:
retraction retains historical buckets, so the corrected control asserts live
projected rows rather than bucket deletion. Initial evidence is retained.
One CPU frame ran; no whole-package suite, headed UI, GPU feature, network or
store was exercised. The owned executable/PDB were removed and no isolated
target/home/worktree was created. Resume with canvas/tests.rs.

Exact hashes, preservation checks and triage are in the
[Pictograph audit receipt](../../../support/receipts/authority-success-audit/pictograph_review_receipt.json).
'''
destination = ROOT / doc
before = destination.read_bytes()
assert b'### Read-only Pictograph implementation audit, 2026-10-16' not in before
destination.write_bytes(before + addition.encode())
diff = subprocess.run(['git', 'diff', '--check'], cwd=ROOT, capture_output=True, text=True)
(HERE / 'pictograph_review_diff_check.txt').write_text(diff.stdout + diff.stderr, encoding='utf-8')
assert diff.returncode == 0

evidence = ['pictograph_boundary_probe.rs', 'run_pictograph_boundary_probe.py', 'pictograph_library_build.jsonl',
    'pictograph_boundary_probe_build.log', 'pictograph_boundary_probe_output.log', 'pictograph_boundary_probe_receipt.json',
    'pictograph_boundary_probe_initial.rs', 'pictograph_boundary_probe_initial_output.log',
    'pictograph_boundary_probe_initial_receipt.json', 'pictograph_review_diff_check.txt']
extra_sources = ['crates/canvas/pictograph/src/signals/producers.rs', 'crates/graph/graph-kernel/src/graph/query.rs']
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(),
    'basis_commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
    'kind': 'Read-only partial Pictograph full-source review; Mere incomplete', 'production_edits_this_pass': False,
    'new_findings': [v['id'] for v in findings], 'authority_entries': [v['id'] for v in entries],
    'coverage': stats, 'complete_pictograph_files': complete, 'next_read': next_read,
    'source_sha256': {p: sha(ROOT / p) for p in complete}, 'preserved_production_and_test_hashes': preserved,
    'targeted_source_excerpt_hashes_not_complete_reading': {p: sha(ROOT / p) for p in extra_sources},
    'diagnostic_tests': 13, 'diagnostic_exit': 0, 'package_tests': 'not run',
    'generated_output_removed': probe['generated_output_removed'], 'retained_isolation': [],
    'document': {'path': doc, 'sha256': sha(destination), 'preserved_prefix_bytes': len(before),
        'preserved_prefix_sha256': hashlib.sha256(before).hexdigest()},
    'diff_check': 0, 'evidence_sha256': {p: sha(HERE / p) for p in evidence},
    'limits': ['Full-source objective remains incomplete; no production repair.',
        'No whole Pictograph suite, GPU feature, headed renderer, store, network or OS input execution.',
        'Native case count describes observed defects and one refusal control, not passing fixed regressions.',
        'Initial retraction harness corrected for retained source tombstones; failure evidence preserved.',
        'Prior Q073 client byte-identity failure remains open; broader earlier gates were not rerun.']}
assert all(sha(ROOT / p) == h for p, h in preserved.items())
save('pictograph_review_receipt.json', receipt)
print(json.dumps({'coverage': stats, 'pictograph_complete': len(complete), 'findings': len(notes['findings']),
    'authority_entries': len(authority['entries']), 'next_read': next_read}))
