"""Read-only solver/catalog audit checkpoint. Diagnostics are not repairs."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib
import json
import subprocess

HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8'))
def save(n,v): (HERE/n).write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8')
def upsert(items,values,key):
    for value in values:
        at=next((i for i,item in enumerate(items) if item.get(key)==value[key]),None)
        if at is None: items.append(value)
        else: items[at]=value

old=load('scenograph_review_receipt.json')
probe=load('scenomise_solver_probe_receipt.json')
assert probe['test_exit']==0 and probe['tests']==8 and probe['generated_output_removed']
assert all(sha(ROOT/p)==h for p,h in probe['source_sha256'].items())
assert all(sha(ROOT/p)==h for p,h in old['source_sha256'].items())
assert all(sha(ROOT/p)==h for p,h in old['preserved_production_and_test_hashes'].items())
assert load('pending_read.json')==[]
notes,authority=load('review_notes.json'),load('authority_map.json')
def finding(number,title,path,evidence,scope,repair):
    return {'id':f'Q{number:03}', 'status':'documented from current source and native public API diagnostic; not repaired',
            'title':title,'source':'crates/cambium/scenes/scenomise/src/'+path,
            'evidence':evidence,'scope':scope,'verification':'scenomise_review_receipt.json',
            'diagnostic':'scenomise_solver_probe_receipt.json',
            'repair_triage':{'disposition':'documented; current continuation is read-only','reason':repair}}
findings=[
 finding(74,'Timeline narrows a valid numeric axis before normalization, producing NaN or collapsing distinct values','families/axial.rs:timeline',
    'The axis stores f64 but span and numerator are cast to f32 before division. A finite score [0,1e100] serializes/reopens correctly, then solve emits NaN for the second x. SceneSnapshot::from_dense accepts it; emitted snapshot JSON cannot reopen. Finite [0,1e-100] instead collapses both endpoints to origin. Normal [0,100] maps to both axis endpoints and snapshot round-trips.',
    'Current public Score/solve/snapshot APIs with generated finite input and default finite layout. No renderer or product data import. This connects a real producer path to the separately observed Q069 missing finite admission.',
    'Normalize in a sufficiently wide representation before narrowing the bounded result, including extreme finite endpoints whose subtraction can itself overflow. Qualify nonzero tiny spans, endpoint mapping and finite output, then enforce shared output admission. Do not change valid input facts to hide the defect.'),
 finding(75,'A host-named L-system grammar silently realizes as Hilbert','families/lsystem.rs:resolve_grammar',
    'Sceno declares Named(id) to mean production rules supplied by the host. resolve_grammar maps every Named value to HILBERT. Public solve at explicit depth 2 returns exactly the same complete Scene for Named("missing-host-grammar") and Hilbert, with no unavailable-grammar result. The existing unit test intentionally checks that this fallback walks.',
    'Intentional source fallback with misleading success semantics at the public Score boundary, not an accidental typo in the native test. The authored catalog offers only the three built-in grammars and refuses other option names; a Rust caller search found no product Named construction. No live editor claim.',
    'Give host grammar lookup an actual seam, or explicitly refuse/report unavailable named grammar. A visible configured fallback can be a separate policy. Keep it distinct from whole-arrangement Custom resolution.'),
 finding(76,'Explicit fractal depth passes range validation without an expansion budget','families/lsystem.rs:expand; families/penrose.rs:expand_tiling',
    'OptionKind::Depth and both catalog constructors accept 255 for one item. Automatic L-system depth is capped at 10 and Penrose at 8, but explicit depth is passed directly into exponential expansion loops. A bounded symbol-count recurrence over the exact Hilbert rules gives 3665038759251 symbols and 1099511627775 forward steps at depth 20; no expansion string was allocated.',
    'Admission executed for L-system and Penrose; expansion-cost analysis is source/mathematical evidence. Hazardous depth-255 solving was deliberately not executed. This is not an observed browser crash or measured peak-memory receipt.',
    'Separate syntactic u8 validity from configurable symbol/tile/output and work budgets. Preflight both explicit and automatic expansion and return bounded refusal/cancellation through compiler/host boundaries. This policy and error-path change is broader than a one-hour audit diversion.'),
 finding(77,'Negative fractional radial indices round into the focus rather than the unreachable path','families/radial.rs:ring_of',
    'ring_of checks nonnegativity after round(). Public solve with a ring-2 control and -1 sends the negative item to the outer fallback; replacing -1 with finite -0.1 places it exactly at the focus. The source contract says negative rings are unassigned rather than silently moved to the center.',
    'Native public solver with default Radial policy; no renderer or source graph mutation. Fractional negative input is admitted by the public numeric-axis type.',
    'Check finite/nonnegative representability before rounding; make any rounding convention explicit and preserve the unreachable policy for negative values.'),
 finding(78,'Radial outer-ring fallback overflows after the maximum representable ring','families/radial.rs:place',
    'ring_of can return u32::MAX from a valid finite axis. The outer fallback computes deepest + 1 without checking. Public solve with [u32::MAX, undisclosed] panics in the current native overflow-checked library; the diagnostic catches that panic. Float-to-u32 saturation also lets larger finite values reach the same boundary.',
    'Native debug-profile panic, not an executed release/wasm profile. With unchecked overflow the source expression can wrap; no release runtime result is claimed.',
    'Use checked ring-domain admission and checked outer-ring selection. Choose/report a bounded fallback when no outer ring is representable instead of inventing a focus position or panicking.'),
 finding(79,'Sets placement silently relaxes the membership meaning when fixed regions cannot realize it','families/setscape.rs:place',
    'The module claims position means membership, yet relaxes tier then exclusion and finally returns the first circle center. A six-set fixture with one item in all six retains that membership in every region, but its returned position lies geometrically inside only one region. The Scene supplies no degraded-membership placement report.',
    'Native public solver/returned polygons only. Region membership data and source facts stay correct; the defect is contradictory spatial presentation. No browser rendering or inference that source membership was overwritten.',
    'Define how impossible intersections are disclosed: a visible unmet-placement report or a representation that can realize the memberships. Fixed circles, strict spatial truth and arbitrary set membership cannot all be promised together. Preserve source memberships; do not normalize them to fit the drawing.'),
]
upsert(notes['findings'],findings,'id')
q69=next(item for item in notes['findings'] if item['id']=='Q069')
q69['related_producer_evidence']='Q074 now supplies a finite, JSON-round-trippable Score that produces NaN through public Timeline solve and reaches the same admitted/non-reopenable snapshot path; scenomise_solver_probe_receipt.json.'
def entry(number,domain,claim,implementations,assessment):
    return {'id':f'A{number:03}','domain':domain,'claim':claim,'implementations':implementations,
            'assessment':assessment,'state':'source reviewed; eight bounded native diagnostics recorded; no production repair',
            'verification':'scenomise_review_receipt.json'}
entries=[
 entry(80,'Arrangement option declarations versus duplicated value parsers','One declaration drives refusal and editor rows so they cannot drift.',
       ['Family.options supplies keys, kinds and defaults','Options.finish reuses declaration keys','Options independently parses f32/u32/u8/flags/choices/lists','OptionKind.refusal separately repeats these predicates','Catalog tests compare declared keys to read keys, not all parser outcomes'],
       'Key and choice-table sharing is real. Value validation has two implementations. Current reviewed branches agree; no parser divergence is established. Measured defaults also bypass authored-value parsing, so declarations alone are not full numeric/output admission.'),
 entry(81,'Axis precision and portable geometry admission','Valid numeric disclosure should map into a finite configured axis.',
       ['AxisValue stores f64','Timeline normalizes via premature f32 casts','Scene construction accepts computed coordinates','Snapshot structural validation permits the resulting NaN'],
       'Q074 and Q069 expose an end-to-end gap across producer and admission. A parseable input and successful structural validation do not prove a portable output. Ordinary axis control passes.'),
 entry(82,'Named grammar ownership and unavailable-implementation fallback','Named grammar production rules come from the host.',
       ['Sceno carries the Named identifier','L-system resolves every Named identifier as Hilbert','Plain solve refuses unknown Custom placement by emitting no items/held-pin report','The authored grammar picker exposes built-ins only'],
       'Q075 is an intentional silent substitution inconsistent with carrying the requested host grammar as an executable choice. Named and Custom have different fallback rules; no shared host grammar authority currently connects this named identifier to supplied production rules.'),
 entry(83,'Stateless solvers versus bounded work','Every solver is described as closed-form, one pass, with no state between calls.',
       ['L-system expands a grammar repeatedly then walks it','Penrose repeatedly subdivides triangles then deduplicates/sorts vertices','Mosaic recursively bisects weights','Sets uses up to three rejection-sampling phases','Automatic depth has local caps while explicit depth lacks a resource budget'],
       'No retained frame state is true; it does not imply one linear pass or bounded cost. Q076 demonstrates that syntactic option validity is a different authority from work/memory admission.'),
 entry(84,'Declared membership and position-derived visual meaning','A Sets item sits inside every member set and outside all others.',
       ['membership lists remain in returned regions','The sampler progressively relaxes geometric constraints','Terminal fallback returns the first circle center','No unmet membership report accompanies the returned Scene'],
       'Q079 shows semantic source truth and spatial presentation can contradict each other despite success. Source ownership remains intact; successful position generation is weaker than membership-faithful realization.'),
 entry(85,'Shared scene construction and specialized sizing/geometry','Interning, bounds, hold reporting and Hulls construction stay in one realizer.',
       ['solve_with centralizes source interning, placed items and hold reports','Family plans supply positions and optional footprints/regions','Mosaic.weight_of and Partition.size_of repeat the same usable-weight predicate','Mosaic.corners and Partition.rect_points repeat rectangle contour construction','Facet merge separately remaps source/space/item/fold/hold references'],
       'Scene construction reuse is real. Small duplicate geometry/weight helpers remain; no valid-input divergence is observed there. Facet remapping is a composition operation, not automatically another source-authority owner. Raw Scene/layout validity and reflection resolution are not established by local Facet.validate.'),
]
upsert(authority['entries'],entries,'id')
upsert(notes['source_qualifications'],[
 {'area':'Scenomise catalog/solver continuation','note':'Catalog, document, facet and tests, root lib, solve and every families module fully read at current hashes. Registry, projection, host-dataset, history, template, relaxation and remaining integration source are still ahead. Local source-generation composition is not a content hash of every facet/layout field.'},
 {'area':'Scenomise solver diagnostics','note':'Freshly built public libraries support eight native diagnostics. Seven characterize defects/unsafe boundaries; the ordinary finite Timeline control reopens. Named fallback and depth acceptance are intentional current code paths. Large expansions, release overflow, renderer, live editor, network and user archives were not executed. Successful diagnostics are not correctness gates or repairs.'},
 {'area':'Facet and document admission limits','note':'SceneDocument.validate compiles primary and every linked reading before admission and applies a byte/count cap; this is stronger than raw SceneInput.validate. Facet.compose validates facet structure and cell-name availability, then assumes supplied scenes have usable references; it does not resolve reflection graphs. No infinite reflection execution is established.'}
],'area')
upsert(notes['candidates'],[
 {'title':'Embedded outside-ring radius can overflow intermediate squared coordinates','source':'crates/cambium/scenes/scenomise/src/families/embedded.rs:ring_radius','note':'The radius squares f32 coordinates before sqrt and scaling. Extreme finite coordinates may overflow while their direct transformed positions remain representable. Source-only boundary observation; no diagnostic or host exposure reproduced yet.'},
 {'title':'Weighted partition and Mosaic arithmetic may overflow before normalizing shares','source':'crates/cambium/scenes/scenomise/src/families/partition.rs:rings; families/mosaic.rs:shape','note':'Finite weight sums, area products and TAU*size intermediates can exceed f32. An invalid ring span may also affect sector point count. Do not run unbounded allocation; qualify checked arithmetic and output admission through projection/host callers first.'},
 {'title':'Duplicate scene entries for a facet swatch are silently ignored after the first','source':'crates/cambium/scenes/scenomise/src/facet.rs:compose_facet','note':'The input slice checks unknown names but scene_of uses find, so duplicate known names select the first scene. Normal facet-driven producers may already supply unique names. Trace caller invariants before calling this a false admission or dropped product update.'}
],'title')
inventory,coverage=load('source_inventory.json')['files'],load('read_coverage.json')
complete=[p for p,v in inventory.items() if p.startswith('crates/cambium/scenes/scenomise/') and coverage.get(p,{}).get('sha256')==v['sha256'] and coverage[p]['through']==v['chars']]
assert all(sha(ROOT/p)==inventory[p]['sha256'] for p in complete)
next_path=next(p for p in sorted(inventory) if p.startswith('crates/cambium/scenes/scenomise/') and p not in complete)
next_read={'path':next_path,'start':coverage.get(next_path,{}).get('through',0)}
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
notes['current_area']='Scenomise built-in catalog and all family solvers read; host/compiler and integration source remain'
notes['next_area']='Scenomise history, host dataset, projection, registry, template and remaining source'
notes['mode']='Read-only full-source audit continuation; preserve prior authorized repairs and concurrent WIP.'
notes['checkpoint']={'classification':'progress','goal_status':'active; full-source objective incomplete','coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),'new_receipts':['scenomise_review_receipt.json'],'next_read':next_read,'diagnostic_outcomes':'Six new findings Q074-Q079; eight native diagnostics passed as observation tests, no repair claim. Large expansions were not executed.','workspace':'Stable C:/t/cargo-targets/mere reused; own probe exe/PDB removed; no isolated target, Cargo home or worktree.'}
notes['audit_continuation']={'previous_turn':'progress: Scenograph full read, four bounded repairs and Q073 compatibility finding','classification_of_previous_turn':'progress','production_edits_this_turn':False,'coverage_checkpoint':stats,'next_read':next_read,'user_instruction':'Continue the full-source audit; preserve concurrent work; do not change production code.','coverage_note':'Sixteen additional source files fully read. No claim that all Scenomise or all Mere source is read.'}
assert len({v['id'] for v in notes['findings']})==len(notes['findings'])
assert len({v['id'] for v in authority['entries']})==len(authority['entries'])
save('review_notes.json',notes);save('authority_map.json',authority)

doc='design_docs/mere_docs/implementation_strategy/2026-10-07_scenograph_editor_plan.md'
addition='''

### Read-only solver audit continuation, 2026-10-16

The built-in catalog, document/facet composition, realizer and every family
solver are now fully read. Remaining Scenomise host/compiler/history/registry/
template/integration code is still under review; package compilation gives no
reading credit. Production source and prior repairs are unchanged this pass.

Six further findings are recorded (Q074–Q079). Timeline casts its f64 span and
numerator to f32 before division: a finite, round-trippable `[0, 1e100]` score
produces NaN, and its structurally admitted snapshot cannot reopen from JSON.
`[0, 1e-100]` collapses distinct endpoints; `[0, 100]` is the passing control.
This supplies an actual producer for Q069, rather than only injecting invalid
geometry directly at snapshot admission.

Every host-named L-system grammar realizes as Hilbert without an unavailable
result, although Sceno says its rules come from the host. This fallback is
intentional in the existing test; the authored picker remains limited to the
three built-ins. Explicit L-system and Penrose depths both admit 255 for one
item, bypassing their automatic depth caps. A symbol-count recurrence over the
reviewed Hilbert rules yields 3,665,038,759,251 symbols at depth 20. No expansion
string or hazardous solve was executed. Configurable work/output budgets and
bounded refusal need a broader compiler/host pass under the one-hour diversion
rule; accepting a u8 is not resource admission.

Radial checks negativity after rounding: `-1` takes the outer fallback, whereas
`-0.1` lands at the focus. A maximum u32 ring plus an undisclosed item overflows
the unchecked outer-ring increment and panics in the native overflow-checked
library. Release/wasm behavior was not executed. Sets keeps six declared source
memberships in its regions but silently places an impossible six-way member
inside only one region. The sampler relaxes semantic constraints without an
unmet-placement report; source membership itself is not changed.

The authority map distinguishes shared declarations from duplicated parsing:
editor/refusal keys and choice tables are shared, but OptionKind and catalog
Options implement value predicates separately. Their reviewed branches agree;
no parser divergence is alleged. Common scene construction/hold reporting is
real reuse, while usable-weight and rectangle-contour helpers remain repeated.
Stateless solvers are not necessarily one linear pass or bounded work.

Eight native public-API diagnostics pass as observation tests, including the
finite Timeline control. These results reproduce defects; they are not repairs
or product acceptance. The probe executable and PDB were removed. Exact source,
library and diagnostic hashes are in the
[solver audit receipt](../../../support/receipts/authority-success-audit/scenomise_review_receipt.json).
'''
destination=ROOT/doc
before=destination.read_bytes()
assert b'### Read-only solver audit continuation, 2026-10-16' not in before
destination.write_bytes(before+addition.encode())
diff=subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True,text=True)
(HERE/'scenomise_review_diff_check.txt').write_text(diff.stdout+diff.stderr,encoding='utf-8')
assert diff.returncode==0
evidence=['scenomise_solver_probe.rs','run_scenomise_solver_probe.py','scenomise_probe_library_build.jsonl','scenomise_solver_probe_build.log','scenomise_solver_probe_output.log','scenomise_solver_probe_receipt.json','lsystem_expansion_cost.json','scenomise_review_diff_check.txt']
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
 'kind':'Read-only built-in solver/catalog full-file review; broader Scenomise/Mere incomplete','previous_turn_classification':'progress','production_edits_this_pass':False,
 'new_findings':[v['id'] for v in findings],'authority_entries':[v['id'] for v in entries],'coverage':stats,'complete_scenomise_files':sorted(complete),'next_read':next_read,
 'source_sha256':{p:sha(ROOT/p) for p in complete},'preserved_production_and_test_hashes':{**old['preserved_production_and_test_hashes'],**old['source_sha256']},
 'diagnostic_tests':8,'diagnostic_exit':0,'expansion_executed':False,'generated_output_removed':probe['generated_output_removed'],'retained_isolation':[],
 'document':{'path':doc,'sha256':sha(destination),'preserved_prefix_bytes':len(before),'preserved_prefix_sha256':hashlib.sha256(before).hexdigest()},
 'diff_check':0,'evidence_sha256':{p:sha(HERE/p) for p in evidence},
 'limits':['No whole-source completion claim.','No production fixes this continuation.','No hazardous expansion, release overflow, wasm, headed UI, real network or user archive execution.','Existing Q073 client byte-identity test remains failing; this pass did not rerun the full package suites.']}
assert all(sha(ROOT/p)==h for p,h in receipt['preserved_production_and_test_hashes'].items())
save('scenomise_review_receipt.json',receipt)
print(json.dumps({'coverage':stats,'findings':len(notes['findings']),'authority_entries':len(authority['entries']),'scenomise_files_complete':len(complete),'next_read':next_read}))
