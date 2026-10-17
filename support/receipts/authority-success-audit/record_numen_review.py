"""Record completed Numen source reading and actual default/native evidence."""
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

old = load('armillary_review_receipt.json')
probe = load('numen_core_probe_receipt.json')
assert probe['test_exit'] == 0 and probe['tests'] == 3 and probe['generated_output_removed']
assert probe['suite_exit'] == 0 and probe['library_build_exit'] == 0
assert '65 passed; 0 failed' in (HERE/'numen_default_suite.log').read_text()
assert load('pending_read.json') == []
preserved = {**old['preserved_production_and_test_hashes'], **old['source_sha256']}
assert all(sha(ROOT / p) == h for p, h in preserved.items())
assert all(sha(ROOT / p) == h for p, h in probe['source_sha256'].items())
assert all(sha(Path(v['path'])) == v['sha256'] for v in probe['library_sha256'].values())
notes, authority = load('review_notes.json'), load('authority_map.json')
findings = [
    {'id': 'Q114', 'title': 'Analytic field gradients ignore spatial variation of Gaussian centres and linear normals',
     'status': 'reproduced through current public Numen evaluator; documented, not repaired',
     'source': 'crates/conatus/numen/src/eval.rs:grad_scalar Gaussian and Linear; field_ast.rs:Gaussian.center and Linear.normal',
     'evidence': 'The portable AST allows arbitrary vector expressions for center and normal. Linear normal=Coord evaluates x*x+y*y, but grad_scalar at(3,4) returns(3,4), not(6,8). Registering that same scalar and differentiating Sample switches to finite differences and returns(5.9995646,8.000374). Gaussian center=Coord+(1,0), sigma10 has constant value0.99501246, yet the analytic gradient returns(0.009950125,0) while finite differences return(0,0). Constant-normal and fixed-center controls are correct. The analytic branches omit the derivative of their vector child; Burn restricts those branches to ConstVec and refuses unsupported dynamic forms.',
     'scope': 'Two generated native tests against a freshly built default Numen library; public AST/evaluation/registry APIs only. Seiche CouplingForce source calls this gradient for scalar physical responses, but no fresh Seiche/Canvas force or GPU execution was performed. Default Numen library suite65 tests passes and does not cover these dynamic-child cases.',
     'diagnostic': 'numen_core_probe_receipt.json',
     'repair_triage': {'disposition': 'documented; active goal forbids production edits',
        'size': 'contained analytic eligibility correction; generalized derivatives broader',
        'reason': 'Use the constant-child formula only when spatial constancy is established, otherwise use the existing finite-difference path or typed refusal consistent with the evaluator contract. A shared derivative/AST analysis across CPU, Burn, animation detection and budget accounting can exceed the user one-hour limit; document that larger work.'}},
    {'id': 'Q115', 'title': 'Named registry allocation silently overwrites an explicit field identity',
     'status': 'reproduced through public FieldRegistry API; documented, not repaired',
     'source': 'crates/conatus/numen/src/registry.rs:insert_with_id, assign_id, insert_scalar/insert_vector',
     'evidence': 'insert_with_id seeds the fields map without updating or reserving against next_id. On a fresh registry, explicit valid FieldId UUID0 containing Const7 is replaced when insert_scalar("new",Const9) mints UUID0. The new field returns the imported id, len remains1 and an existing Sample changes from7 to9. Fresh named entries get distinct ids, and same-name update deliberately reuses its id in controls. The bug is unrelated-name allocation colliding with the explicitly seeded identity domain.',
     'scope': 'One generated native test using unchanged public API and a valid nil UUID; no reserved-nil rule exists in the inspected type. Both paths are public but no single production host mixing them was executed. Graph-to-Seiche bridge source seeds explicit IDs; Canvas authoring helpers use named allocation on projections. Do not claim a reproduced production graph overwrite.',
     'diagnostic': 'numen_core_probe_receipt.json',
     'repair_triage': {'disposition': 'documented; active goal forbids production edits',
        'size': 'contained registry allocation repair',
        'reason': 'Skip already occupied explicit IDs when allocating a fresh name, or separate/admit identity domains while retaining canonical graph IDs. Preserve intentional same-name updates and WASM-safe deterministic allocation; cover imported nonzero IDs and mixed insertion order when repaired.'}},
]
for value in findings:
    value['verification'] = 'numen_review_receipt.json'
