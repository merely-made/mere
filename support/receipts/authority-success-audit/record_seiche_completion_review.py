"""Save the completed Seiche reading checkpoint without changing product code."""
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

old = load('seiche_first_review_receipt.json')
probe = load('seiche_demand_probe_receipt.json')
assert probe['test_exit'] == 0 and probe['tests'] == 3 and probe['generated_output_removed']
assert load('pending_read.json') == []
preserved = {**old['preserved_production_and_test_hashes'], **old['source_sha256']}
assert all(sha(ROOT / p) == h for p, h in preserved.items())
assert all(sha(ROOT / p) == h for p, h in probe['source_sha256'].items())
assert all(sha(Path(v['path'])) == v['sha256'] for v in probe['library_sha256'].values())
assert sha(HERE / 'seiche_library_build.jsonl') == probe['library_build_log_sha256']
notes, authority = load('review_notes.json'), load('authority_map.json')
findings = [
    {'id': 'Q110', 'title': 'The tidal gravity locus stops when the backend tick budget ends',
     'status': 'reproduced inline and on the actual actor; documented, not repaired',
     'source': 'crates/conatus/seiche/src/overlays/gravity_locus.rs:Force impl; runtime.rs:should_tick; field.rs:wants_continuous_tick',
     'evidence': 'GravityLocus::tidal advances a private sinusoidal clock in apply but inherits wants_tick=false. Public Physics with one body and budget2 stops after two ticks, reports no further settling and remains frozen across subsequent advances. A delegating wrapper that changes only wants_tick to true continues through20 inline ticks. The actual offloaded actor also parks at two ticks while the wrapped control continues beyond the budget. Both actors were halted and dropped. The source test named a_tidal_locus_never_settles drives Simulation.tick manually and therefore never exercises backend demand.',
     'scope': 'Two generated native tests, one inline and one actual actor, with the identical force implementation behind a demand-only control. No hosted Canvas or GUI execution. An infinite manual tick loop is not evidence the normal runtime continues scheduling.',
     'diagnostic': 'seiche_demand_probe_receipt.json',
     'repair_triage': {'disposition': 'documented; active goal forbids production edits',
        'size': 'contained force-demand repair',
        'reason': 'Expose continuing demand for an active tidal locus. Qualify zero-amplitude/zero-strength and static locus behavior; do not make every static force perpetual.'}},
    {'id': 'Q111', 'title': 'Empty Density requests continuous ticks without being able to initialize',
     'status': 'reproduced through public inline Physics; documented, not repaired',
     'source': 'crates/conatus/seiche/src/laws/density.rs:apply early empty-node return and wants_tick; runtime.rs:should_tick',
     'evidence': 'Density.apply returns for no nodes before initializing state.flow; wants_tick treats None as pending work. With zero bodies and budget1, actual inline Physics runs180 of180 frames and still reports continuous demand with budget0. The populated Density control finishes, and an empty no-force control stops after its one budgeted tick. Explicit Halt stops the empty Density loop.',
     'scope': 'One generated native test using real Simulation and Physics. No empty Density actor or Canvas empty-graph execution. Distinct from Q102 repeated host affinity resettling: this loop arises inside force demand even without a host resettle call.',
     'diagnostic': 'seiche_demand_probe_receipt.json',
     'repair_triage': {'disposition': 'documented; active goal forbids production edits',
        'size': 'contained initialization/demand repair',
        'reason': 'Represent waiting for nodes separately from queued solver work, and re-arm when a later node sync provides input. Preserve initial populated convergence; avoid arbitrary timeouts.'}},
]
for value in findings:
    value['verification'] = 'seiche_completion_review_receipt.json'
upsert(notes['findings'], findings, 'id')

def entry(n, domain, claim, implementations, assessment):
    return {'id': f'A{n:03}', 'domain': domain, 'claim': claim,
        'implementations': implementations, 'assessment': assessment,
        'state': 'Seiche91/91 source read; three fresh native demand observations; no production edits',
        'verification': 'seiche_completion_review_receipt.json'}
