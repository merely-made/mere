"""Scenotime full source-review checkpoint and diagnostic limits."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib
import json
import re
import subprocess

HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8'))
def save(n,v): (HERE/n).write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8')
notes,authority=load('review_notes.json'),load('authority_map.json')
old=load('chirograph_review_receipt.json')
probe=load('scenotime_review_probe_receipt.json')
preserved=old['preserved_production_and_test_hashes']
assert all(sha(ROOT/p)==h for p,h in preserved.items())
assert probe['test_exit']==0 and probe['tests']==6 and probe['generated_native_output_removed']
assert all(sha(ROOT/p)==h for p,h in probe['source_sha256'].items())
assert load('pending_read.json')==[]
inventory,coverage=load('source_inventory.json')['files'],load('read_coverage.json')
paths=[p for p in inventory if p.startswith('crates/cambium/scenes/scenotime/')]
assert len(paths)==13
assert all(sha(ROOT/p)==inventory[p]['sha256']==coverage[p]['sha256'] and coverage[p]['through']==inventory[p]['chars'] for p in paths)
unitlog=(HERE/'scenotime_unit_integration_output.jsonl').read_text(encoding='utf-8-sig',errors='replace')
counts=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed;',unitlog)
assert counts==[('43','0'),('6','0'),('3','0'),('2','0')],counts
assert any(json.loads(s).get('reason')=='build-finished' and json.loads(s).get('success') for s in unitlog.splitlines() if s.startswith('{'))
findings=[{'id':'Q069','status':'public snapshot/diff/trace admission and non-reopenable output reproduced; not repaired',
 'title':'Structurally validated scene geometry can serialize successfully into bytes that cannot reopen',
 'source':'crates/cambium/scenes/scenotime/src/snapshot.rs:SceneSnapshot::validate',
 'related_sources':['crates/cambium/scenes/scenotime/src/diff.rs:SceneSnapshot::apply_diff','crates/cambium/scenes/scenotime/src/trace.rs:SceneTrace::from_steps','crates/cambium/scenes/scenotime/src/trace.rs:Serialize'],
 'evidence':'Snapshot validation checks slot/reference/fold/hold structure but no finite geometry. Public from_dense and validate accept NaN/Infinity in item position/scale, footprint width and scene bounds. SceneTrace::new accepts each; serde_json serialization succeeds, writing null for the nonfinite numeric value, but deserializing the emitted snapshot/trace fails. A finite trace control reopens identically. A nonfinite SetBounds diff also returns Applied and commits revision 2, and its accepted trace serializes into bytes that cannot reopen.',
 'scope':'Current compiled public Sceno/Scenotime APIs on generated typed data. No product solver, renderer, user archive or remote peer was executed. Malformed numeric values arise before JSON encoding; this is not evidence that JSON decoding accepts NaN syntax. The existing structural validation claims are narrower than full wire round-trip validity; the diagnostic identifies that missing boundary rather than an authorization failure.',
 'verification':'scenotime_review_receipt.json','diagnostic':'scenotime_review_probe_receipt.json',
 'repair_triage':{'disposition':'documented; active goal excludes production edits','reason':'Define and share finite-value admission for all serialized scene geometry, then apply it consistently at from_dense, diff commit, capture and trace boundaries. Preserve explicit zero/degenerate footprint semantics and separately decide permissible negative dimensions/scales. Test serialization/reopening as well as structural validity; do not silently normalize invalid geometry or change source coordinates.'}}]
entries=[
 {'id':'A068','domain':'Scene structure, numeric geometry and portable wire validity','claim':'Validated snapshots/diffs and chain-checked traces are portable scene records.',
  'implementations':['Snapshot validates structural references, world slot/parents, folds and held placements','Diff clones and validates before replacing the old snapshot','Trace construction reuses snapshot/diff validation','JSON serialization independently maps nonfinite f32 values to null'],
  'assessment':'Q069 separates real structural/transactional validation from missing numeric wire validity. Successful validation plus successful serialization does not establish reopenability.','state':'four geometry classes and committed nonfinite diff reproduced; finite control passes','verification':'scenotime_review_receipt.json'},
 {'id':'A069','domain':'Live replay tolerance, trace chain and host undo ownership','claim':'Scenotime owns scene-edit records, while the host owns position and undo/redo.',
  'implementations':['Live apply_diff intentionally accepts repeated/older revisions as no-ops','SceneTrace.advance explicitly requires epoch/base/strict forward progress','Trace reuses apply_diff after checking that stricter contract','History integration keeps immutable SceneTrace values in Backstep; no cursor inside SceneTrace'],
  'assessment':'The stronger trace preflight is an explicit, tested contract difference, not accidental duplicate authorization. Backstep owns editor history; trace owns replay data. The site oracle shares apply_diff, so it checks composition/fixture parity rather than independently proving the diff algorithm.','state':'full source and tests read; package gates passed','verification':'scenotime_review_receipt.json'},
 {'id':'A070','domain':'Coordinate-chain resolution and picking traversal','claim':'Sceno holds transforms while Scenotime supplies a shared picking floor.',
  'implementations':['Dense Scene.to_world composes a parent chain','Sparse SceneTables.space_to_world repeats that walk with tombstone handling','Picking calls sparse resolution per candidate item and inverse-transforms hit geometry','Client fold centroid reuses sparse helper; host receipt world_origin separately walks spaces'],
  'assessment':'Shared Transform2/Footprint math is real reuse. Coordinate traversal still has dense/sparse/receipt implementations; no valid-input divergence is established here. The stated linear-in-items pick cost omits ancestor depth, and snapshot validation also walks each space chain independently. This is a source complexity qualification, not a measured performance regression.','state':'owners and duplication mapped; runtime consumer scale remains ahead','verification':'scenotime_review_receipt.json'},
 {'id':'A071','domain':'Host timestamps, accumulated elapsed time and transient visual offset','claim':'The host owns clock progression; Scenotime owns pure transition sampling and a source-neutral return offset.',
  'implementations':['ProjectionClock accumulates nonnegative deltas and updates its baseline even after backwards samples','TransitionSchedule validates the spec and derives endpoints from a cloned applied diff','ReturnMotion owns local offset only, never scene/source coordinates','Pause accounting depends on continuing to observe host timestamps while paused, as documented'],
  'assessment':'Monotonic repeated-time control is stable. Public diagnostic samples [100,110,105,110] produce elapsed 15, charging previously seen host time again. Treat this as an out-of-order clock boundary to decide, not proof of drift under normal monotonic browser timestamps. ReturnMotion is an explicit local counterpart to physics, not another source-position authority.','state':'clock anomaly qualified by native diagnostic; normal frame host not executed','verification':'scenotime_review_receipt.json'}
]
def upsert(items,added,key):
    for x in added:
        at=next((i for i,v in enumerate(items) if v.get(key)==x[key]),None)
        if at is None: items.append(x)
        else: items[at]=x
upsert(notes['findings'],findings,'id')
upsert(authority['entries'],entries,'id')
upsert(notes['candidates'],[
 {'title':'ProjectionClock rewinds its delta baseline on an out-of-order timestamp','source':'crates/cambium/scenes/scenotime/src/clock.rs:ProjectionClock::observe','note':'Public diagnostic [100,110,105,110] gives elapsed 15 rather than 10; the backwards call itself adds nothing but resets the previous timestamp. Normal browser timestamps are monotonic. Explicitly choose ignore-out-of-order versus reset-clock semantics at a consumer boundary before changing policy; no normal frame-host failure established.'},
 {'title':'Transition scheduling only inspects direct item operations','source':'crates/cambium/scenes/scenotime/src/transition.rs:affected_instance','note':'UpdateSpace, folds and other operations are not collected as affected item instances; interpolation uses item-local transforms, not resolved world endpoints. The target snapshot may intentionally snap these changes. Inspect transition consumers/spec scope before calling all space/fold changes a missing animation bug.'},
 {'title':'Held-placement metadata has no SceneOp update path','source':'crates/cambium/scenes/scenotime/src/diff.rs:SceneOp','note':'honored_holds/unmet_holds travel in snapshots, but no diff variant changes them. Moving/tombstoning an honored instance can fail validation because its old honored record remains. Full snapshots can replace this state. Trace solver/projection update composition before claiming a broken authorized unpin/move path or inventing a new ownership policy.'},
 {'title':'Honored pin position comparison uses the item local translation','source':'crates/cambium/scenes/scenotime/src/snapshot.rs:honored_holds validation','note':'Validation compares item.transform.translate to HeldPlacement.at without resolving parent transforms. A parent can move while this equality remains true. Holds may intentionally be frame-relative, so the coordinate convention needs a consumer/ruling trace before alleging a false world-position claim.'},
 {'title':'ReturnMotion finite settings are not bounded for numerical stability','source':'crates/cambium/scenes/scenotime/src/return_motion.rs:ReturnMotion::tick','note':'Sanitization rejects nonfinite inputs but leaves any finite strength/offset admissible. Explicit Euler-like updates may overflow or become unstable for extreme finite settings. Existing tests cover defaults, finite step variation, zero pull and nonfinite inputs. No extreme-setting or live-host failure executed this pass.'},
 {'title':'Deserialized transition schedules bypass TransitionSpec validation','source':'crates/cambium/scenes/scenotime/src/transition.rs:TransitionSchedule','note':'TransitionSchedule and ScheduledItem derive Deserialize and expose fields; sample_at does not validate duration/windows/endpoints. from_diff validates its spec, but arbitrary wire schedules need a composing admission rule. Locate actual schedule consumers before a persisted/untrusted schedule claim.'}
],'title')
for x in notes['candidates']:
    if x.get('title')=='Canonical routed LinkKey ordering is only applied by its constructor' and 'Scenotime full review and public diagnostic' not in x['note']:
        x['note']+=' Scenotime full review and public diagnostic now confirm a reversed deserialized LinkKey passes snapshot validation and produces a different stable_id. No actual importing/grouping consumer failure has been executed.'
upsert(notes['source_qualifications'],[
 {'area':'Scenotime complete source','note':'All 13 files read at current hashes, including manifests and integration tests. Live diff rollback, append-only slots/tombstones and strict trace chaining are real. Picking checks own visibility plus fold hiding, explicit order/layer and hit overrides through transforms. Snapshot validation centrally delegates fold uniqueness/disjointness to Sceno.'},
 {'area':'Scenotime package execution','note':'Current native gate passed 43 library tests, six fold integration tests, three history tests and two site-projection tests, 54 total. Six standalone diagnostics passed, including finite/monotonic controls, a nonfinite round-trip defect and two qualified boundary observations. No doctest, wasm, browser frame loop, renderer, physical accessibility or user persistence gate.'},
 {'area':'Snapshot boundary scope','note':'Derived SceneSnapshot deserialization does not itself validate tables; explicit admission functions do. active_items_in_order indexes item_order directly and expects admitted shape. Do not turn a deliberately raw DTO constructor/serde path into a separate source-truth owner without tracing its caller. Q069 remains a defect after the explicit structural admission functions succeed.'}
],'area')
if 'crates/cambium/scenes/scenotime' not in notes['completed_areas']: notes['completed_areas'].append('crates/cambium/scenes/scenotime')
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
next_path=next(p for p in sorted(inventory) if p.startswith('crates/cambium/scenes/scenograph/') and (coverage.get(p,{}).get('sha256')!=inventory[p]['sha256'] or coverage.get(p,{}).get('through',0)<inventory[p]['chars']))
next_read={'path':next_path,'start':coverage.get(next_path,{}).get('through',0)}
notes['current_area']='Scenotime complete; shared scene authoring/binding/presentation ownership next'
notes['next_area']='crates/cambium/scenes/scenograph, then Scenomise and remaining Mere source'
notes['checkpoint']={'classification':'progress','goal_status':'active; full-source objective incomplete','coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),
 'new_receipts':['scenotime_review_receipt.json'],'diagnostic_outcomes':'All 54 package tests and six standalone public-API diagnostics passed. One new nonfinite round-trip finding; clock/link-key observations explicitly qualified.',
 'next_read':next_read,'workspace':'Reused C:/t/cargo-targets/mere. Standalone probe exe/PDB removed; no isolated target, Cargo home or worktree.'}
notes['audit_continuation']={'previous_turn':'progress: Chirograph/Sceno complete, Q067-Q068, 89 package tests and five diagnostics','classification_of_previous_turn':'progress','production_edits_this_turn':False,
 'completed_areas_this_pass':['crates/cambium/scenes/scenotime/'],'coverage_checkpoint':stats,'next_read':next_read,'user_instruction':'Continue full source audit; preserve concurrent work and do not change production code.',
 'coverage_note':'Scenotime source complete; whole Mere goal remains incomplete. Diagnostic success establishes a defect/control boundary, not a repair.'}
assert len({x['id'] for x in notes['findings']})==len(notes['findings'])
assert len({x['id'] for x in authority['entries']})==len(authority['entries'])
save('review_notes.json',notes);save('authority_map.json',authority)
diff=subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True,text=True)
(HERE/'scenotime_review_diff_check.txt').write_text(diff.stdout+diff.stderr,encoding='utf-8')
doc='design_docs/scenograph_docs/technical_architecture/2026-07-22_scenotime_epoch_diff_note.md'
evidence=['scenotime_unit_integration_output.jsonl','scenotime_probe_library_build_output.jsonl','scenotime_review_probe.rs','run_scenotime_review_probe.py','scenotime_review_probe_build_output.txt','scenotime_review_probe_output.txt','scenotime_review_probe_receipt.json']
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'previous_turn_classification':'progress',
 'kind':'Complete Scenotime source review, package tests and bounded public-API diagnostics; no repairs','findings':[x['id'] for x in findings],'authority_entries':[x['id'] for x in entries],
 'complete_areas':['crates/cambium/scenes/scenotime/'],'source_files':len(paths),'source_chars':sum(inventory[p]['chars'] for p in paths),'coverage':stats,'next_read':next_read,'production_edits_this_pass':False,
 'preserved_production_and_test_hashes':preserved,'source_sha256':{p:sha(ROOT/p) for p in paths},'governing_document_sha256':{doc:sha(ROOT/doc)},
 'package_tests':{'exit':0,'passed':54,'failed':0,'library':43,'integration':{'fold':6,'history':3,'site_projection':2},'command':'cargo test -p scenotime --lib --tests --locked --offline -j 1 --target-dir C:/t/cargo-targets/mere --config build.build-dir=C:/t/cargo-targets/mere/build --message-format=json'},
 'diagnostic_tests':{'passed':6,'failed':0},'coverage_limits':['No whole-source completion claim.','No renderer, browser, real clock callback, network, user archive or wasm gate.','Link-key and backwards-clock observations have qualified consumer/policy scope.'],
 'generated_output_removed':probe['generated_native_output_removed'],'retained_isolation':[],'diff_check':{'exit':diff.returncode,'log':'scenotime_review_diff_check.txt'},'evidence_sha256':{p:sha(HERE/p) for p in evidence}}
save('scenotime_review_receipt.json',receipt)
print(json.dumps({'coverage':stats,'findings':len(notes['findings']),'authorities':len(authority['entries']),'next_read':next_read,'preserved_hashes':len(preserved),'diff_check':diff.returncode}))
