"""Record actual Seiche source coverage and native observations; Mere remains incomplete."""
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
        at = next((i for i, x in enumerate(items) if x.get(key) == value[key]), None)
        if at is None:
            items.append(value)
        else:
            items[at] = value

old = load('pictograph_completion_review_receipt.json')
probes = [load(n) for n in ['seiche_force_removal_probe_receipt.json', 'seiche_geometry_probe_receipt.json']]
assert [p['tests'] for p in probes] == [3, 2]
assert all(p['test_exit'] == 0 and p['generated_output_removed'] for p in probes)
assert load('pending_read.json') == []
preserved = {**old['preserved_production_and_test_hashes'], **old['source_sha256']}
assert all(sha(ROOT / p) == h for p, h in preserved.items())
assert all(sha(ROOT / path) == h for probe in probes for path, h in probe['source_sha256'].items())
assert all(sha(Path(v['path'])) == v['sha256'] for probe in probes for v in probe['library_sha256'].values())
assert all(sha(HERE / 'seiche_library_build.jsonl') == p['library_build_log_sha256'] for p in probes)
notes, authority = load('review_notes.json'), load('authority_map.json')
findings = [
    {'id': 'Q107', 'title': 'Removing the last force source leaves its persistent Rapier force active',
     'status': 'reproduced through public Simulation API; documented, not repaired',
     'source': 'crates/conatus/seiche/src/lib.rs:clear_forces, set_anchor_force, tick reset window (around904)',
     'evidence': 'After one Boundary tick, clear_forces reports force_count0 but the next velocity grows from-15.719008 to-30.809254; replacing Boundary with strength0 instead decays it to-15.090249. Removing the last AnchorSpring likewise grows velocity from-18.768496 to-36.78625, while a zero-stiffness anchor decays to-18.017756. Identical pre-removal state was checked for Boundary. tick resets persistent Rapier forces only when a built-in/coupling/affinity/anchor/scene-field source remains present, so removing the last source skips clearing its previous accumulator.',
     'scope': 'Two native CPU Simulation observations with zero-force replacement controls. Public direct API behavior; not evidence that Canvas Still fails, since its host law can install Hold or another force and thereby enter the reset branch. Removal through coupling/affinity/scene-field setters shares the inspected reset condition but was not independently executed.',
     'diagnostic': 'seiche_force_removal_probe_receipt.json',
     'repair_triage': {'disposition': 'documented; active goal forbids production edits',
        'size': 'contained force-lifecycle repair',
        'reason': 'Clear stale accumulated user forces even when the newly admitted source list is empty. Qualify force-removal transitions and legitimate external persistent-force ownership before choosing unconditional tick reset versus explicit removal cleanup; preserve gravity and inertia.'}},
    {'id': 'Q108', 'title': 'A zero-sized rounded collider panics instead of applying the advertised minimum clamp',
     'status': 'reproduced through public Simulation API; documented, not repaired',
     'source': 'crates/conatus/seiche/src/node_body.rs:to_shared_shape RoundedSquare around51; set_node_colliders',
     'evidence': 'set_node_colliders accepts Ball radius0 and Square half0 via their positive minimum clamps. RoundedSquare half0 border0 panics because border.clamp(0.1, half-0.1) has inverted bounds, before the positive minimum is applied. The lowering contract explicitly says zero-size nodes retain a pickable body. The diagnostic catches the panic so the successful observation does not imply successful collider admission.',
     'scope': 'One native CPU test with direct public setter, two non-rounded controls and catch_unwind. No partial mutation rollback or hosted UI input path was tested. Canvas ordinary footprint admission clamps to a larger range, so this is not a claim its normal resize slider emits zero.',
     'diagnostic': 'seiche_geometry_probe_receipt.json',
     'repair_triage': {'disposition': 'documented; active goal forbids production edits',
        'size': 'contained admission-order repair',
        'reason': 'Admit a positive finite half-extent before deriving the border clamp bounds, or return a typed refusal consistent with the public contract. Do not silently weaken the already documented zero-size clamp for other shapes.'}},
    {'id': 'Q109', 'title': 'Fluid contact retains an 18-unit node radius after rigid collider resizing',
     'status': 'reproduced through actual public Simulation fluid ticks; documented, not repaired',
     'source': 'crates/conatus/seiche/src/fluid_coupling.rs:couple_fluid_to_bodies around83-91; node_body.rs:set_node_colliders',
     'evidence': 'A pinned node resized to a Ball radius80 leaves a radius7 fluid particle at x60 inside its collider while globally tangible. Tangibility-off control also leaves it at60. A Ball radius4 incorrectly pushes a clear particle from x20 to25, just as the default radius18 does. Actual node position stays at the origin. The fluid adapter builds every graph-node contact with NODE_BODY_RADIUS instead of inspecting its live collider; scene bodies already branch on their stored shapes.',
     'scope': 'One native CPU fluid-tick observation with default, enlarged, reduced and tangibility-off cases. Fluid gravity, viscosity and solver iterations disabled to isolate contact geometry. Ball geometry only, no hull/box/fluid stability claim. This demonstrates separate physical geometry authorities, not merely a drawn-face mismatch.',
     'diagnostic': 'seiche_geometry_probe_receipt.json',
     'repair_triage': {'disposition': 'documented; active goal forbids production edits',
        'size': 'bounded ball-radius correction; broader contact ownership documented separately',
        'reason': 'Read the admitted live collider geometry for node fluid contacts. A shared shape reader spanning graph nodes and scene props, rounded-box/hull approximations and deterministic contact order may exceed the user one-hour audit diversion limit; keep that broader work out of the source-reading pass.'}},
]
for finding in findings:
    finding['verification'] = 'seiche_first_review_receipt.json'