entries = [
    entry(146, 'Active motion versus backend tick demand',
        'A tidal locus never comes to rest.',
        ['GravityLocus advances its own sinusoidal clock inside apply',
         'It inherits the default Force.wants_tick=false',
         'Shared backend should_tick admits budget, drag or continuous demand',
         'Its source test calls Simulation.tick directly'],
        'Q110 reproduces the gap inline and on the actual actor. Force evolution and scheduling declarations are separate authorities; the never-settles test proves only behavior under externally forced ticks.'),
    entry(147, 'Uninitialized work versus continuing work',
        'Density requests ticks until its flow converges.',
        ['Empty-node apply returns before flow initialization', 'wants_tick treats a missing flow as unconverged',
         'The shared scheduler accepts that demand after budget exhaustion'],
        'Q111 promotes the previous empty-Density candidate to a native inline finding. No-body waiting is falsely represented as runnable work. Populated/no-force/Halt controls separate it from the host affinity loop and from deliberate unbounded budget policy.'),
    entry(148, 'Inline and actor stepping authority',
        'Both adapters step the same simulation with the same demand and speed arithmetic.',
        ['runtime.rs shares should_tick', 'runtime/speed.rs shares step_owed across frame, elapsed and actor intervals',
         'Actor owns Simulation and its accepted-time clock',
         'Inline elapsed work is caller driven; actor advance_elapsed only drains snapshots'],
        'The repeated command adapters are real host/thread boundaries, not independent physical solvers. Step counts/durations and None budget semantics are intentionally different where documented. The native tide actor probe now proves actual actor execution for that slice; it does not qualify every actor test or elapsed adapter.'),
    entry(149, 'Actor evidence freshness and diagnostic absence',
        'Snapshots and evidence may be superseded; introspection is limited off-thread.',
        ['Generation barriers guard clock/reset/halt and configured evidence transitions',
         'Trace and membership mailboxes can explicitly report missing superseded generations',
         'Callbacks already execute on the simulation thread before delivery folding',
         'force_count/anchor_count/affinity_count/tick_demand and repulsion_stats are documented inline introspection'],
        'Do not read actor placeholder counts as proof no forces exist, or missing mailbox evidence as proof no callback ran. Latest-only position folding is distinct from durable action ownership. Controlled fabricated-producer tests exercise ordering; those are not additional actual actor receipts.'),
    entry(150, 'Portable declarations versus admitted execution',
        'Seiche owns portable specs; hosts bind catalogs and channels or refuse.',
        ['Spec defaults read actual force constructors', 'Derived currency/class/metric comes from constructed declarations',
         'Node.depth is iterative and capped before recursive derivation',
         'Raw terms construct declarations but current derive refuses raw execution',
         'settle.rs uses an owner-supplied runner and returns StepLimit distinctly'],
        'This is an explicit ownership seam, not a second admitted solver. Rich wire round trips include non-runnable shapes and do not prove runtime support. Host catalog bindings remain separate from portable data; ordinary raw parameter admission and eager construction still need qualification.'),
    entry(151, 'Common reference scale versus live geometry',
        'Force weights use one shared reference scale.',
        ['scale.rs defines fixed reference contact distance and unit length',
         'Force implementations expose per-term Scale/reweighted declarations',
         'Scale tests apply real forces at declared configurations with a mislabeled-exponent control',
         'CONTACTS declaration retains the default radius while actual Rapier shapes may resize'],
        'The fixed scale is explicitly a comparison configuration, not an authority for each live collider. Do not conflate it with the actual Q109 fluid geometry split or Q106 overlap metrics. Repeated per-law conversion mathematics are not independent reference constants.'),
    entry(152, 'Test names versus exercised state and catalog coverage',
        'Tests prove moving Hold, phase clusters, every-force reproducibility and GPU behavior.',
        ['Hold stop test pins/unpins an already stationary body without an actual shove',
         'Kuramoto test bounds within-community spread and orbit radius but discards all-phases data',
         'Reproducibility test checks12 selected force sets, excluding fluids, overlays and generic couplings',
         'GPU tests return successfully when adapter/device admission fails, with explicit skip messages',
         'Actual device tests include answer counters, a sign-flip control and injected submission failure'],
        'Names and comments exceed some exercised claims. Kuramoto source explicitly describes transient clustering and eventual synchronization; no stable-separation defect is inferred. A green GPU test count alone cannot prove GPU execution. Source-read tests were not freshly run; fresh demand evidence has three tests with explicit controls.'),
    entry(153, 'Physical proposals versus host fact commitment',
        'Physics proposes containment/support; the host owns commitment.',
        ['contact_to_fact integration tests define their own Placed and commit_release',
         'Pass-through proposals do not themselves mint facts',
         'Replay uses a test-local HashMap without Simulation',
         'Tactile tests exercise live material state, kind sieve and support geometry'],
        'These are concrete examples of correct ownership but not proof of a production graph log, durable release transaction or distributed replay. Remembered materials intentionally survive removal and re-sync of the same key; stale key reuse is a host identity question, not a demonstrated leak.'),
    entry(154, 'CPU/GPU mathematics and parameter defaults',
        'GPU staging computes the CPU NodeExclusion law.',
        ['Tensor NodeExclusion reference mirrors hard floor/cutoff and rejects invalid lengths',
         'Softened all-pairs reference is explicitly a different law',
         'NodeExclusionParams repeats strength/cutoff/min-distance default literals',
         'GPU staging materializes O(N squared) tensors and performs upload/readback',
         'Resident device lane is a separate execution path'],
        'Parity tests use tolerances and finite/length negative controls, not bit identity. Default literals are a small duplicated parameter authority to track, without a measured drift claim. Bounded staged timings cannot certify resident crossover or reuse of the host unified wgpu device.'),
    entry(155, 'Position read models versus physical shape authority',
        'Read geometry is approximate and can carry per-node radii.',
        ['LayoutView stores positions plus host-supplied radius overrides',
         'apply_snapshot updates positions/scene/fluid but does not carry node radii',
         'Simulation hit/cull follows fixed-radius conventions',
         'Edge distance and rectangle clipping helpers are actually shared',
         'Nearest node pick compares only squared distance while iterating a HashMap'],
        'The introductory uniform-circle prose is stale relative to overrides. Nearest-first overlap picking is not fully deterministic on equal-distance ties because no key tie-break is applied. Existing parity tests use well-separated default bodies, so they do not prove resized shape parity. This is a source qualification, without a fresh tie or host-radius synchronization probe.'),
]
upsert(authority['entries'], entries, 'id')
upsert(notes['candidates'], [
    {'title': 'Empty Density continues asking for ticks before it can initialize a flow',
     'source': 'crates/conatus/seiche/src/laws/density.rs:apply empty-node return; wants_tick',
     'note': 'Promoted to Q111 after a native public inline Physics reproduction:180 ticks, no bodies and exhausted budget. Populated/no-force/Halt controls pass. No actor or Canvas empty-graph claim.'},
    {'title': 'Raw declaration construction admits unbounded or lossy parameters before refusal',
     'source': 'crates/conatus/seiche/src/spec/raw.rs:RawTerm::build and parameter builders; spec.rs:derive',
     'note': 'Raw parameter f64-to-f32 conversions lack a common finite/range gate. Density resolution can allocate a resolution-squared grid during declaration construction even though raw execution is later refused. Anneal builder changes public initial_temperature after seeded construction initializes private temperature, so a custom starting temperature may not reseed state. No large allocation, OOM or raw execution was attempted. Review bounded declaration-only admission, allocation deferral and parameter fidelity separately; broad raw execution remains unadmitted.'},
], 'title')
inventory, coverage = load('source_inventory.json')['files'], load('read_coverage.json')
complete = sorted(p for p, v in inventory.items() if p.startswith('crates/conatus/seiche/')
    and coverage.get(p, {}).get('sha256') == v['sha256'] and coverage[p]['through'] == v['chars'])