upsert(notes['findings'], findings, 'id')

def entry(n, domain, claim, implementations, assessment):
    return {'id': f'A{n:03}', 'domain': domain, 'claim': claim,
        'implementations': implementations, 'assessment': assessment,
        'state': 'Numen16/16 source read;65 default package tests and3 generated observations; no production edits',
        'verification': 'numen_review_receipt.json'}
entries = [
    entry(160, 'Scalar value versus declared gradient',
        'Known kernels use analytic gradients; arbitrary compositions fall back to finite differences.',
        ['Gaussian.center and Linear.normal permit arbitrary vector expressions',
         'eval_scalar actually samples those children at each point',
         'grad_scalar applies formulas assuming their spatial derivative is zero',
         'Wrapping the same expression in Sample forces a different gradient path'],
        'Q114 proves evaluation and differentiation disagree on the same portable expression. Expression routing changes force semantics. Existing constant-child tests cannot qualify the arbitrary-child analytic branch. Direct vector Gradient and physical CouplingForce inherit that evaluator by source, without a fresh host proof.'),
    entry(161, 'Canonical UUID identity versus local counter allocation',
        'Registry uses canonical FieldId and duplicate names intentionally keep identity.',
        ['FieldId is the same UUID-backed type in graph truth and registry',
         'insert_with_id imports canonical IDs without registering names',
         'assign_id mints from next_id without checking occupied keys',
         'Rhai handles assume local small counter IDs'],
        'Q115 demonstrates colliding allocation authorities, not duplicated FieldId types. Explicit overwrite of the same supplied id is documented; unrelated name admission overwriting that id is not. Local counter IDs are not globally federatable identities merely because their storage type is UUID.'),
    entry(162, 'Portable field types and host graph authority',
        'One portable AST and field identity are shared by consumers.',
        ['lib.rs reexports field_ast as ast rather than copying enums',
         'registry reexports FieldDefinition as FieldDef and canonical FieldId',
         'FieldProjection is a per-view registry/coupling/path bundle',
         'Former commit_to_graph bridge is removed; graph writes belong to the host'],
        'These are actual shared types and an intentional evaluation projection, not a second durable graph owner. Older kernel/crate module links and Phase3b future-minting prose remain stale. Names, lifecycle and extent are not all enforced by a bare registry; hosts must supply those policies.'),
    entry(163, 'Reference errors and evaluator refusal contracts',
        'Missing or wrong-typed samples have explicit backend behavior.',
        ['CPU eval returns0 or(0,0), explicitly documented',
         'Burn returns UnknownField or SampleTypeMismatch',
         'Burn refuses nonconstant centers/normals and unavailable gradients',
         'CPU eval recursively follows samples without general cycle/depth admission',
         'Responsive path has its own transitive cycle/depth/node/work checker'],
        'Backend error behavior is intentionally different and cannot be summarized as universal successful evaluation. Responsive-path refusal tests now actually pass, but their private validator does not protect direct evaluation or every Seiche consumer. Cycle policy and common bounded admission remain broader follow-ups.'),
    entry(164, 'Responsive path scope and shared evaluation',
        'P5 geometry is bounded, deterministic, view-local output, with fixed endpoints.',
        ['Evaluator calls the common eval_vector rather than copying field math',
         'Source validates bounds, finite inputs, sample expansion, work and output',
         'Compatible attraction uses endpoint geometry and preserves supplied identity labels',
         'Field/attraction switches are independent; disabled field ASTs are deliberately not evaluated'],
        'All default responsive tests ran with the package suite, including cycle-hidden-in-scale, nested gradient budget, geometry relabel/reversal and nonfinite controls. This qualifies bounded local geometry only; it is not graph commitment, a production renderer, or a physics integrator receipt. Q114 remains reachable through the shared evaluator despite path resource validation.'),
    entry(165, 'CPU/Burn operator coverage and device ownership',
        'The optional backend lowers field expressions into tensor operations.',
        ['CPU analytic evaluator and Burn lowerer repeat supported formulas',
         'Burn product-gradient rule is analytic while CPU Mul uses finite differences',
         'Production lowering accepts caller-created tensors and does not mint a device',
         'WGPU tests select DiscreteGpu0 separately from NdArray and compare257 points',
         'Timing includes upload/readback and warmup; ignored by default'],
        'Distinct backend mathematics are legitimate implementations requiring parity, not proof of a single executable evaluator. WGPU-as-future module prose is stale because the feature and test source exist. No Burn/Rhai/WGPU tests were freshly executed. Claims of fused execution or host unified-device integration are not established by AST-to-tensor source or default tests.'),
    entry(166, 'Rhai authoring success versus admitted runtime configuration',
        'build_from_script returns a FieldProjection when the script evaluates successfully.',
        ['BuildError has only EvalError',
         'Numeric constructors cast f64 to f32 without common finite admission',
         'edge_path_field_line casts i64 max_steps to u32 directly',
         'Field IDs pass through small signed handles and coupling IDs through local counts',
         'Script tests check authored shape and counts rather than executing a graph/field-line host'],
        'A built projection is authored data, not evidence every numeric parameter or reference is runnable. Negative/oversized step casts and nonfinite float admission are qualified candidates; no fresh Rhai or consumer runtime reproduced them. The documentation example supplies a scalar id to a field-line declaration even though the enum describes vector tracing.'),
    entry(167, 'Extent geometry and open response boundaries',
        'Numen answers portable geometry and preserves unrecognized response data.',
        ['AttachedToNode contains/boundary_distance explicitly return None for host resolution',
         'Polygon containment checks finite points, degeneracy and inclusive edges in f64',
         'Rectangle boundary_distance uses minimum signed axis clearance',
         'Open responses retain predicate IRIs while recognized IRIs canonicalize via shared methods',
         'DampenInside.from_iri restores factor1 and documents parameter data separately'],
        'These limits are mostly honest. Rectangle outside-corner distance is not Euclidean distance; source does not prescribe that metric. Polygon distance calculations still use f32 despite f64 containment. No new geometry defect was executed. Open ignored by force is explicit lack of a recognized consumer, not a hidden successful semantic action.'),
]
upsert(authority['entries'], entries, 'id')
upsert(notes['candidates'], [
    {'title': 'General field evaluation lacks the responsive path cycle/work admission seam',
     'source': 'crates/conatus/numen/src/eval.rs; lower_burn.rs; registry.rs; responsive_path.rs:validate_*',
     'note': 'Public registry methods admit self- or mutually sampled definitions, and general CPU/Burn evaluation recursively follows them without a shared cycle or work bound. Responsive geometry has a private guarded traversal with explicit cycle/work/depth errors and passed its native package controls. Do not extend that receipt to other callers. No deliberate stack overflow or process abort was attempted. Shared admission/diagnostics and late registry edits can exceed the one-hour audit diversion; document scope instead of redesigning here.'},
    {'title': 'Rhai projection construction does not admit runtime numeric ranges',
     'source': 'crates/conatus/numen/src/rhai_bindings.rs:register_constructors and register_edge_path_methods',
     'note': 'f64-to-f32 casts can create nonfinite values; signed max_steps casts can wrap. build_from_script Ok only certifies Rhai evaluation to a DTO. No fresh field-rhai build, malformed script or production field-line consumer was executed. Keep authoring/data construction distinct from admitted runtime behavior.'},
], 'title')
inventory, coverage = load('source_inventory.json')['files'], load('read_coverage.json')
complete = sorted(p for p,v in inventory.items() if p.startswith('crates/conatus/numen/')
    and coverage.get(p,{}).get('sha256') == v['sha256'] and coverage[p]['through'] == v['chars'])