upsert(notes['findings'], findings, 'id')
def entry(number, domain, claim, implementations, assessment):
    return {'id': f'A{number:03}', 'domain': domain, 'claim': claim,
        'implementations': implementations, 'assessment': assessment,
        'state': 'partial Seiche source review; five generated native CPU observations; no production edits',
        'verification': 'seiche_first_review_receipt.json'}
entries = [
    entry(136, 'Registered forces versus persistent Rapier state',
        'Removing the built-in force list leaves bodies coasting under damping.',
        ['clear_forces clears only the registered Vec', 'Removing anchor/affinity/coupling slots changes their records',
         'tick conditionally resets user forces only with an active source', 'Rapier user-force accumulation persists across ticks'],
        'Q107 proves reported source removal does not remove the last applied physical force. The registration record and the solver accumulator diverge. Zero-strength replacement controls isolate the lifecycle defect; ordinary Canvas Still behavior is not inferred.'),
    entry(137, 'Rigid and fluid node geometry',
        'NodeCollider is the admitted physical shape and size.',
        ['set_node_colliders writes live Rapier shapes', 'Fluid graph-node contacts copy constant NODE_BODY_RADIUS',
         'Fluid scene contacts branch on their stored shape', 'Simulation hit/cull/view construction also retain a fixed-radius convention'],
        'Q109 proves fluid contact diverges after Ball resizing. This is an actual second physical geometry implementation. Fixed-radius hit/cull helpers are explicitly documented as approximations; do not silently call them collider-accurate merely because node_body module prose says hit geometry matches the visible face.'),
    entry(138, 'Collider input admission',
        'Zero-size shapes clamp to a positive pickable minimum.',
        ['Ball and Square clamp before shape construction', 'RoundedSquare derives border clamp from unadmitted half',
         'Hull has explicit small-input/failed-hull fallback', 'Public node and scene setters share to_shared_shape'],
        'Q108 proves the rounded branch panics for a finite zero extent while its sibling branches honor the contract. Only direct node setter behavior was executed; sharing the lowerer does not constitute a fresh scene-loader test.'),
    entry(139, 'Proposed coupling effects versus actual writes',
        'Point evaluation describes one proposed effect; traces describe dedicated-slot writes before integration.',
        ['evaluate_target contains response logic', 'Both traced and untraced paths consume it through write_effect',
         'Trace retains per-call before/after state, including ignored writes to fixed bodies',
         'Duplicate physical applications retain ordered slots and immutable input Arcs', 'Trace cap counts omissions without skipping execution'],
        'This is real shared evaluation/write code. Tests distinguish inert evaluation, missing body, ignored force calls, sequential velocity scaling and integrated motion, with trajectory digests and trace-off controls. Generic boxed CouplingForce execution is intentionally outside dedicated-slot trace coverage; these read tests were not freshly executed.'),
    entry(140, 'Membership and callback authority',
        'Owner entry actions are distinct from physical writes and observed only on later entry.',
        ['Membership distinguishes Initial, Entered, Exited and Unavailable',
         'Stable supplied appearance IDs deduplicate repeated membership actions while physical slots repeat',
         'Duplicate named bindings become ambiguous and refuse callbacks',
         'Completed means the installed handler returned Ok, with explicit documentation',
         'Callbacks run synchronously before trace folding or frame dropping'],
        'The inspected contract does not falsely certify a durable graph mutation from callback success. Initial matches and resets suppress entry callbacks; unavailable scope is not fabricated as outside. Handler thread/authority remains owner supplied. This is a qualified example of correctly limited success reporting.'),
    entry(141, 'Field time and solver age',
        'Accepted simulation time is independent of lagged-result step age.',
        ['FieldClock accumulates accepted dt in f64', 'Invalid tick returns before clock/evidence mutation',
         'Reset rewinds elapsed time and membership epoch without resetting solver steps',
         'Dedicated animated couplings request continued ticks only while field clock unpaused',
         'Read-only evaluate_at explicitly samples time0; evaluate_at_time accepts a supplied clock'],
        'The current source supersedes a stale CouplingForce comment calling a real clock a future owner checkpoint. Runtime clock tests read here cover pause, drag, reset and elapsed caps, but the backend runtime implementation remains unread. Do not claim actor equivalence from these inline sources alone.'),
    entry(142, 'Barnes-Hut approximation and exact-rung language',
        'Theta0 is used as the exact rung in numerical instrument tests.',
        ['Opening-angle recursion is disabled at theta0', 'Minimum-cell leaves still merge distinct nearby points',
         'Merged mass includes the queried body when evaluated at the original point',
         'Public min_cell_size and distance cutoff are independently tunable'],
        'A native pair at distance0.2, epsilon0.01, theta0 yields force-20 with min-cell1 versus pairwise-5 with min-cell0.01. Theta0 alone is not a global pairwise guarantee. This is a qualified limitation of an explicitly approximate primitive, not a newly alleged failure of default Charge or existing quality bars; default force distance cutoff8 exceeds min-cell1.'),
    entry(143, 'Retired architecture prose versus current executable ownership',
        'Seiche is kernel free and hosts supply projected positions.',
        ['lib introduction still assigns committed persistent position to graph and says tick writes back',
         'Later lib comment says the graph write-back wrapper was removed',
         'Barnes-Hut module/export prose says live Force wiring is unfinished despite an implementation and catalog use',
         'Fluid solver prose says rigid-world integration is next despite fluid_coupling and tick calling it'],
        'These are stale implementation/authority claims in current source, not evidence of surviving graph write-back code. The code separates host graph facts, projected physical state, dedicated couplings and scene fluid. Removing stale prose is smaller than redesigning those boundaries; no production edits made.'),
    entry(144, 'Numerical tests and experimental implementation copies',
        'Declarations are checked against force/energy behavior rather than labels alone.',
        ['Probe applies real Force implementations in a separate Rapier world',
         'Gradient checks use central differences of declared energy; Jacobian, loop-work and motion are distinct instruments',
         'Positive controls include wrong energy, rotational curl and unequal-group full-force spread',
         'Density grid has a distinct spectral-reference solver in tests',
         'Orbit diagnostics retain experimental counter-damping implementations, explicitly cfg(test) and labelled diagnostic'],
        'Experimental copies and analytic references are not extra production authority merely because they repeat mathematics. Read instruments have real negative controls. Motion/loop diagnostics are not the authoritative no-energy class gate. The retained Orbit Recipe::catalog default uses the old Full/no-centring control, while ruled cases override it explicitly, so its current-catalog wording is stale.'),
    entry(145, 'Lagged result identity and policy edits',
        'A lagged answer is usable for the same bodies within the allowed age.',
        ['Answer stores body handles, submission step and forces',
         'RepulsionRequest carries strength/cutoff/min_distance at submission',
         'Acceptance compares only body order and step age',
         'Existing tests cover delayed/refused/failed/mismatched-body responses and threshold controls'],
        'Request parameters are not retained in answer validity. Changing coefficients for the same bodies can leave a previous-law answer eligible for the staleness window. This is a source-qualified policy/identity question, not a fresh GPU defect reproduction. Decide whether parameter edits invalidate in-flight work before broadening the acceptance identity.'),
]
upsert(authority['entries'], entries, 'id')
upsert(notes['candidates'], [
    {'title': 'Fluid contact order bypasses key-order reproducibility',
     'source': 'crates/conatus/seiche/src/fluid_coupling.rs:scene_bodies.values and bodies_by_node.values; fluid.rs:resolve_contacts',
     'note': 'Contacts are collected from independently randomized maps, then resolve_contacts mutates particle position sequentially for each contact. Overlapping contacts may therefore depend on iteration order even though layout node force sums sort keys. No repeated-run divergence was executed. Defining scene/node contact order and overlap resolution is broader work, potentially beyond the user one-hour diversion limit; preserve the candidate without asserting all simulation trajectories fail determinism.'},
    {'title': 'Empty Density continues asking for ticks before it can initialize a flow',
     'source': 'crates/conatus/seiche/src/laws/density.rs:apply empty-node return; wants_tick',
     'note': 'apply returns for zero nodes before assigning flow; wants_tick treats flow None as true. Direct empty simulation and actual host scheduling have not been probed. Trace runtime/host empty-graph admission before promoting this to a perpetual-frame product finding.'},
    {'title': 'Non-finite numerical instrument results and N-class admission',
     'source': 'crates/conatus/seiche/src/instruments/reading.rs:Reading::agrees_by; conservative',
     'note': 'N-class admission checks whether velocity is read or the conservative candidate fails; no explicit finite-value gate appears. NaN Jacobian medians compare false against the tolerance and could be read as nonconservative. This is a source-qualified diagnostic-admission candidate, not an executed false-positive receipt.'},
    {'title': 'Generic timed coupling keep-ticking policy while field clock paused',
     'source': 'crates/conatus/seiche/src/field.rs:wants_continuous_tick; coupling_force.rs:wants_tick',
     'note': 'Dedicated coupling demand is gated by field_clock.paused; a timed CouplingForce installed as a generic boxed force participates in the ungated forces branch. Generic placement is public and deliberately outside dedicated trace/membership, so its pause-demand contract needs qualification. No idle-loop failure was executed.'},
], 'title')
inventory, coverage = load('source_inventory.json')['files'], load('read_coverage.json')
complete = sorted(p for p, v in inventory.items() if p.startswith('crates/conatus/seiche/')
    and coverage.get(p, {}).get('sha256') == v['sha256'] and coverage[p]['through'] == v['chars'])