assert len(complete) == 91
assert all(sha(ROOT / p) == inventory[p]['sha256'] for p in complete)
stats = json.loads(subprocess.check_output(['python', str(HERE / 'read_coverage.py'), 'status'], cwd=ROOT))
assert stats['complete_files'] == 846 and stats['read_chars'] == 12319189
assert not stats['changed_since_inventory'] and not stats['pending_chunks']
next_path = next(p for p, v in sorted(inventory.items()) if p.startswith('crates/armillary/')
    and coverage.get(p, {}).get('through', 0) < v['chars'])
next_read = {'path': next_path, 'start': coverage.get(next_path, {}).get('through', 0)}
notes['current_area'] = 'Seiche91/91 fully read; complete package source and three fresh demand observations'
notes['next_area'] = 'crates/armillary'
notes['checkpoint'] = {'classification': 'progress', 'goal_status': 'active; full-source objective incomplete',
    'coverage': stats, 'finding_count': len(notes['findings']), 'authority_entries': len(authority['entries']),
    'new_receipts': ['seiche_completion_review_receipt.json'], 'next_read': next_read,
    'diagnostic_outcomes': 'Q110 inline and actual actor; Q111 inline, with three generated native tests. No GPU/package suite.',
    'classification_of_previous_turn': 'progress', 'production_edits_this_turn': False,
    'coverage_checkpoint': stats, 'user_instruction': 'Audit all Mere source; preserve concurrent work; do not change production code.',
    'coverage_note': '51 additional complete files and586819 chars; Mere846/3352; Seiche91/91. Repository objective active.'}
