"""Checkpoint studies, Dramatis qualifiers and dated updater comparison source."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,importlib.util,json,subprocess,sys
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];NAME='support_qualifier_review_receipt.json'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8-sig'))
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT,text=True).strip()
assert not (HERE/NAME).exists()
old=load('scripts_review_receipt.json');head=git('rev-parse','HEAD');upstream=git('rev-parse','origin/main')
assert head==old['basis_commit'] and upstream==old['upstream_commit']
guards={**old['preserved_production_and_test_hashes'],**old['source_sha256']};aux=dict(old['auxiliary_read_sha256']);runtime=old['runtime_input_sha256'];shared=old['shared_working_file_sha256'];external=dict(old['external_context_sha256'])
assert all(sha(ROOT/p)==h for p,h in {**guards,**aux,**runtime}.items())
assert all(sha(Path(p))==h for p,h in external.items())
assert all(sha(HERE/p)==h for p,h in old['evidence_sha256'].items())
doc=ROOT/old['document']['path'];prefix=doc.read_bytes();assert sha(doc)==old['document']['sha256']
assert all(sha(ROOT/p)==h for p,h in shared.items() if p!=old['document']['path'])
index=Path(git('rev-parse','--git-path','index'))
if not index.is_absolute():index=ROOT/index
assert sha(index)==old['git_index_sha256']
i=load('source_inventory.json')['files'];c=load('read_coverage.json')
paths=[p for p in i if p.startswith(('support/browser-history-intake/','support/design-studies/','support/dramatis','support/luggage-comparison/'))]
assert len(paths)==13 and sum(i[p]['chars'] for p in paths)==133925
assert all(sha(ROOT/p)==i[p]['sha256']==c[p]['sha256'] and c[p]['through']==i[p]['chars'] for p in paths)
assert load('pending_read.json')==[];sources={p:sha(ROOT/p) for p in paths}
probe=load('support_driver_probe_receipt.json');assert probe['exit']==0 and probe['passed']==4
assert all(sha(ROOT/p)==h for p,h in probe['source_sha256'].items())
assert all(sha(HERE/p)==h for p,h in probe['evidence_sha256'].items())
# Reconcile the entire live source inventory, including explicit JSON/diff source
# inclusions. This does not award reading credit to any unread source.
spec=importlib.util.spec_from_file_location('audit_source_membership',HERE/'read_coverage.py');module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
explicit={'design_docs/mere_docs/receipts/natal-cap-20261017/mere-intake-narrow-final.diff','ports/graphshell/web/extension/manifest.chromium.json','ports/graphshell/web/extension/manifest.firefox.json'}
live=set()
for p in subprocess.check_output(['git','ls-files','--cached','--others','--exclude-standard','-z'],cwd=ROOT).decode().split('\0'):
 if not p or p.startswith('support/receipts/authority-success-audit/'):continue
 f=ROOT/p
 if f.is_file() and (module.is_source(f) or p in explicit):live.add(p)
assert live==set(i)
assert all(sha(ROOT/p)==v['sha256'] for p,v in i.items())
notes=load('review_notes.json');authority=load('authority_map.json')
assert notes['findings'][-1]['id']=='Q421' and authority['entries'][-1]['id']=='A1273'
new=[
 ('Q422','Dramatis package qualifier accepts a successful process that executes zero named tamper tests',
  'support/dramatis_package_receipts.py:31-38,103-106',
  'The signed-feed command selects --ignored --exact real_signed_package_and_tamper_controls, but run checks only process exit0. It never requires the named test or one passing test in the log. Exact AST-extracted run evaluated with a subprocess double emitting running0tests/0passed/1filteredout returns normally and records exit0; a nonzero-exit control raises. The full main later declares signed manifest/artifact/publisher/staging tamper gates in its receipt. A renamed/filtered/missing ignored test can therefore remove those checks while preserving the process success path.',
  'Require the exact intended test to execute and pass, ideally through structured test result or an explicit named receipt. Bind that result to the compiled binary/source instead of treating cargo exit as tamper-test completion. Keep package signing/extraction success separate from test coverage.',
  'One exact Python helper case plus a refusal control with deterministic subprocess/log double. No actual zero-test Cargo invocation, installer, signer, complete package pipeline or generated published receipt was run. This is the demonstrated qualifier admission weakness, not a claim current retained packages executed zero tests.'),
 ('Q423','Dramatis consent negative control can overwrite a concurrent production edit while restoring source',
  'support/dramatis_review_receipts.py:55-82',
  'consent_control reads production keeper/review.rs, writes an environment-gated denial bypass into the same primary source path, executes builds/tests, then unconditionally writes the saved bytes in finally. It does not require a clean isolated checkout or compare current bytes with the mutant before restore. Exact AST-extracted function with an entirely in-memory source double injects a concurrent owner edit during the negative-test call; finally overwrites it, and the byte-exact restoration assertion still passes. The no-concurrent-edit control restores as intended. Shared target builds can also retain mutant artifacts until explicit restored builds succeed.',
  'Run this mutation control in an isolated permitted checkout only when needed for a real source collision, or supply a test-local substitution that never rewrites shipping source. If restoring a mutable file, compare ownership/current bytes before overwriting and preserve conflicting edits. A restored source hash alone cannot prove concurrent work was preserved or every shared binary was rebuilt.',
  'Two exact-function cases with in-memory source and build/test doubles only. No actual product source, binary, custody bypass, native dialog, real vault or concurrent user work changed. The risk is directly present in the full driver source; the driver itself was not executed.')]
for ident,title,source,evidence,repair,scope in new:
 notes['findings'].append({'id':ident,'title':title,'source':source,'evidence':evidence,'status':'reproduced exact AST-extracted qualifier behavior with explicit doubles; documented, not repaired','scope':scope,'repair_triage':{'disposition':'documented in read-only full-source audit','reason':repair},'verification':NAME,'diagnostics':['support_driver_probe_receipt.json']})
rows=[
 ('Browser presentation study is an explicit second model with a parity seam','HTML implements presentation-rule validation/resolution locally; checked fixture/Rust comparison exists separately.',
  ['support/design-studies/scene-rules.html','scripts/mere_scene_study_check.mjs','scripts/mere_scene_study_pack.py'],
  'Full84,934-character HTML read includes embedded inert video and fixture payloads, JS validator/evaluator/winner traces and UI. It keeps rule bounds, priority/later-tie resolution, hysteresis, fixed occurrence IDs and multiple accesses to one resource. This duplicates typed Scenograph semantics as a browser study, not a production shared authority. Checker requires parity for supplied contexts but does not bind supplied Rust executable source. New evaluator cases can drift unless the comparison grows.'),
 ('Study force solver and resource/provenance projection are intentionally local','Browser uses deterministic bounded360-step study solver and fixed nodes/relations/family parameters.',
  ['support/design-studies/scene-rules.html'],
  'The solver explicitly says not a production dynamics engine. UI differentiates occurrence/resource, position/foreground pins, selection, lens, kept/dismissed nodes and representation. Shared-resource relation is a fixed authored fixture whose explanation matches the two shown resource strings, not a generic runtime derivation. Do not label this solver a complete shared graph runtime or an obsolete duplicate without consumer evidence.'),
 ('Study persistence is best-effort widget state rather than durable product authority','restore sanitizes known IDs/themes/positions/rules; save is optional, size-gated and suppresses rejection.',
  ['support/design-studies/scene-rules.html'],
  'Unknown/incompatible rule state falls back to defaults. The rendered edit takes effect even if widget state cannot be saved; there is no explicit saved-success claim in the UI. save refuses oversized state silently and catches promise rejection. Force geometry signature excludes viewport and measured-size changes, so ResizeObserver can leave positions unchanged after resize. These are study/lifetime followups, not proof product durable storage failed.'),
 ('Local source overlay and backdrop conversion are mutating utilities','One temporary Cargo config patches a coherent portable-scene cohort; PNG helper converts retained RGBA.',
  ['support/design-studies/backdrops/with_local_scene_source.py','support/design-studies/backdrops/render_cpu_pngs.py','support/browser-history-intake/.gitattributes'],
  'Overlay explicitly warns Cargo may update the consumer lock and exits with child status. It checks member manifests but does not prove the URL patch was selected. PNG helper checks byte extent but overwrites named output and does not bind receipt digest or reject zero extents. Neither executed. Intake attributes only preserve evidence byte/whitespace handling; they are not an acquisition or schema implementation.'),
 ('Dramatis package qualification has real controls but incomplete test/run ownership','Driver uses synthetic signer keys, validates extracted contents/native client and expects feed tamper source controls.',
  ['support/dramatis_package_receipts.py'],
  'Q422 isolates empty-test admission. Private signer output is suppressed, child-only signing environment is used, forbidden installer content is checked and tamper inputs must restore hashes. Keys/extraction cleanup checks parent/marker. It still reuses write-mode receipt paths, unbounded subprocesses, hardcoded toolchain and legacy target/debug assumptions. Windows debug receipt explicitly leaves release/installed trust/other hosts pending. Do not execute unchanged over retained evidence.'),
 ('Native consent witness is scoped to a synthetic process while the mutation control is not isolated','Native driver enumerates hostPID/title/button IDs; review driver mutates production consent source.',
  ['support/dramatis_native_consent.py','support/dramatis_review_receipts.py'],
  'Native driver checks contents/default-No and captures dialog before BM_CLICK on observed child; that is programmatic native control activation, not physical input or arbitrary product approval. Q423 concerns separate review-driver source custody, not native window matching. Both use fixed endpoints/output names and old direct binary paths. Full review hashes sources/executables after running and says captures require separate visual review. No live driver executed.'),
 ('Updater comparison deliberately preserves dated gaps rather than claiming current safe installation','Standalone Luggage/Velopack/TUF research harness records success and known failures as observations.',
  ['support/luggage-comparison/Cargo.toml','support/luggage-comparison/src/main.rs','support/luggage-comparison/src/tuf.rs','support/luggage-comparison/verify.py'],
  'verify explicitly requires known gaps to remain gaps for a dated comparison and says these are not future updater safety requirements. It checks public fixture SHA256/Git blobs, lock, Luggage source and evidence hashes. Current Luggage source changes should invalidate old source receipts; do not reinterpret this verifier as a green current updater acceptance gate. Rust harness disables auto-apply at startup; explicit apply is used only by disposable test driver. No comparison rebuilt or rerun.'),
 ('Windows replacement probe owns disposable fixture application state','Velopack driver stages/tampers fixture package and injects a directory handle to block rename.',
  ['support/luggage-comparison/windows.py'],
  'It checks marker-named work root and records failure even in finally, bounds subprocess/replace waits and terminates its own helper. It creates16MiB random payload, reuses fixed logs/build inputs and assumes target/debug layout. It never installs a product, but explicit apply does replace disposable fixture current state. Its local success merely produces observed JSON; dated verify checks the expected gaps separately. No such mutation performed by audit.')]
ids=[]
for number,(domain,claim,implementations,assessment) in enumerate(rows,1274):
 ident=f'A{number}';ids.append(ident);authority['entries'].append({'id':ident,'domain':domain,'claim':claim,'implementations':implementations,'assessment':assessment,'state':'complete support study/qualifier/comparison source read; implementation and evidence scopes distinct','verification':NAME})
notes['review_followups'].extend([
 {'area':'Qualifier source and test custody','detail':'Q422 zero-test admission and Q423 in-place production consent mutation require bounded runner fixes. Fixed output/endpoint paths, post-run source sampling, legacy output locations and arbitrary test-source mutation must not become present acceptance claims merely because historical receipts exist. No actual driver ran here.','verification':NAME},
 {'area':'Browser study parity and lifetime','detail':'Presentation JS is an explicit study duplicate with Rust parity checker, not shared production authority. Force solver parameters/provenance are fixed study fixtures; persistence is optional/size-limited and errors suppressed. Trace current consumers and viewport/signature behavior before promoting study semantics into product claims.','verification':NAME}])
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==3016 and stats['read_chars']==35240370 and not stats['changed_since_inventory'] and stats['pending_chunks']==0
notes['completed_source_areas'].append({'prefix':'Support intake attributes, scene studies/backdrops, Dramatis drivers and Luggage comparison','files':13,'chars':133925,'verification':NAME,'basis_commit':head,'qualification':'Full source read including embedded data.4 exact AST-function cases with explicit in-memory/process doubles; no live driver/native/signing/mutation/comparison run. Full3738 source inventory membership and SHA256 reconfirmed without awarding unread credit.'})
next_read={'path':'support/patches/burn-cubecl/Cargo.toml','start':0,'reason':'Read all remaining supplier patches/source, including duplicated build manifests and retained fork deltas; preserve denominator and authority ledger.'}
pending='No audit-owned process running.4 qualifier CPU cases pass with explicit doubles.3016/3738 source files read;722 remain. All3738 live source hashes and membership unchanged. No production edit, generated binary, isolated target/home/worktree or live native/system action.'
notes['current_area']='Support studies and qualifiers complete; supplier patches next';notes['next_area']=next_read['reason']
notes['checkpoint'].update({'classification':'progress','classification_of_previous_turn':'progress','coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),'next_read':next_read,'pending_verification':pending,'audit_authored_production_edits_this_turn':False})
notes['checkpoint']['new_receipts'].extend([NAME,'support_driver_probe_receipt.json'])
addition=f'''

### Support studies, custody qualifiers and updater comparison review, 2026-10-17

All13 files in this slice are fully read (133,925 characters): intake attributes, complete scene study/embedded payloads, backdrop helpers, three Dramatis drivers and six standalone updater-comparison sources/manifests. Coverage is {stats['complete_files']:,}/{stats['total_files']:,} files and {stats['read_chars']:,}/{stats['total_chars']:,} characters.722 source files remain. The entire live3,738-file inventory was reconciled against tracked/untracked source membership and every source SHA256; no additions, removals or changes. This verifies current evidence, without crediting unread source.

Q422: the package qualifier selects an exact ignored tamper test but accepts exit0 without requiring that test to execute. Its unchanged run helper accepts a synthetic0-tests log; the nonzero control refuses. Q423: the review qualifier edits production consent source in place and blindly restores saved bytes in finally. Exact AST-function probes with an entirely in-memory source show an intervening owner edit overwritten while its restoration assertion passes; no-conflict restoration control passes. The audit did not mutate product source, compile a consent bypass, sign/extract an installer, launch native dialog or operate a vault. These probes establish runner admission/custody flaws, not that a retained real package ran zero tests or any actual concurrent edit was lost.

A1274-A1281 keep intentional studies and historical evidence distinct from product authorities. Browser presentation rules are a local JS model with a Rust parity seam, and the force solver explicitly identifies itself as a bounded study. Occurrence/resource/pin distinctions are visible, while widget persistence is optional/size-limited and failures suppressed. The PNG/overlay helpers can overwrite outputs or update consumer locks and were not run. Native consent uses a synthetic process-owned dialog and programmatic button messages; source mutation belongs to the separate review-driver risk. The Luggage/Velopack/TUF comparison explicitly verifies dated known gaps and source/evidence hashes; it must not be called current safe-updater acceptance after product source changes. No comparison pipeline ran.

Four extracted-function CPU cases pass with explicit process/source doubles. All protected source/shared work, report prefix and index are preserved. No binary, isolated target/home/worktree or live audit process remains; retained evidence stays below100MB. Local/upstream remain {head}/{upstream}. Evidence: [checkpoint](../../../support/receipts/authority-success-audit/{NAME}) and [qualifier probes](../../../support/receipts/authority-success-audit/support_driver_probe_receipt.json). Continue supplier patches/source, starting burn-cubecl.
'''
doc.write_bytes(prefix+addition.encode())
for name,value in [('review_notes.json',notes),('authority_map.json',authority)]: (HERE/name).write_text(json.dumps(value,indent=2)+'\n',encoding='utf-8')
assert subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True).returncode==0
budget=sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file());assert budget<100_000_000
evidence=['record_support_qualifier_review.py','support_driver_probe_receipt.json',*probe['evidence_sha256']]
record={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':head,'upstream_commit':upstream,'prior_receipt':{'path':'scripts_review_receipt.json','sha256':sha(HERE/'scripts_review_receipt.json')},
 'coverage':stats,'source_sha256':sources,'preserved_production_and_test_hashes':guards,'auxiliary_read_sha256':aux,'runtime_input_sha256':runtime,'shared_working_file_sha256':shared,'external_context_sha256':external,
 'new_findings':[v[0] for v in new],'authority_entries':ids,'full_repository_source_read':False,'next_read':next_read,'pending_verification':pending,'audit_authored_production_edits_this_pass':False,
 'all_inventory_live_membership_and_hashes_verified':{'files':len(i),'added':[],'removed':[],'changed':[]},
 'document':{'path':old['document']['path'],'sha256':sha(doc),'preserved_prefix_bytes':len(prefix),'preserved_prefix_sha256':hashlib.sha256(prefix).hexdigest()},
 'evidence_sha256':{p:sha(HERE/p) for p in evidence},'git_index_sha256':sha(index),'retained_isolation':[],'shared_generated_cache':old['shared_generated_cache'],
 'receipt_budget':{'cap_bytes':100_000_000,'retained_bytes_before_receipt':budget},'qualification':'Progress, not full repository completion.13 files133925 chars.4 original AST-function cases with explicit process/in-memory-source doubles. No live mutation/native/signing/installer/comparison run; no binaries/isolation retained. Full source membership/hash reconciliation unchanged.'}
with (HERE/NAME).open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
assert doc.read_bytes().startswith(prefix) and sha(index)==old['git_index_sha256']
assert all(sha(ROOT/p)==h for p,h in {**guards,**sources,**aux,**runtime}.items())
assert all(sha(ROOT/p)==h for p,h in shared.items() if p!=old['document']['path'])
print(json.dumps({'receipt':NAME,'coverage':stats,'findings':record['new_findings'],'authority_entries':ids,'budget_bytes':budget,'next_read':next_read},indent=2))