assert len(complete) == 40 and sum(inventory[p]['chars'] for p in complete) == 490432
assert all(sha(ROOT / p) == inventory[p]['sha256'] for p in complete)
stats = json.loads(subprocess.check_output(['python', str(HERE / 'read_coverage.py'), 'status'], cwd=ROOT))
assert not stats['changed_since_inventory'] and not stats['pending_chunks']
next_path = next(p for p, v in sorted(inventory.items()) if p.startswith('crates/conatus/seiche/')
    and coverage.get(p, {}).get('through', 0) < v['chars'])
next_read = {'path': next_path, 'start': coverage.get(next_path, {}).get('through', 0)}
notes['current_area'] = 'Seiche40/91 fully read; laws through Gravity including diagnostics; lib/node-body/scene adapter read'
notes['next_area'] = 'crates/conatus/seiche'
notes['checkpoint'] = {'classification': 'progress', 'goal_status': 'active; full-source objective incomplete',
    'coverage': stats, 'finding_count': len(notes['findings']), 'authority_entries': len(authority['entries']),
    'new_receipts': ['seiche_first_review_receipt.json'], 'next_read': next_read,
    'diagnostic_outcomes': 'Q107/Q108/Q109 reproduced; Barnes-Hut minimum-cell limitation measured. Five native observations total. No package/GPU/actor suite claim.',
    'workspace': 'Stable C:/t/cargo-targets/mere reused; both owned native executables/PDBs removed; no isolated target, Cargo home or worktree.'}