upsert(notes['source_qualifications'], [{
    'area': 'Seiche completed source review and native demand observations',
    'note': 'All91 inventoried Seiche files read,1077251 chars. Three fresh native diagnostic tests pass against unchanged current Seiche CPU/default-actor library, including actual actor execution. Package suites, GPU, renderer, headed UI and ignored sweeps were not executed. Previously source-only empty-Density candidate is now Q111; tide scheduling is Q110.'
}], 'area')
assert len({x['id'] for x in notes['findings']}) == len(notes['findings'])
assert len({x['id'] for x in authority['entries']}) == len(authority['entries'])
save('review_notes.json', notes)
save('authority_map.json', authority)

doc = 'design_docs/mere_docs/implementation_strategy/2026-10-07_scenograph_editor_plan.md'
addition = '''

### Read-only Seiche complete source review, 2026-10-16

All 91 Seiche source files are now fully read: 1,077,251 characters. This
checkpoint adds 51 files and 586,819 characters, completing the remaining laws,
overlays, backend adapters, elapsed/speed handling, portable specs and wire
formats, shared scales, scenes, geometry views and test source. Mere coverage
is 846/3352 files and 12,319,189/36,854,820 characters. The full-source objective
remains active. Production is unchanged. Resume with crates/armillary.

Q110: GravityLocus::tidal promises continuing motion but inherits the default
wants_tick=false. Both inline Physics and the actual actor park at the end of
a two-tick budget. A wrapper delegating the unchanged force calculation while
returning true for demand keeps both paths running. The existing never-settles
test calls Simulation.tick manually, bypassing scheduling. The contained repair
is the tidal demand declaration, with explicit static/zero-drive policy. These
are public runtime observations, not hosted Canvas UI execution.

Q111: empty Density has the opposite fault. apply returns before initializing
its flow, and wants_tick treats missing flow as continuing work. Native inline
Physics keeps stepping for all180 frames with no bodies and budget0. Populated
Density finishes; empty no-force stops after one budgeted tick; Halt stops the
loop. This promotes the previously qualified empty-Density candidate. Distinguish
waiting for input from queued work and re-arm on later sync. No empty-Density
actor or product empty-graph path was exercised. This differs from Q102's host
affinity resettle loop.

Runtime review confirms actual shared should_tick and step_owed helpers. The
inline and actor command surfaces do not constitute separate physics solvers.
Actor elapsed calls drain updates rather than admitting caller time; diagnostic
counts marked inline-only are not evidence an actor has no forces. Snapshot
barriers and evidence mailboxes have explicit freshness/omission boundaries.
Callbacks execute before delivery folding; missing mailbox evidence does not
certify absence of an action. Fresh actor execution is now established for Q110,
superseding only that limitation of the previous checkpoint.

Portable specs derive declarations from actual constructors, cap iterative
depth before recursive derivation and refuse unbound raw execution. Rich wire
round trips include shapes that are deliberately not runnable. The generic
settle loop uses an owner-provided runner and reports StepLimit explicitly.
These correct boundaries must survive cleanup. Raw parameter builders still
cast floats without common admission, may eagerly allocate Density grids before
refusing execution, and may change Anneal's public starting temperature after
its private state was seeded. These are source-qualified follow-ups; no large
allocation or raw runtime was attempted. General admission and allocation
ownership should not prolong this audit beyond the user's one-hour limit.

Test claims need tighter scope. Hold's moving-stop test supplies no actual shove;
Kuramoto checks within-group spread and radius without an inter-group separation
assertion; every-force reproducibility covers12 selected sets rather than fluid,
all overlays or all couplings. GPU tests explicitly print skips and return when
device admission fails. A successful test count therefore cannot itself prove
device execution. They do contain useful device counters, sign-flip and injected
submission-failure controls. None of those package tests was freshly executed.
Host-local contact-to-fact examples prove their own commitment discipline, not
a production durable graph transaction or distributed replay implementation.

The shared force scale deliberately uses a fixed reference configuration. That
is distinct from a live collider geometry authority. Tensor law defaults retain
a small literal copy of CPU defaults; no actual parameter drift was measured.
Softened all-pairs repulsion is explicitly a separate law from NodeExclusion.
Staged O(N squared) GPU timings include upload/readback and do not qualify a
resident crossover or unified host-device reuse. LayoutView really shares edge
geometry helpers, but its circle/radius map remains host-maintained and snapshots
do not carry node radii. Its deterministic-overlap wording omits equal-distance
HashMap ties; source parity tests use separated default bodies only.

Hairier issues remain documented: fluid contact order, lagged answer parameter
identity, generic coupling pause demand and non-finite diagnostic admission.
Three generated demand tests passed with the existing current CPU library;
the actual actors were halted/dropped and the owned executable/PDB removed.
No GPU, renderer, headed UI or full package suite ran. No isolated target,
Cargo home or worktree exists. Evidence and preservation hashes are in the
[Seiche completion receipt](../../../support/receipts/authority-success-audit/seiche_completion_review_receipt.json).
'''
destination = ROOT / doc
before = destination.read_bytes()
assert b'### Read-only Seiche complete source review, 2026-10-16' not in before
assert hashlib.sha256(before[:old['document']['preserved_prefix_bytes']]).hexdigest() == old['document']['preserved_prefix_sha256']
destination.write_bytes(before + addition.encode())
diff = subprocess.run(['git', 'diff', '--check'], cwd=ROOT, capture_output=True, text=True)
(HERE / 'seiche_completion_diff_check.txt').write_text(diff.stdout + diff.stderr, encoding='utf-8')
assert diff.returncode == 0
evidence = ['seiche_demand_probe.rs', 'run_seiche_demand_probe.py',
    'seiche_demand_probe_build.log', 'seiche_demand_probe_output.log',
    'seiche_demand_probe_receipt.json', 'seiche_completion_diff_check.txt']
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(),
    'basis_commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
    'kind': 'Complete Seiche source review; full Mere source objective incomplete',
    'production_edits_this_pass': False, 'new_findings': [x['id'] for x in findings],
    'authority_entries': [x['id'] for x in entries], 'coverage': stats,
    'complete_seiche_files': complete,
    'added_complete_files': sorted(set(complete) - set(old['complete_seiche_files'])),
    'source_sha256': {p: sha(ROOT / p) for p in complete},
    'preserved_production_and_test_hashes': preserved, 'next_read': next_read,
    'diagnostic_tests': 3, 'diagnostic_exit': 0, 'actual_actor_execution': True,
    'package_tests': 'source read only; not run', 'generated_output_removed': True,
    'retained_isolation': [],
    'document': {'path': doc, 'sha256': sha(destination), 'preserved_prefix_bytes': len(before),
        'preserved_prefix_sha256': hashlib.sha256(before).hexdigest()},
    'diff_check': 0, 'evidence_sha256': {p: sha(HERE / p) for p in evidence},
    'limits': ['Full Mere source coverage incomplete; no production fixes.',
        'Three diagnostic tests are not a full package suite.',
        'Actual actor execution qualified only the tidal demand slice.',
        'No GPU, renderer, headed UI or ignored sweep executed.',
        'Other unprobed candidates remain source qualified.']}
assert all(sha(ROOT / p) == h for p, h in preserved.items())
assert all(sha(ROOT / p) == h for p, h in receipt['source_sha256'].items())
save('seiche_completion_review_receipt.json', receipt)
print(json.dumps({'coverage': stats, 'seiche_complete': len(complete), 'findings': len(notes['findings']),
    'authority_entries': len(authority['entries']), 'next_read': next_read}))
