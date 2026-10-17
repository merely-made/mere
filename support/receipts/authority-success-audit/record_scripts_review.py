"""Record complete remaining root script reads and scoped CPU controls."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];NAME='scripts_review_receipt.json'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8-sig'))
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT,text=True).strip()
assert not (HERE/NAME).exists()
old=load('signalman_tabard_review_receipt.json');head=git('rev-parse','HEAD');upstream=git('rev-parse','origin/main')
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
i=load('source_inventory.json')['files'];c=load('read_coverage.json');scripts=[p for p in i if p.startswith('scripts/')];paths=[p for p in scripts if p not in guards]
assert len(scripts)==24 and len(paths)==20 and sum(i[p]['chars'] for p in paths)==176275
assert all(sha(ROOT/p)==i[p]['sha256']==c[p]['sha256'] and c[p]['through']==i[p]['chars'] for p in scripts)
assert load('pending_read.json')==[];sources={p:sha(ROOT/p) for p in paths}
probe=load('script_owner_probe_receipt.json');assert probe['exit']==0 and probe['export_cases_passed']==3 and probe['source_classifier_cases_passed']==4 and probe['original_unit_tests_passed']==4
assert all(sha(ROOT/p)==h for p,h in probe['source_sha256'].items())
assert all(sha(HERE/p)==h for p,h in probe['evidence_sha256'].items())
for p,h in probe['external_context_sha256'].items():assert sha(Path(p))==h;external[p]=h
assert not (ROOT/'scripts/remote-receipt.ps1').exists()
notes=load('review_notes.json');authority=load('authority_map.json')
assert notes['findings'][-1]['id']=='Q418' and authority['entries'][-1]['id']=='A1263'
new=[
 ('Q419','Cross-repo smoke falsely certifies registry and non-HTTPS Git dependencies as local',
  'scripts/cross-repo-smoke.ps1:82-119',
  'Assert-Local promises that the edited sibling resolves to a local path. Its only remote-source test is a rendered first Cargo tree line containing (https://. A registry package line and an ssh:// Git line pass and print local. Four executions of the exact AST-extracted PowerShell function with Cargo tree doubles reproduce those false positives and retain controls for HTTPS refusal and local-path acceptance. No comparison to the requested sibling path exists.',
  'Inspect Cargo metadata source and resolved manifest_path, require source=null and the expected local checkout owner. Do not use display formatting or absence of one scheme as a source authority. Modern immutable Git consumption may be intentional; update lane policy before requiring local patches everywhere.',
  'Four exact-function PowerShell cases with deterministic Cargo output and an existing harmless working directory. No actual cargo graph/build or remote repository mutation.'),
 ('Q420','Firefox metadata matching drops valid unclaimed visits outside its four-position search window',
  'scripts/firefox_history_export.py:90-130',
  'visit_view_times promises nearest unclaimed same-place visit within tolerance, but examines only pivot-2 through pivot+1. With visits1000..1004ms and five metadata rows at1002ms, tolerance10ms, it matches four and drops the fifth while visit5 at1004ms remains unclaimed and within tolerance. The production function runs unchanged against in-memory SQLite. Two ordinary matches and an out-of-tolerance refusal control also pass. Existing original export test exercises parent identity and microseconds, not metadata matching.',
  'Search outward through the tolerance interval for an unclaimed candidate, preserving an explicit deterministic tie rule. Avoid rebuilding every timestamp array for each metadata row. This fixes the stated one-to-one heuristic; it does not turn heuristic matching into exact Firefox visit provenance.',
  'Three direct current-function calls with synthetic in-memory SQLite only. No browser profile, actual Firefox history schema/version compatibility or Rust ingestion run.'),
 ('Q421','Wayland and X11 receipt wrappers call an absent helper after host extraction',
  'scripts/wayland-frame-receipt.ps1:46-69; scripts/x11-shadow-receipt.ps1:143-176',
  'Both wrappers resolve remote-receipt.ps1 beside themselves under Mere/scripts, where it does not exist. That helper is present in the sibling Genet checkout. Both also default the remote repository and Repo identity to Genet while naming Cambium host/example/scenario paths now in Mere. Current Genet manifest explicitly records Pelt/Tabard/Cambium extraction; Mere manifest owns the host. These wrappers cannot launch their named checked-in helper as written. No actual SSH or native run was attempted.',
  'Choose one maintained runner owner and explicit source repository, then bind wrapper/helper/package/scenario/commit/executable provenance consistently. Avoid copying another runner implementation as an extraction workaround. Add a read-only dependency/preflight check before remote process setup.',
  'Complete wrapper source read plus read-only current filesystem/helper membership and targeted current Mere/Genet manifest evidence. This is a missing-path/source-owner finding, not an executed platform failure.')]
for ident,title,source,evidence,repair,scope in new:
 notes['findings'].append({'id':ident,'title':title,'source':source,'evidence':evidence,'status':'reproduced source-owner classifier/export behavior or confirmed missing helper; documented, not repaired','scope':scope,'repair_triage':{'disposition':'documented in read-only full-source audit','reason':repair},'verification':NAME,'diagnostics':['script_owner_probe_receipt.json']})
rows=[
 ('Build-topology authority lags repository extraction','Cross-repo smoke hardcodes owners/package names and local-path assumptions independently of Cargo manifests.',
  ['scripts/cross-repo-smoke.ps1','Cargo.toml'],
  'It still invokes Genet cargo check -p pelt despite current Genet extraction and Mere owning Pelt. The script will count build failures, including missing repositories, rather than simply declaring green. This fail-closed behavior is real; the maintained net is stale, and Q419 undermines its local-source assertion. Logs overwrite fixed target/smoke filenames and script does not bind source/executable hashes. No whole lattice run performed.'),
 ('Port boundaries are an enforceable but bounded dependency assertion','check_port_boundaries checks direct crate path edges into ports and a named WASM forbidden-package cone.',
  ['scripts/check_port_boundaries.py'],
  'Cargo metadata errors and tree errors fail. It resolves symlinks before beneath checks and requires the Graphshell manifest at its actual port path. The test is direct manifest path dependencies plus a denylist, not proof of every possible external source/transitive authority or complete feature matrix. It does not use --locked/--offline, so do not run casually in this read-only shared lane.'),
 ('Windows native qualifier provenance still names the old repository','Maximized and Snap scripts execute Cargo from Mere repoRoot but emit repo=genet.',
  ['scripts/windows-maximized-receipt.ps1','scripts/windows-snap-receipt.ps1'],
  'The commit is Mere HEAD while the manifest repo label remains Genet. Both discover any new smoke process with matching title rather than binding the selected child image/hash; concurrent lanes can collide. Geometry/HTMAXBUTTON and app-result checks are real source assertions, but no current native run was performed. Snap success requires hit testing, foreground/cursor and a saved screenshot, with no automatic verification that the OS Snap popup is visible. Retain screenshot review as a distinct witness before claiming hover-popup acceptance.'),
 ('Remote native qualifiers retain shared temporary process identities','Wayland/X11 wrappers rely on a runner outside their current repository.',
  ['scripts/wayland-frame-receipt.ps1','scripts/x11-shadow-receipt.ps1'],
  'Q421 is the immediate missing helper. X11 also shares fixed /tmp release/probe/script paths, selects a window by title and ignores the background runner exit as a direct gate; it later checks app receipt/WM extents/trace. Wayland requires complementary decorated answers and anchored RESULT ok. XWayland scope is explicitly narrower than all X11 managers. These checks were read, not executed or generalized to physical/native acceptance.'),
 ('DR-C separates actual refusal from restored-fallback control','The driver requires matching child exits/content and a restored deliberate mutant in an isolated control checkout.',
  ['scripts/dr_c_receipts.py'],
  'Source lists/head/hash checks, compiler JSON test executable selection and exact1-passed filter are real safeguards. Absent/locked keeper endpoints are unique, and generated vault cleanup is marker- and process-bound. It still uses fixed reusable output logs/receipt paths with write mode, has unbounded ordinary subprocess calls, and assumes old target/debug binary paths. Do not execute unchanged in this audit: it mutates a control source and can overwrite evidence. A strict child check here is separate from prior misleading historical custody wrappers.'),
 ('Firefox exporter owns source identity but dwell matching remains a heuristic','Stable supplied profile/generation and original parent/microsecond identity flow into imported visits.',
  ['scripts/firefox_history_export.py','scripts/test_firefox_history_export.py'],
  'Q420 affects metadata-to-visit heuristic assignment. Parent IDs survive missing referrer rows, titles and transitions retain explicit conversions, and no URL is printed by success summaries. URL digest selection for frecency remains caller-coordinated with Rust; JSON sidecar has no algorithm envelope. The read-only SQLite URI is constructed from an unescaped path, and older schemas without moz_places_metadata are not handled. These compatibility/encoding issues need scoped composition evidence; no real profile opened.'),
 ('Documentation fact audit and judgment audit prove different claims','mere_doc_audit checks paths/status/header shape; judgment audit checks D2 record structure and coverage.',
  ['scripts/mere_doc_audit.py','scripts/mere_doc_judgment_audit.py','scripts/test_mere_doc_judgment_audit.py'],
  'Neither verifies present semantic correctness of every design claim. Judgment pins original snapshot, applies explicit corrections, lets later batches supersede, disambiguates full paths and checks totals. Four original Python tests (one export, three judgment) pass. Fact audit zero by default is a reporting mode; --fail-on-findings is explicit. It has planted-defect controls in source, but its self-test creates repositories and was not executed in this shared audit. Index markdown parsing differs from per-document fenced-code filtering; archived supplement matching uses basename. Followups, not fabricated whole-doc-review completion.'),
 ('Header rewriting has its own provenance ledger and write gate','relicense_headers shares header_violations with doc audit rather than duplicating the scanner.',
  ['scripts/relicense_headers.py','scripts/mere_doc_audit.py'],
  'Retained/frozen first-column ledger entries exempt exact subtrees; derivative sections do not. --apply refuses tracked dirty sources, supports bounded prefix and preserves upstream notices only under the declared retain/normalization paths. The header check tests presence of an opening marker across tracked file content, not full Exhibit-A syntax or author provenance. No licensing ruling/rewrite was performed. Self-tests and full audits remain unexecuted here.'),
 ('Scene study parity and packaging are separate authorities','Checker compares extracted browser rule model to a supplied Rust example; packer embeds fixture/face/media payloads.',
  ['scripts/mere_scene_study_check.mjs','scripts/mere_scene_study_pack.py'],
  'Checker actually requires Rust exit0 and matching results, malformed refusals, deterministic force/pin/overlap controls; force checks themselves are browser-only. Fixture/model hashes do not bind supplied Rust binary to current source/lock, and synchronous child runs have no deadline. Packer is a production-asset mutation and was not run. Upcoming support HTML read must trace whether this is study-only duplication or a maintained runtime authority.'),
 ('Owner-local SSH controls and toolchain/native launcher stay operational plumbing','SSH setup scripts and thin custody framing client do not create a second wallet/signing engine.',
  ['scripts/codex-peer-enroll-windows.ps1','scripts/codex-ssh-shell-windows.ps1','scripts/djinn-ssh-control.py','scripts/repo-toolchain.ps1','scripts/run_macos_scenario.py'],
  'Setup gates on elevation and marked config, preserves snapshot/restores/concurrent registry checks, and modifies system SSH/firewall only when invoked. Python client repeats broker handshake/frame transport with16MiB read cap and chooses resident roster profile; signing stays on agent route. Windows pipe timeout differs from Unix socket. Toolchain helper reads pin and forbids fallback/download. macOS launcher binds a copied binary, unique process path, first receipt line and timeout rather than open exit alone; scenario hash is sampled after run, not sealed before. None of these system/native paths executed.')]
ids=[]
for number,(domain,claim,implementations,assessment) in enumerate(rows,1264):
 ident=f'A{number}';ids.append(ident);authority['entries'].append({'id':ident,'domain':domain,'claim':claim,'implementations':implementations,'assessment':assessment,'state':'all root script source read; extraction/provenance/custody boundaries tracked','verification':NAME})
notes['review_followups'].extend([
 {'area':'Receipt runner extraction and custody','detail':'Q421 missing helper, Genet labels around Mere commits, stale Genet Pelt smoke step, title-only process selection, shared X11 tmp files, screenshot-only Snap popup witness, and fixed overwriteable DR-C logs are unfinished runner ownership migration. Consolidate source/process/output identity at a maintained owner before treating these scripts as current platform gates.','verification':NAME},
 {'area':'Exporter and documentation detector limits','detail':'Q420 dense metadata loss reproduced; SQLite URI path escaping, old metadata schema, frecency digest envelope and matching heuristic provenance need separate checks. Doc audits are path/record coverage detectors, not proof of semantic truth. Preserve that distinction in current acceptance claims.','verification':NAME}])
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==3003 and stats['read_chars']==35106445 and not stats['changed_since_inventory'] and stats['pending_chunks']==0
notes['completed_source_areas'].append({'prefix':'Remaining root scripts','files':20,'chars':176275,'verification':NAME,'basis_commit':head,'qualification':'Completes all24 inventoried root scripts.3 actual exporter cases,4 extracted PowerShell classifier cases,4 original Python tests pass. No full builds/native/system/SSH/profile mutation; no binary/isolation retained.'})
next_read={'path':'support/browser-history-intake/.gitattributes','start':0,'reason':'Continue support intake/study/receipt drivers, then supplier patches and source; keep all remaining inventory.'}
pending='No audit-owned process running. All root scripts read;735 source files remain.3 exporter cases+4 source classifier cases+4 original Python tests pass. No production edit, generated binary, isolation or system/native/SSH action.'
notes['current_area']='All root scripts complete; support source next';notes['next_area']=next_read['reason']
notes['checkpoint'].update({'classification':'progress','classification_of_previous_turn':'progress','coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),'next_read':next_read,'pending_verification':pending,'audit_authored_production_edits_this_turn':False})
notes['checkpoint']['new_receipts'].extend([NAME,'script_owner_probe_receipt.json'])
addition=f'''

### Root scripts and qualifier ownership review, 2026-10-17

The remaining20 root scripts are fully read (176,275 characters), completing all24 inventoried scripts. Overall coverage is {stats['complete_files']:,}/{stats['total_files']:,} files and {stats['read_chars']:,}/{stats['total_chars']:,} characters, with no stale/pending credit.735 source files remain. Local/upstream remain {head}/{upstream}; no production code changed.

Q419: the smoke driver's Assert-Local certifies registry and ssh:// Git sources as local because it checks only for the rendered substring (https://. Four executions of the exact PowerShell function reproduce those false positives, with HTTPS-refusal and path-acceptance controls. Q420: Firefox metadata matching only searches four positions around the time pivot, dropping a valid fifth unclaimed visit inside tolerance. Three direct unmodified-exporter calls against synthetic in-memory SQLite reproduce it and retain ordinary/out-of-tolerance controls. Q421: both Wayland and X11 wrappers call Mere/scripts/remote-receipt.ps1, which is absent; their helper remains in Genet, while host/package/scenario ownership moved to Mere. This last finding is source/membership evidence, not an attempted remote run.

A1264-A1273 track extraction and independent authority drift. Cross-repo smoke still asks Genet to build Pelt; Windows receipt manifests label Mere HEAD as repo=genet. Native process selection is title/time-based, X11 shares fixed tmp paths, Snap success checks hit testing and saves a screenshot without testing popup visibility. These remain weaker witnesses than a bound process/image/platform result. DR-C has real refusal/mutant/source controls but overwriteable fixed logs, old output-path assumptions and no ordinary subprocess deadline. It was not executed. SSH setup/custody client, toolchain helper and LaunchServices launcher remain operational plumbing; none were used to change the system or contact another host.

The documentation detectors check path/header shape and judgment-record coverage rather than semantic truth. They share the header scanner correctly. Study rule parity checker and asset packer have separate source/binary and mutation boundaries; the support study source is next. Four original Python tests pass (one exporter, three judgment), in addition to the seven targeted classifier/exporter cases. No actual browser profile, cross-repo build, native window, physical input/AT, SSH or system setup ran. No new binary, isolated target/home/worktree or live audit process remains. Report prefix, all protected source/shared work and index are preserved. Evidence: [checkpoint](../../../support/receipts/authority-success-audit/{NAME}) and [script probes](../../../support/receipts/authority-success-audit/script_owner_probe_receipt.json).
'''
doc.write_bytes(prefix+addition.encode())
for name,value in [('review_notes.json',notes),('authority_map.json',authority)]: (HERE/name).write_text(json.dumps(value,indent=2)+'\n',encoding='utf-8')
assert subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True).returncode==0
budget=sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file());assert budget<100_000_000
evidence=['record_scripts_review.py','script_owner_probe_receipt.json',*probe['evidence_sha256']]
record={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':head,'upstream_commit':upstream,'prior_receipt':{'path':'signalman_tabard_review_receipt.json','sha256':sha(HERE/'signalman_tabard_review_receipt.json')},
 'coverage':stats,'source_sha256':sources,'preserved_production_and_test_hashes':guards,'auxiliary_read_sha256':aux,'runtime_input_sha256':runtime,'shared_working_file_sha256':shared,'external_context_sha256':external,
 'new_findings':[v[0] for v in new],'authority_entries':ids,'all_root_scripts_inventory_source_read':True,'full_repository_source_read':False,'next_read':next_read,'pending_verification':pending,'audit_authored_production_edits_this_pass':False,
 'document':{'path':old['document']['path'],'sha256':sha(doc),'preserved_prefix_bytes':len(prefix),'preserved_prefix_sha256':hashlib.sha256(prefix).hexdigest()},
 'evidence_sha256':{p:sha(HERE/p) for p in evidence},'git_index_sha256':sha(index),'retained_isolation':[],'shared_generated_cache':old['shared_generated_cache'],
 'receipt_budget':{'cap_bytes':100_000_000,'retained_bytes_before_receipt':budget},'qualification':'Progress, not full repository completion.20 files176275 chars complete all24 root scripts.3 current exporter+4 extracted PS function cases+4 original Python tests pass. No full builds/native/SSH/browser profile/system actions; no new binary/isolation.'}
with (HERE/NAME).open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
assert doc.read_bytes().startswith(prefix) and sha(index)==old['git_index_sha256']
assert all(sha(ROOT/p)==h for p,h in {**guards,**sources,**aux,**runtime}.items())
assert all(sha(ROOT/p)==h for p,h in shared.items() if p!=old['document']['path'])
print(json.dumps({'receipt':NAME,'coverage':stats,'findings':record['new_findings'],'authority_entries':ids,'budget_bytes':budget,'next_read':next_read},indent=2))
