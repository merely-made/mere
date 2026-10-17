"""Four current Pictograph rereads and a fresh std-only cache ownership check."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];NAME='pictograph_shared_image_reread_receipt.json'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8-sig'))
def save(n,v):(HERE/n).write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8')
def git(*a):return subprocess.check_output(['git',*a],cwd=ROOT,text=True).strip()
assert not (HERE/NAME).exists()
old=load('upstream_cambium_reread_receipt.json');head=git('rev-parse','HEAD');assert head==old['basis_commit']
guards={**old['preserved_production_and_test_hashes'],**old['source_sha256']};assert len(guards)==1944
aux=old['auxiliary_read_sha256'];assert all(sha(ROOT/n)==h for n,h in {**guards,**aux}.items())
assert all(sha(HERE/n)==h for n,h in old['evidence_sha256'].items())
shared=old['shared_working_file_sha256'];doc=ROOT/old['document']['path'];prefix=doc.read_bytes();assert sha(doc)==old['document']['sha256']
assert all(sha(ROOT/n)==h for n,h in shared.items() if n!=old['document']['path'])
index=Path(git('rev-parse','--git-path','index'));index=index if index.is_absolute() else ROOT/index;index_hash=sha(index);assert index_hash==old['git_index_sha256']
inv,cov=load('source_inventory.json'),load('read_coverage.json');assert inv['commit']==head and load('pending_read.json')==[]
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT));assert stats['complete_files']==2396 and stats['read_chars']==27985549 and len(stats['changed_since_inventory'])==20 and not stats['pending_chunks']
reread=sorted(set(old['coverage']['changed_since_inventory'])-set(stats['changed_since_inventory']));assert len(reread)==4 and sum(inv['files'][n]['chars'] for n in reread)==127605
assert all(cov[n]['sha256']==sha(ROOT/n)==inv['files'][n]['sha256'] and cov[n]['through']==inv['files'][n]['chars'] for n in reread)
area='crates/canvas/pictograph/';files=[n for n in inv['files'] if n.startswith(area)]
assert len(files)==165 and sum(inv['files'][n]['chars'] for n in files)==2249061
assert all(cov[n]['sha256']==sha(ROOT/n)==inv['files'][n]['sha256'] and cov[n]['through']==inv['files'][n]['chars'] for n in files)
probe=load('shared_image_cache_probe_receipt.json');assert probe['build_exit']==probe['run_exit']==0 and probe['checks']==4 and probe['generated_output_removed']
assert probe['runner_sha256']==sha(HERE/'run_shared_image_cache_probe.py') and probe['probe_sha256']==sha(HERE/'shared_image_cache_probe.rs')
assert all(sha(ROOT/n)==h for n,h in probe['source_sha256'].items()) and all(not Path(v['path']).exists() for v in probe['removed_outputs'])
notes,authority=load('review_notes.json'),load('authority_map.json');assert notes['findings'][-1]['id']=='Q350' and authority['entries'][-1]['id']=='A963'
rows=[
 ('Decoded favicon/frame payload sharing is actual allocation reuse','The image cache and per-frame ImageResource carry Arc<Vec<u8>>.', ['crates/canvas/pictograph/src/canvas.rs','crates/canvas/pictograph/src/canvas/resolved_image_cache.rs','crates/canvas/pictograph/src/canvas/frame.rs'],'Cache insertion converts the owned Vec once and frame face_layer clones its Arc rather than copying bytes. Fresh unchanged cache module tests show two frame handles point to the same allocation. The frame call site is source-verified; no whole current Canvas/renderer build or GPU upload claim.'),
 ('Cache working-set accounting is narrower than total live image memory','ResolvedImageCache.used_bytes counts current entry lengths.', ['crates/canvas/pictograph/src/canvas/resolved_image_cache.rs','crates/canvas/pictograph/src/canvas/nodes.rs'],'A fresh check retains a frame Arc, evicts to a zero-byte cache bound and observes cache usage0 while the frame bytes remain valid. This is correct ownership, not evidence of a leak or violated cache bound. Do not promote the working-set counter to a bound on retained scenes, renderer caches or process memory.'),
 ('Scene sprite storage shares bytes but has a separate lifetime and policy','scene_sprite_textures persists across clear_scene and clones Arc into its image layer.', ['crates/canvas/pictograph/src/canvas.rs','crates/canvas/pictograph/src/canvas/nodes.rs','crates/canvas/pictograph/src/canvas/frame.rs'],'Scene registration converts its Vec to Arc and same-handle replacement replaces that registry entry. This is not the digest-keyed byte-bounded favicon LRU. Its documented persistent registry has separate ownership and no limit/removal surface in these methods. No claim that the favicon setting bounds scene-prop storage or all image memory.'),
 ('Canvas imagery remains a host-resolved projection rather than durable storage','Frame misses queue ImageRef digests and the host later registers decoded pixels.', ['crates/canvas/pictograph/src/canvas/nodes.rs','crates/canvas/pictograph/src/canvas/frame.rs'],'Miss coalescing and requested_images suppress repeated I/O; LRU evictions clear that request marker. The cache intentionally does not fetch durable bytes or prove the supplied RGBA corresponds to a digest. Oversize/rejected entries and retry after policy changes remain a source followup, not a newly reproduced product bug. A misplaced scene-sprite doc block still precedes register_resolved_image; frame module prose still cites a600LOCceiling although the current file is over1100lines.'),
]
entries=[{'id':f'A{964+i}','domain':d,'claim':c,'implementations':paths,'assessment':a,'state':'four current source files fully reread; fresh std-only private-cache tests; no whole renderer/native execution','verification':NAME} for i,(d,c,paths,a) in enumerate(rows)];authority['entries'].extend(entries)
areas=[{'prefix':area,'files':165,'chars':2249061,'verification':NAME,'basis_commit':head,'qualification':'All165Pictograph inventory files have complete matching-hash coverage after4whole-file rereads. Fresh cache-only checks do not qualify full Canvas/paint/dependency execution.'}]
notes['completed_source_areas'].extend(areas)
for a in notes['completed_source_areas']:
 pending=a.get('pending_current_revision_reread')
 if pending:
  remaining=[n for n in pending['paths'] if n in stats['changed_since_inventory']]
  a['pending_current_revision_reread']={'verification':NAME,'basis_commit':head,'paths':remaining,'resolved_paths':pending.get('resolved_paths',[])+[n for n in pending['paths'] if n not in remaining]}
  if not remaining:a['current_revision_restored_by']=NAME
transition={'verification':NAME,'basis_commit':head,'reread_files':reread,'reread_chars':127605,'remaining_upstream_rereads':stats['changed_since_inventory'],'qualification':'Full emitted, reviewed and acknowledged files; no diff credit.'}
notes.setdefault('source_revision_refreshes',[]).append(transition);authority.setdefault('source_revision_refreshes',[]).append(transition)
notes['review_followups'].append({'verification':NAME,'area':'Rejected image retry policy','detail':'requested_images intentionally coalesces missing/undecodable blobs. An oversize registration is rejected without reporting the requested digest as evicted; changing the limit does not explicitly invalidate negative requests. Trace host re-registration/invalidation and intended negative-cache lifetime before promoting a concrete permanent-missing-image bug.'})
next_read={'path':'crates/graphshell/graphshell-client/Cargo.toml','start':0,'reason':'Finish20upstream-changed full rereads, then every remaining unread source.'}
notes['current_area']='Pictograph restored on current upstream;four image-sharing rereads complete';notes['next_area']=next_read['reason']
notes['checkpoint']={'classification':'progress','classification_of_previous_turn':'progress','goal_status':'active; full source incomplete','coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),'new_receipts':[NAME,'shared_image_cache_probe_receipt.json'],'audit_authored_production_edits_this_turn':False,'next_read':next_read,'pending_verification':None,'diagnostic_outcomes':'Four fresh private cache checks pass:2original+2Arcownership observations. No whole Canvas/renderer/dependency/GPU/native run.'}
diff=subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True,text=True);assert diff.returncode==0
with (HERE/'pictograph_shared_image_diff_check.txt').open('x',encoding='utf-8') as f:f.write(diff.stdout+diff.stderr)
addition=f'''

### Current Pictograph image-sharing reread, 2026-10-17

Four changed files /127,605characters are fully reread at unchanged HEAD{head}. All165Pictograph source files /2,249,061characters now have current matching-hash coverage. Overall valid coverage is{stats['complete_files']:,}/{stats['total_files']:,}files and{stats['read_chars']:,}/{stats['total_chars']:,}characters. Twenty upstream-changed files remain for rereading, followed by ordinary unread source. The full audit remains active and incomplete.

{entries[0]['id']}-{entries[-1]['id']} separate actual shared image storage from broader memory claims. The decoded favicon cache converts owned pixels to Arc and face_layer clones that handle into the frame sidecar, replacing the earlier byte copy. Scene sprites now share their allocations too, but their persistent registry has a distinct lifetime and policy from the digest-keyed LRU. Host storage still owns durable bytes and decoding; a paint miss queues an ImageRef rather than fetching in the frame loop.

Four fresh checks compile the unchanged std-only cache module: two original LRU/oversize tests plus two ownership observations. Two frame handles share one allocation. Eviction to a zero-byte cache leaves a retained frame handle valid and cache usage0, so the counter is a current-entry working-set count, not all live image memory. This is expected ownership, not a leak or process-memory acceptance claim. No whole Canvas, external paint/compositor/dependency graph, image store, native renderer or GPU test was run. All generated executable/debug outputs were marker-checked, hashed and removed.

Rejected image retry policy remains a caller-trace followup: requested_images intentionally avoids repeated missing/undecodable I/O, and an oversize rejection does not itself invalidate that request marker. No permanent product image failure is asserted without tracing host invalidation/re-registration. Existing stale doc attachment/line-ceiling prose is recorded separately from functional behavior. No new bug finding is promoted in this pass.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/{NAME}) and [fresh cache checks](../../../support/receipts/authority-success-audit/shared_image_cache_probe_receipt.json). Protected sources, auxiliary runtime artifacts, prior evidence, shared WIP and index are preserved; prior document bytes remain an exact prefix. Diff check passes. No retained isolated target, Cargo home, worktree or new build output. Continue graphshell-client's changed manifest and frozen realization.
'''
doc.write_bytes(prefix+addition.encode('utf-8'));save('review_notes.json',notes);save('authority_map.json',authority)
evidence=['record_pictograph_shared_image_reread.py','pictograph_shared_image_diff_check.txt','run_shared_image_cache_probe.py','shared_image_cache_probe.rs','shared_image_cache_probe_compile.log','shared_image_cache_probe_output.log','shared_image_cache_probe_receipt.json']
budget=sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file());assert budget<100_000_000
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':head,'kind':'Current Pictograph full-source coverage restored; whole repository incomplete','prior_receipt':{'path':'upstream_cambium_reread_receipt.json','sha256':sha(HERE/'upstream_cambium_reread_receipt.json')},'coverage':stats,'completed_source_areas':areas,'whole_current_files':reread,'source_sha256':{n:sha(ROOT/n) for n in reread},'auxiliary_read_sha256':aux,'preserved_production_and_test_hashes':guards,'shared_working_file_sha256':shared,'new_findings':[],'authority_entries':[e['id'] for e in entries],'audit_authored_production_edits_this_pass':False,'source_revision_refresh':transition,'diagnostics':{'shared_image_cache_probe_receipt.json':{'build_exit':0,'run_exit':0,'checks':4,'original_tests':2,'audit_cases':2,'generated_output_removed':True}},'pending_verification':None,'next_read':next_read,'document':{'path':old['document']['path'],'sha256':sha(doc),'preserved_prefix_bytes':len(prefix),'preserved_prefix_sha256':hashlib.sha256(prefix).hexdigest()},'evidence_sha256':{n:sha(HERE/n) for n in evidence},'git_index_sha256':index_hash,'retained_isolation':[],'receipt_budget':{'cap_bytes':100_000_000,'retained_bytes_before_receipt':budget},'limits':'Only4fresh std-only private cache checks;no whole Canvas/renderer/supplier/native/GPU/store acceptance.20upstreamrereads and ordinary unread source remain.'}
save(NAME,receipt)
assert sha(index)==index_hash and doc.read_bytes().startswith(prefix)
assert all(sha(ROOT/n)==h for n,h in {**guards,**receipt['source_sha256'],**aux}.items())
assert all(sha(HERE/n)==h for n,h in receipt['evidence_sha256'].items())
assert all(sha(ROOT/n)==h for n,h in shared.items() if n!=old['document']['path'])
assert subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True).returncode==0
print(json.dumps({'receipt':NAME,'coverage':stats,'reread_files':len(reread),'reread_chars':127605,'authority_entries':receipt['authority_entries'],'protected_sources':len({**guards,**receipt['source_sha256']}),'retained_bytes':budget},indent=2))
