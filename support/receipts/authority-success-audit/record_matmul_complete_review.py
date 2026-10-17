"""Source reading checkpoint: all retained matmul files, including optional test trees."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,importlib.util,json,subprocess,sys
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
NAME='matmul_complete_review_receipt.json';OLD='matmul_key_test_review_receipt.json'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8-sig'))
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT,text=True).strip()
assert not (HERE/NAME).exists();old=load(OLD)
assert git('rev-parse','HEAD')==old['basis_commit'] and git('rev-parse','origin/main')==old['upstream_commit']
guards={**old['preserved_production_and_test_hashes'],**old['source_sha256']}
assert all(sha(ROOT/p)==h for p,h in {**guards,**old['auxiliary_read_sha256'],**old['runtime_input_sha256']}.items())
assert all(sha(Path(p))==h for p,h in old['external_context_sha256'].items())
assert all(sha(HERE/p)==h for p,h in old['evidence_sha256'].items())
doc=ROOT/old['document']['path'];prefix=doc.read_bytes();assert sha(doc)==old['document']['sha256']
assert all(sha(ROOT/p)==h for p,h in old['shared_working_file_sha256'].items() if p!=old['document']['path'])
index=Path(git('rev-parse','--git-path','index'));index=index if index.is_absolute() else ROOT/index
assert sha(index)==old['git_index_sha256']
i=load('source_inventory.json')['files'];c=load('read_coverage.json');assert load('pending_read.json')==[]
M='support/patches/cubek-matmul/';T=M+'tests/';F=T+'multi_level/full/';S=M+'src/'
paths=[p for p in i if p.startswith(M) and c.get(p,{}).get('through',0)>0 and p not in guards]
assert len(paths)==33 and sum(c[p]['through'] for p in paths)==133335
matmul=[p for p in i if p.startswith(M)]
assert len(matmul)==317 and all(c[p]['sha256']==i[p]['sha256']==sha(ROOT/p) and c[p]['through']==i[p]['chars'] for p in matmul)
spec=importlib.util.spec_from_file_location('audit_source_membership',HERE/'read_coverage.py');module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
explicit={'design_docs/mere_docs/receipts/natal-cap-20261017/mere-intake-narrow-final.diff','ports/graphshell/web/extension/manifest.chromium.json','ports/graphshell/web/extension/manifest.firefox.json'}
live=set()
for p in subprocess.check_output(['git','ls-files','--cached','--others','--exclude-standard','-z'],cwd=ROOT).decode().split('\0'):
 if not p or p.startswith('support/receipts/authority-success-audit/'):continue
 f=ROOT/p
 if f.is_file() and (module.is_source(f) or p in explicit):live.add(p)
assert live==set(i) and all(sha(ROOT/p)==v['sha256'] for p,v in i.items())
notes=load('review_notes.json');authority=load('authority_map.json')
assert notes['findings'][-1]['id']=='Q457' and authority['entries'][-1]['id']=='A1483'
rows=[
 ('Full test migration omissions recur across routine families','Full unit imports test_matmul_test_strategy from missing harness export; TMA MMA arm uses MultiLevel without local import.',[F+'unit/mod.rs',F+'tma/mod.rs',T+'harness/mod.rs',F+'plane_accelerated/mod.rs'],'Extends Q457 static source mismatch. Current helper exists under multi_level::extended. Same obsolete common three-argument builder expands across every full include family. Full-feature Cargo compile remains unrun; no exact compiler error count or default-suite failure claimed.'),
 ('Full cartesian test shape has repeated precision/stage scaffolding','Plane-accelerated, plane-vecmat, TMA and unit trees retain byte-identical precision and some stage files.',[F+'plane_accelerated/precision.rs',F+'plane_vecmat/precision.rs',F+'tma/precision.rs',F+'unit/precision.rs',F+'unit/tiling_scheme/stage.rs'],'Shared common include files reduce some repetition, but per-family precision/stage modules still copy policy. OS-selected tile assumptions and all combinations may be deliberately invalid; enforce policy decides whether refused combos count as pass. Large generated test counts do not qualify executing all families.'),
 ('Tiled CMMA tests pin explicit refusal versus availability contracts','Tests distinguish oversized TMA box, invalid input register type, stored-K disagreement and unsupported hardware.',[T+'tiled/cmma.rs',S+'tiled/cmma/base.rs',S+'tiled/cmma/launch.rs'],'Some tests exercise pure blueprint validation, others use capture + policy. Stored tile across M, grid order and finest read use real relayout and CPU reference when executed. A test claiming never refuses tiled weight under TMA permits Unavailable, which is correctly narrower than execution acceptance. Comments saying forced path only validates are incomplete because capability gates also run after pure validation.'),
 ('CPU layout laboratory is a test implementation, not production layout authority','InnerLayout builds physical dims/strides and copy_logical scatters/gathers test operands; production reads binding metadata.',[T+'tiled/cpu_gemm/inner_layout.rs',T+'tiled/cpu_gemm/layouts.rs',S+'tiled/matrix.rs'],'Explicit synthetic layout enum is justified by independent oracle intent. Its vector tensor_arg arithmetic and local nested tile factors remain copied; current calls use scalar width. Logical copy uses one cube/one unit, resolving suspicion of duplicate concurrent writes. Tests poison output before production launch.'),
 ('CPU layout positive overhang case appears stale against strict stored-tile admission','tiled_storage_partitioner_overhang feeds4x4 stored tiles to a forced3x3x3 instruction and2x2 plane grid.',[T+'tiled/cpu_gemm/layouts.rs',S+'tiled/cpu_gemm/launch.rs',S+'tiled/matrix.rs'],'Current launch cut_by checks input storage tile matches a kernel level before dispatch; the3/6/etc geometry appears not to contain4x4, while positive test calls run(...).unwrap. Named source contradiction requiring actual shared Partitioning/CPU admission qualification, not promoted to a confirmed backend failure. Nested-below-leaf test explicitly expects refusal, and recursive output is a separate supported contract.'),
 ('CPU GEMM test descriptions lag the current divisor heuristic','many_tiles_inferred_size and indivisible_inferred comments say inference picks L1-sized, non-divisor overhang blocks.',[T+'tiled/cpu_gemm/mod.rs',S+'tiled/cpu_gemm/base.rs'],'Current heuristic sizes register budget and snaps each tile to a divisor; primes likely collapse rather than exercise claimed overhang. Forced indivisible tests still cover edge masks. Broadcast tests called unbatched use shape![1], retaining equal operand rank, so they do not close Q455 asymmetric-rank key coverage.'),
 ('Accumulator precision test supplies a discriminating numeric oracle','CPU test uses1.0 plus8191 half-ULP increments to distinguish F32 accumulate from F16 output.',[T+'tiled/cpu_gemm/mod.rs'],'Explicit <0.01 assertion rejects NaN and catches roughly5 versus1 behavior when executed. Test source is read, not rerun; skip_unless_cpu keeps a green GPU run from proving CPU behavior. Same helper default policy can still skip some other config rejections.'),
 ('Direct QuantGemv tests use independent packed inputs and NaN-rejecting comparisons','Tests construct signed q8/q4 packed words, F16 activation/block scales and optional F32 global scale, comparing host sum.',[T+'tiled/quant_gemv.rs',S+'tiled/quant_gemv/launch.rs'],'Explicit <=tolerance asserts reject NaN, unlike Q453 benchmark refusal comparison. Covers one/multiple activation rows and two scale levels. Output begins zero, so tests alone do not prove each-call reset over arbitrary previous memory, although kernel source explicitly zeroes. No fresh actual backend execution claimed.'),
 ('Storage round-trip tests use a separate physical-location oracle','tile/untile tests check every cell, metadata shape/strides, nested/grid orders and packed fields.',[T+'tiled/storage.rs',S+'tiled/storage.rs'],'Plain-value round trips and packed-word field positions are separate assertions; packed untile is not claimed. dtypeu32 is correctly supplied for packed relayout, narrowing earlier malformed dtype concern to direct API misuse/domain. Invalid tile divisibility/already-tiled/plain-untiling and word-finest cases explicitly reject; no malformed axes/rank test here.'),
 ('Remaining manual selector probes are timing studies with omitted failures','selector_probe median_ms turns any bench Err intoNone and prints declined; all tests ignored by default.',[T+'tiled/selector_probe.rs',T+'tiled/bench_catalog.rs',T+'tiled/mod.rs',T+'tiled/stride_zero.rs'],'No correctness proof in the selector timing probe; forced-point correctness is separate ignored test on a smaller shape. CPU stride-zero uses Q456 shared helper. Duplicate cfg(feature=extended) on stride_zero is inert cruft. Read coverage includes all optional/ignored source without substituting it for execution.'),
 ('Entire retained matmul source has now been read','317 inventoried source/manifests/tests/patch files totaling1590560 characters complete.',matmul,'This is source review completion for retained cubek-matmul only. Selected root consumers use this0.3 crate. Full repository remains incomplete; remaining reduction patch is root-unselected but used by named Distillery probe workspaces. No old source was dropped to simplify scope, no production edits or full-suite/GPU acceptance inferred.')]
ids=[]
for number,(domain,claim,implementations,assessment) in enumerate(rows,1484):
 assert all((ROOT/p).exists() for p in implementations)
 ident=f'A{number}';ids.append(ident);authority['entries'].append({'id':ident,'domain':domain,'claim':claim,'implementations':implementations,'assessment':assessment,'state':'all retained matmul source read, execution scope remains separately qualified','verification':NAME})
notes['review_followups'].extend([
 {'area':'Matmul source review closure','detail':'All317 files read. Q457 extends to unit harness import and TMA MultiLevel import. CPU positive tiled-storage/3x3-leaf case appears stale versus cut_by; qualify actual Partitioning before calling it a backend failure. Retain open findings without converting source-reading completion into GPU/full-suite proof.','verification':NAME},
 {'area':'Remaining source scope','detail':'89 retained cubek-reduce files and35 other support/workflow/tool files remain. Root reduce uses registry0.3, but independent Distillery probes select retained patch; audit it rather than dropping unselected source.','verification':NAME}])
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==3614 and stats['read_chars']==38923017 and not stats['changed_since_inventory'] and stats['pending_chunks']==0
notes['completed_source_areas'].append({'prefix':'support/patches/cubek-matmul/','files':317,'chars':1590560,'files_newly_read':33,'chars_newly_read':133335,'verification':NAME,'basis_commit':old['basis_commit'],'qualification':'Entire retained matmul source complete, including manifests, production, evaluation and all optional/ignored tests. This checkpoint is source-only; no new actual backend/full feature/test suite run.'})
next_read={'path':'support/patches/cubek-reduce/Cargo.toml','start':0,'reason':'Read all retained reduction source, then remaining35 support/workflow/tool source and reconcile full objective/open findings.'}
pending='No audit-owned process running.3614/3738 source files complete;124 remain. All317 matmul files1590560chars complete. No new execution receipt in this source-only checkpoint. All3738 live source membership/hashes unchanged. Retained reduce89 and other35 source files pending. No production edit or isolated output.'
notes['current_area']='Retained matmul source entirely read';notes['next_area']=next_read['reason']
notes['checkpoint'].update({'classification':'progress','classification_of_previous_turn':'progress','coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),'next_read':next_read,'pending_verification':pending,'audit_authored_production_edits_this_turn':False})
notes['checkpoint']['new_receipts'].append(NAME)
addition=f'''

### Entire retained matmul source read, 2026-10-17

Read the remaining33 matmul test files,133,335 characters. All317 retained matmul source/manifests/tests are now fully read,1,590,560 characters. Overall3,614/3,738 files and38,923,017/39,556,704 characters are read;124 files remain. All3,738 live working-source membership and hashes remain unchanged.

{ids[0]}-{ids[-1]} record the remaining test ownership/evidence boundaries. Q457's static migration mismatch also appears in full unit harness imports and the full TMA MMA arm's missing MultiLevel import. Byte-identical per-family precision/stage scaffolding still surrounds shared includes. The CPU layout laboratory has a named possible stale positive case:4x4 stored inputs with a3x3 leaf conflict with current cut_by admission; actual shared Partitioning/CPU qualification is still required before reporting an executed failure. Inferred-heuristic test comments also lag current register-budget/divisor policy, so their names do not prove the claimed overhang coverage.

Newer direct QuantGemv and accumulator-precision tests use <=/< assertions that reject NaN, unlike Q453 benchmark gates. Storage tests have independent every-cell and packed-field oracles. Their source is read, not claimed freshly run. TMA refusal-versus-unavailability and output-reset contracts remain scoped; ignored timing studies and conditional CPU tests do not constitute cross-backend execution receipts.

This checkpoint adds no product edit or new execution claim. Report prefix, source/shared guards and index remain preserved, evidence below100MB, no live process or retained isolated build/home/worktree. [Checkpoint](../../../support/receipts/authority-success-audit/{NAME}). Continue retained cubek-reduce Cargo.toml, then its full89-file tree and the remaining35 support/workflow/tool files. Full-source objective remains active and incomplete.
'''
doc.write_bytes(prefix+addition.encode())
for name,value in [('review_notes.json',notes),('authority_map.json',authority)]: (HERE/name).write_text(json.dumps(value,indent=2)+'\n',encoding='utf-8')
assert subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True).returncode==0
budget=sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file());assert budget<100_000_000
record={**old,'date_utc':datetime.now(timezone.utc).isoformat(),'prior_receipt':{'path':OLD,'sha256':sha(HERE/OLD)},'coverage':stats,'matmul_coverage':{'complete_files':317,'total_files':317,'read_chars':1590560,'total_chars':1590560},'source_sha256':{p:sha(ROOT/p) for p in paths},'preserved_production_and_test_hashes':guards,'read_through':{p:c[p]['through'] for p in paths},'new_findings':[],'authority_entries':ids,'next_read':next_read,'pending_verification':pending,
 'document':{'path':old['document']['path'],'sha256':sha(doc),'preserved_prefix_bytes':len(prefix),'preserved_prefix_sha256':hashlib.sha256(prefix).hexdigest()},'evidence_sha256':{'record_matmul_complete_review.py':sha(HERE/'record_matmul_complete_review.py')},'receipt_budget':{'cap_bytes':100_000_000,'retained_bytes_before_receipt':budget},'qualification':'Progress only.33 newly complete test files133335chars; all317 retained matmul source1590560chars complete. Source-only checkpoint, no new GPU/CPU algorithm/full Cargo receipt. All3738 live source membership/hashes unchanged. Whole-repository source review still incomplete.'}
with (HERE/NAME).open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
assert doc.read_bytes().startswith(prefix) and sha(index)==old['git_index_sha256']
assert all(sha(ROOT/p)==h for p,h in {**guards,**record['source_sha256'],**old['auxiliary_read_sha256'],**old['runtime_input_sha256']}.items())
print(json.dumps({'receipt':NAME,'coverage':stats,'authority_entries':ids,'budget_bytes':budget,'next_read':next_read},indent=2))