assert len(complete) == 16 and sum(inventory[p]['chars'] for p in complete) == 150254
assert all(sha(ROOT/p) == inventory[p]['sha256'] for p in complete)
stats = json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files'] == 868 and stats['read_chars'] == 12501105
assert not stats['changed_since_inventory'] and not stats['pending_chunks']
next_path = next(p for p,v in sorted(inventory.items()) if p.startswith('crates/conatus/nisus/')
    and coverage.get(p,{}).get('through',0) < v['chars'])
next_read = {'path':next_path,'start':coverage.get(next_path,{}).get('through',0)}
notes['current_area'] = 'Numen16/16 fully read; default evaluator, Burn/Rhai source, registry and responsive geometry reviewed'
notes['next_area'] = 'crates/conatus/nisus'
notes['checkpoint'] = {'classification':'progress','goal_status':'active; full-source objective incomplete',
    'coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),
    'new_receipts':['numen_review_receipt.json'],'next_read':next_read,'production_edits_this_turn':False,
    'diagnostic_outcomes':'65 default Numen package tests and3 native observations pass; Q114/Q115 reproduced.',
    'classification_of_previous_turn':'progress','coverage_checkpoint':stats,
    'user_instruction':'Audit all Mere source; preserve concurrent work; do not change production code.',
    'coverage_note':'16 more full files and150254 chars. Full Mere review remains active.'}
