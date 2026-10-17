"""Record Gazette, remaining Moot source and Djinn personal-facts test review."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess,tomllib
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];NAME='gazette_moot_review_receipt.json'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8-sig'))
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT,text=True).strip()
assert not (HERE/NAME).exists()
old=load('design_receipt_review_receipt.json');head=git('rev-parse','HEAD');upstream=git('rev-parse','origin/main')
assert head==old['basis_commit'] and upstream==old['upstream_commit']
guards={**old['preserved_production_and_test_hashes'],**old['source_sha256']};aux=dict(old['auxiliary_read_sha256'])
runtime=old['runtime_input_sha256'];shared=old['shared_working_file_sha256']
assert all(sha(ROOT/p)==h for p,h in {**guards,**aux,**runtime}.items())
assert all(sha(HERE/p)==h for p,h in old['evidence_sha256'].items())
doc=ROOT/old['document']['path'];prefix=doc.read_bytes();assert sha(doc)==old['document']['sha256']
assert all(sha(ROOT/p)==h for p,h in shared.items() if p!=old['document']['path'])
index=Path(git('rev-parse','--git-path','index'))
if not index.is_absolute():index=ROOT/index
assert sha(index)==old['git_index_sha256']
inventory=load('source_inventory.json')['files'];coverage=load('read_coverage.json')
paths=['ports/djinn/tests/personal_facts.rs']+[p for p in inventory if p.startswith('ports/gazette/')]
paths+=['ports/moot/'+p for p in ['examples/commons_practice_peer.rs','src/captured_web.rs','src/conversation.rs','src/lib.rs','tests/conversation.rs']]
assert len(paths)==12 and sum(inventory[p]['chars'] for p in paths)==174538
sources={p:sha(ROOT/p) for p in paths}
for p,h in sources.items():
    assert h==inventory[p]['sha256']==coverage[p]['sha256'] and coverage[p]['through']==inventory[p]['chars']
assert load('pending_read.json')==[]
probe=load('gazette_owner_probe_receipt.json');assert probe['exit']==0 and probe['passed']==23 and probe['original_tests']==20
assert all(sha(ROOT/p)==h for p,h in probe['source_sha256'].items())
assert all(sha(HERE/p)==h for p,h in probe['evidence_sha256'].items())
assert len(probe['removed_outputs'])==4 and all(not Path(item['path']).exists() for item in probe['removed_outputs'])
root_lock=tomllib.loads((ROOT/'Cargo.lock').read_text());test_lock=tomllib.loads((HERE/'cpu-gazette-runtime/Cargo.lock').read_text())
supplier_sources={}
for name in ['gaz','genet-static-dom','fleece']:
    actual=[p['source'] for p in test_lock['package'] if p['name']==name and 'source' in p]
    expected=[p['source'] for p in root_lock['package'] if p['name']==name and 'source' in p]
    assert len(actual)==1 and actual==expected;supplier_sources[name]=actual[0]
aux['ports/gazette/README.md']=sha(ROOT/'ports/gazette/README.md')
external=Path('C:/Users/mark_/.cargo/git/checkouts/dramatis-c1632f2f3bdc2597/cf562f4/crates/gaz/src/intake/acct.rs')
notes=load('review_notes.json');authority=load('authority_map.json')
assert notes['findings'][-1]['id']=='Q396' and authority['entries'][-1]['id']=='A1186'
new=[
 ('Q397','Ledger citation reports instance deltas honored without resolving or applying them','ports/gazette/src/ledger.rs:239-279',
  'A citation with unknown view, foreign source adapter/absent contact/facet and invalid selection text returns honored_instance_deltas=1. Resolver only validates envelope, compares caller-supplied generation strings, parses instances and counts them. Moved generation and malformed instance JSON controls refuse.',
  'Return parsed/validated delta counts honestly, or resolve addresses, deserialize selection and apply against the owning projection before reporting honored. The API receives no projection to mutate; no current live host/store adoption or lost user state is asserted. Current parity test repeats the same shallow receipt check.'),
 ('Q398','WebFinger resolver prepends a second scheme to uppercase ACCT identifiers','ports/gazette/src/lib.rs:218-250; ports/gazette/src/intake.rs:82-98',
  'normalize_resource(ACCT:Alice@EXAMPLE.org) returns acct:ACCT:Alice@EXAMPLE.org, and endpoint_url sends that exact resource query. Actual current Gaz-backed intake normalizes original input to acct:Alice@example.org and refuses the resolver-normalized result against that subject. Lowercase control agrees.',
  'Use one case-insensitive account parse at the request boundary while preserving username case and URL-resource support. Shared Gaz normalization already supplies account comparison; the documented future finger-protocol migration must preserve it. No external request was sent by this case.'),
 ('Q399','WebFinger labels generic JSON-LD links as ActivityPub actors','ports/gazette/src/lib.rs:168-211,408-414',
  'A describedby link with media type application/ld+json and an ordinary linked-data target is placed in activitypub_actors without any ActivityStreams profile/context evidence. The same classifier also uses case-sensitive URI prefix groups: uppercase GEMINI goes to catch-all while lowercase enters capsules.',
  'Recognize ActivityStreams media types/profile parameters explicitly and retain unrelated JSON-LD links as generic endpoints with source metadata. Normalize scheme recognition through the URI parser. This is classification of supplied unverified claims, not demonstrated authentication bypass, delivered content or live federation failure.')]
for ident,title,source,evidence,repair in new:
    notes['findings'].append({'id':ident,'title':title,'source':source,'evidence':evidence,'status':'reproduced current public CPU API behavior; documented, not repaired',
      'scope':'Actual unchanged Gazette crate root and exact current Gaz/Genet suppliers, with original positive/negative controls. No live contacts or external service changed.',
      'repair_triage':{'disposition':'documented in read-only full-source audit','reason':repair},'verification':NAME,'diagnostics':['gazette_owner_probe_receipt.json']})
rows=[
 ('Ledger retains a private projection alongside shared matrix/grid','Production ledger.rs builds cells, contributor lists, HTML, instance addresses and shelfmarks itself; shared path appears in parity tests.',
  ['ports/gazette/src/ledger.rs','ports/gazette/tests/ledger_parity.rs','ports/gazette/Cargo.toml'],
  'Parity test explicitly says retirement onto scenomise::matrix and FrozenGrid waits for the dramatis split. Equality scaffolding is useful, but passing the original three Ledger tests does not complete this migration or retire the duplicate writer. Parity integration source read, not executed in the lightweight CPU root.'),
 ('Gazette resolver and intake have divergent account admission','Old blocking resolver normalization coexists with new account-bound Gaz intake and planned sans-io protocol ownership.',
  ['ports/gazette/src/lib.rs','ports/gazette/src/intake.rs','ports/gazette/src/intake/tests.rs','ports/gazette/README.md'],
  'Q398 proves uppercase scheme divergence. Intake correctly uses actual shared Gaz normalization, requires matching explicit returned acct subject, refuses alias-based contact selection and emits unverified addresses rather than keys. Original conflict/dedup/trust-preservation tests pass. Resolver still builds its own reqwest client/timeout, outside future caller-supplied persona networking policy.'),
 ('Typed address classification is not authentication or lossless JRD custody','Classified resolver fields feed an account-bound address projection; source is preserved as classified data.',
  ['ports/gazette/src/lib.rs','ports/gazette/src/intake.rs'],
  'Q399 is a classifier defect in supplied claims. DID aliases remain opaque names, not verified keys. Intake filters malformed URI/endpoint groups and retains source. Known links lose some relation/media detail during classification by design; docs explicitly disclaim lossless original JRD archival. Announcing, other resolvers and live store wiring remain unbuilt.'),
 ('Ledger receipt validates generation labels but does not replay state','resolve_ledger_shelfmark compares role generations and counts parseable deltas.',
  ['ports/gazette/src/ledger.rs','ports/gazette/tests/ledger_parity.rs'],
  'Q397 unknown instance and malformed selection acceptance reproduced. Caller-supplied role map is not a fetched adapter/record proof, and no host apply callback exists in this API. Keep citation vocabulary, authority lookup, semantic validation and actual application receipts distinct.'),
 ('Supplied article extraction reuses Fleece and respects response syntax','Gazette accepts declared HTML/XHTML and delegates extraction over StaticDocument.',
  ['ports/gazette/src/lib.rs','crates/fleece/src/lib.rs'],
  'Current original tests execute real Genet static HTML/XML DOM plus Fleece extraction; non-HTML and navigation-only controls refuse. This is actual shared extraction reuse, not a private fetcher/parser implementation. Future feed scheduling, acquisition, reader persistence and room remain separate unbuilt owner work.'),
 ('Moot conversation separates pane state from shared write authority','Pane owns attention/drafts; snapshot reads authority-filtered encrypted Commons history; SendIntent requires owner recheck.',
  ['ports/moot/src/conversation.rs','ports/moot/tests/conversation.rs','ports/moot/src/lib.rs'],
  'Read all current source and integration tests. Stable author/operation IDs and causal ordering are projected, withheld content contributes counts, foreign refresh/version errors refuse, and successful preparation does not clear a draft or grant membership. Consumer-supplied lifecycle consistency is explicit; preparation after revocation is intentionally an intention, not send success. These integration tests were not rerun this pass.'),
 ('Captured search is a remintable projection over authorized fauna','Moot filters current signed fauna, validates resolved Fleece integrity and groups exact canonical text with retained contribution identities.',
  ['ports/moot/src/captured_web.rs'],
  'Canonical schema/normalization/profile/text hash determine groups; aliases and latest contribution are search projection choices. Missing/unsupported/invalid records stay explicit, withdrawals remove their contribution on remint. Caller owns custody map binding and current authority. Source and its controls read, not freshly executed.'),
 ('Practice peer uses real governance inside explicit proof fixtures','One process/store, fixed public identities and proof clock compose actual signed delegation lane and Commons projection.',
  ['ports/moot/examples/commons_practice_peer.rs'],
  'Constitution/space/founder/container binding and tamper checks are substantial. Revoked/expired peers may still exchange retained statements so revocations can arrive; authoring/recheck use current admission. Installed versus authorized is deliberate, not authorization bypass. File envelope/local lifecycle writes are proof persistence, not shared production custody. Self-check folds JSON booleans but main returns Ok after printing; outer callers must inspect JSON, pending review of actual runner. No transport, production key or live peer receipt claimed.'),
 ('Personal facts test crosses real processes and two app identities','Djinn original test launches create and reopen children over a shared temporary reservoir.',
  ['ports/djinn/tests/personal_facts.rs'],
  'Read fresh-owner process spawning, two admitted app clients, corrected immutable birth records, associations and whole-catalog reopen equality. Public fixed seeds and synthetic facts only. This source is stronger than an in-process clone test but was not executed here; platform socket/process acceptance is not inferred from reading.')]
ids=[]
for number,(domain,claim,implementations,assessment) in enumerate(rows,1187):
    ident=f'A{number}';ids.append(ident);authority['entries'].append({'id':ident,'domain':domain,'claim':claim,'implementations':implementations,'assessment':assessment,'state':'current source fully read; selected real CPU behavior separately qualified; full audit ongoing','verification':NAME})
for area,detail in [
 ('Gazette ownership migration','Private Ledger remains until dramatis split; parity scaffolding is not consumer migration. Old blocking WebFinger fetch/normalization remains pending sans-io protocol lift, while intake already uses shared Gaz account identity.'),
 ('Moot practice runner success boundary','Main prints self-check JSON and returns Ok even if aggregate ok=false; inspect actual caller handling before treating this as a separate false-success bug. Do not run fixture against a real store or reinterpret public fixed-key proof identities as product identity ownership.'),
 ('Gazette test override lifetime','Test-only global override restores only on ordinary closure return, with a separate mutex held across run. Panic/nested-run behavior remains a source followup; not production fetch state.')]:
    notes['review_followups'].append({'area':area,'detail':detail,'verification':NAME})
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==2808 and stats['read_chars']==32769091 and not stats['changed_since_inventory'] and stats['pending_chunks']==0
notes['completed_source_areas'].append({'prefix':'ports/gazette plus five remaining Moot files and Djinn personal facts test','files':12,'chars':174538,'verification':NAME,'basis_commit':head,'qualification':'All inventoried Gazette and Moot source now read; Gazette original unit suite and three audit cases run. Moot and Gazette parity integrations source-only this pass.'})
assert all(coverage[p].get('sha256')==v['sha256'] and coverage[p].get('through')==v['chars'] for p,v in inventory.items() if p.startswith(('ports/gazette/','ports/moot/')))
next_read={'path':'ports/muniment-opfs-probe/.cargo/config.toml','start':0,'reason':'Continue OPFS probe and remaining ports/scripts/vendor patch source without narrowing inventory.'}
pending='No audit-owned process running.23 Gazette CPU tests passed (20 original,3 audit); four owned exe/PDB files removed. Moot and Gazette parity integrations not rerun. Local main/upstream unchanged; eight reviewed upstream commits not installed because shared report overlaps. Full repository audit remains incomplete.'
notes['current_area']='Gazette and remaining Moot source reviewed; continue OPFS probe'
notes['next_area']=next_read['reason'];notes['checkpoint'].update({'classification':'progress','coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),'next_read':next_read,'pending_verification':pending})
notes['checkpoint']['new_receipts'].extend([NAME,'gazette_owner_probe_receipt.json'])
addition=f'''

### Gazette and remaining Moot ownership review, 2026-10-17

Read 12 more source files (174,538 characters): all six Gazette source/manifest/test files, five remaining Moot files and Djinn's personal-facts real-process test. Gazette and Moot inventoried source are now completely read. Overall coverage is {stats['complete_files']:,}/{stats['total_files']:,} files and {stats['read_chars']:,}/{stats['total_chars']:,} characters, with no stale or pending credit. This remains a partial repository audit. Main/upstream remain {head}/{upstream}; previously reviewed upstream changes are still not installed.

Q397: Ledger resolve reports an unknown view/source/contact/facet delta honored after merely parsing/counting it, and accepts malformed selection text. Generation mismatch and invalid instance JSON controls refuse. It has no projection argument or application callback, so receipt wording overstates replay. Q398: uppercase ACCT:Alice@EXAMPLE.org becomes acct:ACCT:Alice@EXAMPLE.org in resolver normalization and the actual resource query. Current Gaz-backed intake instead recognizes acct:Alice@example.org and refuses the malformed resolver result against that subject; lowercase control agrees. Q399: any application/ld+json link is classified as an ActivityPub actor without an ActivityStreams profile, and URI prefix case changes typed classification. These are supplied-claim classification errors, not evidence of authenticated identity or live federation compromise.

All 20 original Gazette unit tests and three additional public API audit cases passed against its actual unchanged crate root and current lock-seeded Gaz/Genet suppliers. Existing intake conflict, alias, trust-preservation, URI validation, declared HTML/XHTML extraction and loopback HTTP controls passed. Gazette parity integration and Moot integration files were read, not rerun. No external fetch, live contact mutation, browser/layout/GPU acceptance or production edit. Four exact completed-run exe/PDB outputs were marker-checked and removed; stable shared dependency cache remains. No isolated target/home/worktree was created.

A1187-A1195 track remaining ownership work beside the bugs. Ledger still privately constructs cells, HTML and shelfmarks; its shared matrix/grid parity tests explicitly defer retirement to the dramatis split. Gazette intake already uses shared Gaz identity, while old blocking reqwest fetch and permissive normalization await the documented sans-io protocol migration. Supplied Fleece extraction is actual reuse and does not own future feed polling/storage. Moot's pane keeps drafts/attention local, uses authority-filtered Commons snapshots and returns an intention requiring owner recheck before authoring; prepare_send is not send success. Captured search is reminted from authorized fauna and resolved capture integrity, preserving contribution provenance. The practice peer uses real governance but fixed public proof identities/clock and proof persistence, not a production owner or transport. Its self-check exit versus JSON handling is left as a caller-tracing followup. Djinn's two-app/fresh-process persistence test is source-read only.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/{NAME}) and [Gazette actual CPU tests](../../../support/receipts/authority-success-audit/gazette_owner_probe_receipt.json). Prior report bytes, guarded source, shared WIP and index are preserved. Receipt bytes stay below 100 MB. Continue with OPFS probe, remaining ports, scripts and supplier patches.
'''
doc.write_bytes(prefix+addition.encode('utf-8'))
for n,v in [('review_notes.json',notes),('authority_map.json',authority)]: (HERE/n).write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8')
assert subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True).returncode==0
evidence=['record_gazette_moot_review.py','gazette_owner_probe_receipt.json',*probe['evidence_sha256']]
budget=sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file());assert budget<100_000_000
record={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':head,'upstream_commit':upstream,'prior_receipt':{'path':'design_receipt_review_receipt.json','sha256':sha(HERE/'design_receipt_review_receipt.json')},
 'coverage':stats,'source_sha256':sources,'preserved_production_and_test_hashes':guards,'auxiliary_read_sha256':aux,'runtime_input_sha256':runtime,'shared_working_file_sha256':shared,
 'external_context_sha256':{str(external):sha(external)},'supplier_sources':supplier_sources,'new_findings':[r[0] for r in new],'authority_entries':ids,
 'all_gazette_moot_inventory_source_read':True,'full_repository_source_read':False,'next_read':next_read,'pending_verification':pending,'audit_authored_production_edits_this_pass':False,
 'document':{'path':old['document']['path'],'sha256':sha(doc),'preserved_prefix_bytes':len(prefix),'preserved_prefix_sha256':hashlib.sha256(prefix).hexdigest()},
 'evidence_sha256':{p:sha(HERE/p) for p in evidence},'git_index_sha256':sha(index),'retained_isolation':[],'shared_generated_cache':old['shared_generated_cache'],
 'receipt_budget':{'cap_bytes':100_000_000,'retained_bytes_before_receipt':budget},'qualification':'Progress, not full repository completion.12 source files read;23 actual Gazette CPU tests pass, including3 audit cases. Four run-owned binaries removed. Selected integration source read, not executed. No production edits or new isolation.'}
with (HERE/NAME).open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
assert doc.read_bytes().startswith(prefix) and sha(index)==old['git_index_sha256']
assert all(sha(ROOT/p)==h for p,h in {**guards,**sources,**aux,**runtime}.items())
assert all(sha(ROOT/p)==h for p,h in shared.items() if p!=old['document']['path'])
print(json.dumps({'receipt':NAME,'coverage':stats,'findings':record['new_findings'],'authority_entries':ids,'budget_bytes':budget,'retained_isolation':[],'next_read':next_read},indent=2))