notes['audit_continuation'] = {'previous_turn': 'progress: Pictograph source complete; Q106 diagnostic mismatch',
    'classification_of_previous_turn': 'progress', 'production_edits_this_turn': False,
    'coverage_checkpoint': stats, 'next_read': next_read,
    'user_instruction': 'Audit all Mere source; preserve concurrent work; do not change production code.',
    'coverage_note': '40 additional files and490432 chars; Mere795/3352; Seiche40/91. Repository objective remains active.'}
upsert(notes['source_qualifications'], [{
    'area': 'Seiche first source review and native observations',
    'note': '40 source files fully read,490432 chars, including core Simulation, force/coupling/membership/clock/fluid adapters, instruments, grouped composition, Anneal/Boids/Density/Gravity and Orbit diagnostic source. Five generated native CPU observations pass with current separately built Seiche serde/default-actor library. Actor code is compiled but actor execution was not tested. No package suite or ignored diagnostic sweep run.'
}], 'area')
assert len({x['id'] for x in notes['findings']}) == len(notes['findings'])
assert len({x['id'] for x in authority['entries']}) == len(authority['entries'])
save('review_notes.json', notes)
save('authority_map.json', authority)

doc = 'design_docs/mere_docs/implementation_strategy/2026-10-07_scenograph_editor_plan.md'
addition = '''

### Read-only Seiche force lifecycle and geometry review, 2026-10-16

Forty of Seiche's 91 inventoried source files are fully read, adding 490,432
characters. This covers the core Simulation, coupling/trace/membership/clock,
fluid and scene adapters, grouped composition, numerical instruments and laws
through Gravity, including the full Orbit diagnostic source. Mere coverage is
795/3352 files and 11,732,370/36,854,820 characters. Resume at laws/hold.rs.
The full-source objective remains active. Production is unchanged.

Q107: removing the last force source does not clear its persistent Rapier
accumulator. After one Boundary tick, clear_forces reports zero registered
forces but speed grows from 15.719 to 30.809 on the next tick. A zero-strength
replacement instead coasts down to 15.090. Removing the last anchor similarly
grows speed from 18.768 to 36.786; its zero-stiffness control decays to 18.018.
tick clears prior user forces only inside the branch for a nonempty installed
force/coupling/affinity/anchor/scene-field source. Removing the last one skips
that reset. These are direct Simulation observations; Canvas Still may retain
Hold or another installed term, so an ordinary UI Still failure is not claimed.
The contained repair concerns source removal and accumulator lifetime. Preserve
gravity/inertia and qualify any external persistent-force owner before choosing
unconditional reset versus explicit removal cleanup.

Q108: RoundedSquare half0/border0 panics on inverted clamp bounds even though
the lowerer promises a positive minimum for zero-size bodies. Ball radius0 and
Square half0 both clamp successfully. The native diagnostic catches the panic;
its passing result means the defect was observed, not that rounded admission
succeeded. Admit the half-extent before deriving border bounds, or define a
typed refusal. Canvas normal footprint controls clamp higher; their ordinary
resize path was not demonstrated to emit this input.

Q109: rigid node resizing and fluid contact use different physical geometry.
A pinned Ball radius80 leaves a radius7 particle at x60 inside the node even
while tangible. A radius4 node wrongly pushes a clear x20 particle to25, as
the default radius18 does. Tangibility-off and default-size controls isolate
the mismatch. The fluid node adapter copies NODE_BODY_RADIUS instead of the
live collider, while scene props already branch on stored shapes. The native
test disables fluid gravity, viscosity and constraint iterations to isolate
actual contact in a tick. A bounded radius repair is separate from a shared
shape reader for graph nodes, scene props, rounded/hull approximations and
deterministic contact ordering. That broader work may exceed the user's
one-hour audit diversion limit and remains documented.

One measured numerical qualification: theta0 does not eliminate minimum-cell
merging in Barnes-Hut. Two bodies 0.2 apart at epsilon0.01 produce force20
with min-cell1, versus the pairwise force5 with min-cell0.01. The primitive
is explicitly approximate; this is not a new default Charge failure. Its
default force cutoff8 exceeds min-cell1. Statements calling theta0 an exact
rung must state that remaining spatial approximation. Existing quality bars
were not rerun or invalidated by this tuned-input observation.

Authority findings also include correct boundaries. Coupling point evaluation
proposes a write. Both trace paths share evaluation and write helpers; traces
capture calls, ignored fixed-body writes, immutable inputs and omitted counts
before integration. Membership distinguishes unavailable from outside and
initial matches from later entry. Completed explicitly means the installed
owner callback returned Ok, not that a durable graph change was verified.
Duplicate named bindings refuse callback authority. Accepted field time is
separate from solver age; reset, pause and read-only probes have explicit
contracts. Reviewed tests include trajectory hashes, cap-off controls,
negative numerical controls and a distinct spectral reference for diffusion.
These test sources were read; their package suites were not freshly executed.

Several source comments retain retired designs: graph-committed positions and
tick write-back, unfinished Barnes-Hut Force wiring, unfinished fluid integration,
and a future real field clock. Current code contradicts those comments. The
old experimental Orbit counter-damping copies are test-only diagnostics with
ruled-law controls, not hidden production owners. Their Recipe::catalog label
still denotes the older Full/no-centring baseline, not the current ruled recipe.

Broader qualified follow-ups remain open: fluid contacts are collected from
HashMaps then resolved sequentially; lagged result validity binds body order
and age but not changed strength/cutoff parameters; empty Density may keep
requesting ticks before a flow can initialize; generic timed CouplingForce
placement has different pause-demand gating than its dedicated slot; and
non-finite numerical readings lack an explicit N-class admission guard.
None was promoted to an executed product failure in this pass.

The current Seiche CPU library built successfully. Five generated native
observations passed; no GPU, renderer, actor execution, full package suite or
ignored Orbit sweep ran. Both owned executables/PDBs were removed. The normal
Mere build cache remains at C:/t/cargo-targets/mere; no isolated target, Cargo
home or worktree was created. Preservation hashes and exact limits are in the
[Seiche first review receipt](../../../support/receipts/authority-success-audit/seiche_first_review_receipt.json).
'''
destination = ROOT / doc
before = destination.read_bytes()
assert b'### Read-only Seiche force lifecycle and geometry review, 2026-10-16' not in before
destination.write_bytes(before + addition.encode())
diff = subprocess.run(['git', 'diff', '--check'], cwd=ROOT, capture_output=True, text=True)
(HERE / 'seiche_first_review_diff_check.txt').write_text(diff.stdout + diff.stderr, encoding='utf-8')
assert diff.returncode == 0
evidence = ['seiche_library_build.jsonl', 'seiche_library_build_stderr.log',
    'seiche_force_removal_probe.rs', 'run_seiche_force_removal_probe.py',
    'seiche_force_removal_probe_build.log', 'seiche_force_removal_probe_output.log',
    'seiche_force_removal_probe_receipt.json', 'seiche_geometry_probe.rs',
    'run_seiche_geometry_probe.py', 'seiche_geometry_probe_build.log',
    'seiche_geometry_probe_output.log', 'seiche_geometry_probe_receipt.json',
    'seiche_first_review_diff_check.txt']
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(),
    'basis_commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
    'kind': 'Partial Seiche source review; full Mere source objective incomplete',
    'production_edits_this_pass': False, 'new_findings': [x['id'] for x in findings],
    'authority_entries': [x['id'] for x in entries], 'coverage': stats,
    'complete_seiche_files': complete, 'added_complete_files': complete,
    'source_sha256': {p: sha(ROOT / p) for p in complete},
    'preserved_production_and_test_hashes': preserved, 'next_read': next_read,
    'diagnostic_tests': 5, 'diagnostic_exit': 0,
    'package_tests': 'source read only; not run',
    'current_library_build': {'path': 'seiche_library_build.jsonl', 'sha256': sha(HERE / 'seiche_library_build.jsonl'), 'exit': 0},
    'generated_output_removed': all(p['generated_output_removed'] for p in probes), 'retained_isolation': [],
    'document': {'path': doc, 'sha256': sha(destination), 'preserved_prefix_bytes': len(before),
        'preserved_prefix_sha256': hashlib.sha256(before).hexdigest()},
    'diff_check': 0, 'evidence_sha256': {p: sha(HERE / p) for p in evidence},
    'limits': ['Full Mere and Seiche source coverage incomplete; no production fixes.',
        'Compiled default actor feature does not prove actor execution.',
        'Five generated native observations are not a full package suite.',
        'No GPU, renderer, model, headed UI or ignored diagnostic sweep executed.',
        'Barnes-Hut tuned near-body observation is not a default Charge failure.',
        'Broader lagged/contact-order/tick-demand/nonfinite-instrument candidates remain source qualified.']}
assert all(sha(ROOT / p) == h for p, h in preserved.items())
save('seiche_first_review_receipt.json', receipt)
print(json.dumps({'coverage': stats, 'seiche_complete': len(complete), 'findings': len(notes['findings']),
    'authority_entries': len(authority['entries']), 'next_read': next_read}))