upsert(notes['source_qualifications'], [{
    'area':'Numen complete source review and native evaluator/registry evidence',
    'note':'All16 inventoried files read,150254 chars. Fresh current default library builds,65 default library tests pass,3 generated native defect observations pass with fixed-child and ordinary identity controls. No Burn/Rhai/GPU/renderer/hosted graph or physical coupling execution. Generated diagnostic executable/PDB removed; ordinary stable Cargo build cache retained.'
}], 'area')
save('review_notes.json',notes)
save('authority_map.json',authority)
doc = old['document']['path']
destination = ROOT/doc
before = destination.read_bytes()
assert sha(destination) == old['document']['sha256']
addition = '''

### Read-only Numen complete source review, 2026-10-16

All16 Numen source files are fully read,150,254 characters, including portable
types, default evaluation, registry/projection, Burn lowering and its tests,
Rhai authoring, responsive-path geometry and its tests/example. Mere coverage
is868/3352 files and12,501,105/36,854,820 characters. The full-source goal remains
active. Production/concurrent work are unchanged. Resume at crates/conatus/nisus.

Q114: the default analytic gradient assumes Gaussian centres and linear normals
are spatially constant although the AST admits arbitrary vector expressions.
Linear(normal=Coord) is x*x+y*y: at(3,4) evaluation is25 but its gradient is(3,4),
half the correct(6,8). Sampling that same registered scalar selects finite
differences and returns(5.9995646,8.000374). Gaussian(center=Coord+(1,0),sigma10)
has constant value0.99501246 but returns gradient(0.009950125,0), versus measured
finite difference(0,0). Fixed-centre and constant-normal controls work. These
two native observations prove value/gradient disagreement and expression-route
dependence. Seiche's physical scalar response calls this evaluator by source;
no fresh physical coupling or hosted Canvas execution is claimed. Burn already
refuses dynamic centre/normal forms instead of applying those closed forms.
The contained repair is analytic eligibility with finite-difference fallback.
A general shared derivative/AST-analysis design is a separate larger issue.

Q115: explicit canonical IDs and named local allocation collide. insert_with_id
does not reserve against next_id. Seeding valid UUID0 with Const7 then adding
an unrelated named Const9 returns the same id, leaves len1 and changes an
existing Sample from7 to9. Ordinary distinct-name and same-name-update controls
behave correctly. One native test proves the public registry defect; a product
graph overwrite was not exercised. Skip occupied explicit IDs or admit separate
identity domains while preserving same-name updates and canonical graph IDs.
Graph-to-Seiche source seeds explicit IDs; authoring helpers allocate names,
but their coexistence in one live host was not demonstrated.

This package contains real sharing: ast is a reexport, FieldDef aliases the same
FieldDefinition, and the registry keys canonical FieldId. FieldProjection is a
view-local evaluation bundle. The old commit_to_graph bridge is gone; host graph
writes remain host-owned. Those are useful boundaries, not extra durable truth.
CPU missing/wrong-typed samples explicitly evaluate to zero; Burn returns typed
errors. Backend refusal contracts differ and cannot be flattened into one
unqualified success claim. The lowerer accepts caller tensors and mints no device.
WGPU-as-future prose is stale because the opt-in feature and parity tests exist;
that source does not prove current unified host-device integration or fusion.

The current default Numen library built successfully, and all65 default library
tests passed. They include responsive-path limits, recursive sample refusal,
hidden scalar-cycle checks, nested gradient cost, nonfinite controls and geometry
relabel/reversal. All three additional native defect observations also passed.
No Burn/Rhai/WGPU suite, renderer, hosted graph or physical coupling ran. Reading
optional backend test source is not a fresh backend qualification. The generated
diagnostic executable/PDB was removed; normal cache remains at
C:/t/cargo-targets/mere. No isolated target, Cargo home or worktree was created.

Hairier follow-ups remain documented: the responsive path's private cycle/depth/
work validator is not shared by direct CPU/Burn evaluation or every Seiche caller;
public registries can store recursive references. No deliberate stack overflow
was attempted. General admission, late registry edits and consistent diagnostics
can exceed the user's one-hour audit diversion. Rhai construction casts floats
and signed step counts without a common runtime-range gate; script success is
authored-data success only. No malformed Rhai input or field-line runtime was
freshly exercised. Its example even declares a scalar id for vector field-line
tracing, so it should not be presented as an executed rendering receipt.

Preservation hashes, build/suite output and exact diagnostic scope are in the
[Numen review receipt](../../../support/receipts/authority-success-audit/numen_review_receipt.json).
'''
assert b'### Read-only Numen complete source review, 2026-10-16' not in before
destination.write_bytes(before+addition.encode())
diff = subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True,text=True)
(HERE/'numen_review_diff_check.txt').write_text(diff.stdout+diff.stderr,encoding='utf-8')
assert diff.returncode == 0
evidence = ['numen_core_probe.rs','run_numen_core_probe.py','numen_library_build.jsonl',
    'numen_library_build_stderr.log','numen_default_suite.log','numen_core_probe_build.log',
    'numen_core_probe_output.log','numen_core_probe_receipt.json','numen_review_diff_check.txt']
receipt = {'date_utc':datetime.now(timezone.utc).isoformat(),
    'basis_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
    'kind':'Complete Numen source review; full Mere source objective incomplete',
    'production_edits_this_pass':False,'new_findings':[x['id'] for x in findings],
    'authority_entries':[x['id'] for x in entries],'coverage':stats,'complete_numen_files':complete,
    'source_sha256':{p:sha(ROOT/p) for p in complete},'preserved_production_and_test_hashes':preserved,
    'next_read':next_read,'diagnostic_tests':3,'diagnostic_exit':0,'default_library_tests':65,
    'default_library_test_exit':0,'generated_output_removed':True,'retained_isolation':[],
    'document':{'path':doc,'sha256':sha(destination),'preserved_prefix_bytes':len(before),
        'preserved_prefix_sha256':hashlib.sha256(before).hexdigest()},
    'diff_check':0,'evidence_sha256':{p:sha(HERE/p) for p in evidence},
    'limits':['Full Mere source coverage incomplete; no production fixes.',
        'Default library test suite does not include optional Burn or Rhai.',
        'No GPU, renderer, graph host or physical coupling execution.',
        'Broader cycle, work, numeric-admission and identity consumer paths remain qualified.']}
assert all(sha(ROOT/p)==h for p,h in preserved.items())
assert all(sha(ROOT/p)==h for p,h in receipt['source_sha256'].items())
save('numen_review_receipt.json',receipt)
print(json.dumps({'coverage':stats,'numen_complete':len(complete),'findings':len(notes['findings']),
    'authority_entries':len(authority['entries']),'next_read':next_read}))
