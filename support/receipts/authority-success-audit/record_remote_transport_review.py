"""Complete Burn remote source coverage, preserving the client/session checkpoint."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,importlib.util,json,subprocess,sys
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];NAME='remote_transport_review_receipt.json'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8-sig'))
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT,text=True).strip()
assert not (HERE/NAME).exists();old=load('remote_session_review_receipt.json')
assert git('rev-parse','HEAD')==old['basis_commit'] and git('rev-parse','origin/main')==old['upstream_commit']
guards={**old['preserved_production_and_test_hashes'],**old['source_sha256']}
assert all(sha(ROOT/p)==h for p,h in {**guards,**old['auxiliary_read_sha256'],**old['runtime_input_sha256']}.items())
assert all(sha(Path(p))==h for p,h in old['external_context_sha256'].items())
assert all(sha(HERE/p)==h for p,h in old['evidence_sha256'].items())
doc=ROOT/old['document']['path'];prefix=doc.read_bytes();assert sha(doc)==old['document']['sha256']
assert all(sha(ROOT/p)==h for p,h in old['shared_working_file_sha256'].items() if p!=old['document']['path'])
index=Path(git('rev-parse','--git-path','index'))
if not index.is_absolute():index=ROOT/index
assert sha(index)==old['git_index_sha256']
i=load('source_inventory.json')['files'];c=load('read_coverage.json');assert load('pending_read.json')==[]
paths=[p for p in i if p.startswith('support/patches/burn-remote/')]
assert len(paths)==58 and sum(i[p]['chars'] for p in paths)==433101
assert all(sha(ROOT/p)==i[p]['sha256']==c[p]['sha256'] and c[p]['through']==i[p]['chars'] for p in paths)
newpaths=[p for p in paths if old.get('read_through',{}).get(p,0)<i[p]['chars']]
delta=sum(i[p]['chars']-old.get('read_through',{}).get(p,0) for p in newpaths);assert len(newpaths)==19 and delta==121654
spec=importlib.util.spec_from_file_location('audit_source_membership',HERE/'read_coverage.py');module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
explicit={'design_docs/mere_docs/receipts/natal-cap-20261017/mere-intake-narrow-final.diff','ports/graphshell/web/extension/manifest.chromium.json','ports/graphshell/web/extension/manifest.firefox.json'}
live=set()
for p in subprocess.check_output(['git','ls-files','--cached','--others','--exclude-standard','-z'],cwd=ROOT).decode().split('\0'):
 if not p or p.startswith('support/receipts/authority-success-audit/'):continue
 f=ROOT/p
 if f.is_file() and (module.is_source(f) or p in explicit):live.add(p)
assert live==set(i) and all(sha(ROOT/p)==v['sha256'] for p,v in i.items())
notes=load('review_notes.json');authority=load('authority_map.json');assert notes['findings'][-1]['id']=='Q428' and authority['entries'][-1]['id']=='A1308'
P='support/patches/burn-remote/'
rows=[
 ('Transport seam consolidates session behavior but retains concrete identity cases','Both Iroh and WebSocket server adapters invoke the same drive_session.',
  [P+'src/transport/mod.rs',P+'src/transport/link.rs',P+'src/transport/identity.rs',P+'src/transport/iroh/protocol.rs',P+'src/transport/websocket/server.rs'],
  'Complete58-file patch read confirms a single session pump, opaque frame traits and separate tensor-transfer seam. PeerId/PeerAddr distinguish stable identity/dialing hints for Iroh; WebSocket identity is its address. The transport module says the rest stays cfg-free and selection is contained here, but client/service connection enums, host, server exports/builders and error conversion still carry transport cfgs. This is a false literal consolidation claim in prose, not a second session pump.'),
 ('Iroh endpoint sharing is deliberate but not one universal node authority','Application clients use weak id-keyed APP_NODES; relay-owned clients use strong per-relay OnceCells; server constructors can own another node.',
  [P+'src/transport/iroh/node.rs',P+'src/transport/iroh/host.rs',P+'src/transport/iroh/relays.rs',P+'src/transport/iroh/server.rs'],
  'Native same-id endpoints with different bound sockets are refused by for_endpoint; browser has no socket witness and shares the first node. from_endpoint deliberately shares nothing, including server protocol construction, so connection pools are not universally unified per Endpoint. Dialed connections are pooled and closed cells removed with Arc identity check; accepted connections are not reused for outgoing streams because the accepting side alone answers streams. Owned relay nodes persist strongly for process lifetime and config is applied only at first bind. App-endpoint versus relay settings validation is explicit.'),
 ('Iroh stable identity creation has a real no-clobber file boundary','Write/sync staged key and persist_noclobber; a competing winner is loaded.',
  [P+'src/transport/iroh/identity.rs'],
  'Source uses tempfile staged in the same directory, file sync, no-clobber publication and exact32-byte load. Debug exposes public id only. Unix owner-only permissions, concurrent creators and corrupted length tests are source-read, not executed. No directory durability sync or parent creation is claimed; Windows inherits directory permissions as documented. Deterministic from_bytes seed guidance must be understood as secret seed input before product reuse.'),
 ('Protocol cancellation and owner-reclaim acknowledgement are different lifetimes','ProtocolLifetime cancels on last owner drop; targeted close awaits SessionManager teardown.',
  [P+'src/transport/iroh/protocol.rs',P+'src/transport/iroh/server.rs',P+'src/server/pump.rs',P+'src/server/session.rs'],
  'Worker-held cancellation token clones cannot keep the protocol owner guard alive. ProtocolHandler.shutdown cancels but does not itself await every detached pump; close_session is the acknowledgement path. ShutdownOnDrop starts router close only with a current Tokio runtime; Iroh port-rebind tests retry until released rather than treating abort as instant release. Pump documentation claims clean close/stream-end/shutdown drains remaining responses, while finish_session sends close and the writer biased select can abandon writing on that signal. Normal drain versus forced cancellation remains an explicit source/receipt followup.'),
 ('Cross-server transfer capability semantics differ by transport','Iroh binds full capability to authenticated target peer; WebSocket reduces it toU64 and ignores target.',
  [P+'src/transport/iroh/transfer.rs',P+'src/transport/websocket/transfer.rs',P+'src/shared/task.rs'],
  'Iroh take preserves an entry on wrong-peer refusal, enforces download count, enables Notify before map check, and bounds pending rendezvous plus TTL to300seconds. max_downloads0 still permits a first take, duplicate exposure replaces a capability and old expiry removes by key without generation check; these require caller-contract/negative controls before broader findings. WebSocket openly documents64-bit data-service keys and absence of authenticated target binding. Its fail method only logs instead of installing a terminal denial, unlike Iroh. Transport-independent TensorTransfer does not promise identical safety/liveness here; pending consumer error path needs followup.'),
 ('Framing limits and handshake limits have different admission scopes','Iroh frames have a1GiB cap while stream/init reads can wait without deadline.',
  [P+'src/transport/iroh/node.rs',P+'src/transport/iroh/protocol.rs',P+'src/client/service.rs'],
  'recv_frame narrows theU64 length to usize before comparing the cap; on32-bit wasm a larger encoded length can wrap below the cap. No wasm framing probe was run. accept reads each stream header before the next accept and without shutdown/deadline select, so one silent header can head-of-line block one connection. Client init response also has no explicit timeout. These are source-level bounded-ingress/liveness followups, not claimed tested network failures. STREAM_VERSION/ALPN remain1 while application PROTOCOL_VERSION2 is negotiated/refused deliberately, not stale protocol drift.'),
 ('Integration tests cover many migrations but do not cover independent process namespaces','Same-host concurrent transfer stress shares the same global client counter.',
  [P+'tests/iroh.rs',P+'tests/websocket.rs',P+'src/lib.rs'],
  'Read all integration source: custom/fusion source outputs/clones, Iroh roundtrip/direct transfer, owned/app endpoints, late lookup, authorizer, private relay, retry, port release, upload drops, WebSocket device moves/multiple devices/enumeration/local crossing, profile fallback and server lifetime. Q428 independent counter namespace is absent from the100-iteration two-thread same-process stress; successful stress would not refute it. Iroh within_hang_limit joins and rethrows panic; WebSocket watchdog fails on panic (though labels it Deadlock), unlike Q427 success helper. IPv6 tests return early on unavailable host capability and then appear passed, so test success alone is not IPv6 evidence. No integration test or live endpoint/relay/signal executed in this audit.')]
ids=[]
for number,(domain,claim,implementations,assessment) in enumerate(rows,1309):
 assert all((ROOT/p).exists() for p in implementations)
 ident=f'A{number}';ids.append(ident);authority['entries'].append({'id':ident,'domain':domain,'claim':claim,'implementations':implementations,'assessment':assessment,'state':'selected Burn remote entire source read; new transport notes source-only, no new execution claim','verification':NAME})
notes['review_followups'].extend([
 {'area':'Burn transport lifetime and capability differences','detail':'Iroh full/target-bound capability and bounded take versus WebSocketU64/targetless key and log-only fail. Trace terminal failure and cancellation through external data-service owner; check zero allowed downloads, duplicate exposure/old expiry generations, detached normal-close tasks and framing admission before claiming equivalent contracts.','verification':NAME},
 {'area':'Burn transport claims and evidence','detail':'Shared pump is real; cfg-free-outside-transport prose is false literally. Pool scope is per explicit node constructor, not universally per endpoint. Header/handshake deadlines and32-bit length narrowing need qualification. IPv6 source tests skip unavailable hosts and relay connect tests are not route-forced proof. All integration tests source-read only.','verification':NAME}])
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==3193 and stats['read_chars']==36368709 and not stats['changed_since_inventory'] and stats['pending_chunks']==0
notes['completed_source_areas'].append({'prefix':'support/patches/burn-remote/ remaining transport and integration test source','files':19,'chars_newly_read':delta,'total_patch_files':58,'total_patch_chars':433101,'verification':NAME,'basis_commit':old['basis_commit'],'qualification':'All58 files now complete. No additional probe/integration/network/relay run; previous12 CPU cases retain their exact limitations.'})
next_read={'path':'support/patches/cubecl-runtime/Cargo.toml','start':0,'reason':'Continue full selected CubeCL runtime source, then retained WGPU/Cubek patches and all remaining support/workflows. Full repository audit remains active.'}
pending='No audit-owned process running. Burn remote58files433101chars fully read.3193/3738 source files read;545 remain. All3738 source membership/hashes unchanged. No new binary, isolation, production edit or live integration/GPU/network run.'
notes['current_area']='Selected Burn remote source complete; selected CubeCL runtime next';notes['next_area']=next_read['reason']
notes['checkpoint'].update({'classification':'progress','classification_of_previous_turn':'progress','coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),'next_read':next_read,'pending_verification':pending,'audit_authored_production_edits_this_turn':False})
notes['checkpoint']['new_receipts'].append(NAME)
addition=f'''

### Burn remote transport and integration source completion, 2026-10-17

All58 Burn remote files are now fully read (433,101 characters). This continuation completes19 files, adding121,654 characters after the preceding partial transport checkpoint. Overall coverage is {stats['complete_files']:,}/{stats['total_files']:,} files and {stats['read_chars']:,}/{stats['total_chars']:,} characters.545 source files remain. Full live membership and all3,738 source hashes are unchanged.

A1309-A1315 distinguish real consolidation from overclaims: Iroh/WebSocket both use the shared session pump, but transport cfgs remain outside the transport module despite its cfg-free prose. Endpoint/node constructor scopes retain separate pools; owner cancellation starts shutdown while targeted close supplies backend-release acknowledgement. Iroh transfers use full capability/target checks and bounded rendezvous/expiry; WebSocket uses64-bit keys, ignores target and only logs pre-exposure failure. Frame length narrowing before bounds on32-bit, silent headers/handshakes without deadlines, duplicate capability/expiry generations and normal-close response drain remain source-qualified followups. None was promoted to a tested deployment failure.

Integration source includes direct/relay/app-owned endpoint, authorization, retry, port rebind, fusion/custom source/clone/upload-drop, multi-device/concurrent/local crossing and profile/lifetime checks. Their same-process concurrent transfer stress does not exercise Q428 independent-client counter collision. Iroh panic-propagating watchdog is stronger than Q427 helper; WebSocket panic watchdog fails but can mislabel the cause as deadlock. IPv6 tests return early on unsupported hosts, so green test status alone is not IPv6 evidence. No integration test, live relay/server/network, signal or GPU run occurred here; the earlier12 CPU witnesses retain their exact qualifications.

All prior report bytes, protected production/shared files and Git index remain unchanged except this append. Evidence remains below100MB, with no owned process, binary or isolation retained. Evidence: [checkpoint](../../../support/receipts/authority-success-audit/{NAME}). Continue selected cubecl-runtime source without narrowing the full audit denominator.
'''
doc.write_bytes(prefix+addition.encode())
for name,value in [('review_notes.json',notes),('authority_map.json',authority)]: (HERE/name).write_text(json.dumps(value,indent=2)+'\n',encoding='utf-8')
assert subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True).returncode==0
budget=sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file());assert budget<100_000_000
record={**old,'date_utc':datetime.now(timezone.utc).isoformat(),'prior_receipt':{'path':'remote_session_review_receipt.json','sha256':sha(HERE/'remote_session_review_receipt.json')},'coverage':stats,'source_sha256':{p:sha(ROOT/p) for p in paths},'preserved_production_and_test_hashes':guards,'read_through':{p:c[p]['through'] for p in paths},'new_findings':[],'authority_entries':ids,'next_read':next_read,'pending_verification':pending,
 'document':{'path':old['document']['path'],'sha256':sha(doc),'preserved_prefix_bytes':len(prefix),'preserved_prefix_sha256':hashlib.sha256(prefix).hexdigest()},'evidence_sha256':{**old['evidence_sha256'],'record_remote_transport_review.py':sha(Path(__file__))},'receipt_budget':{'cap_bytes':100_000_000,'retained_bytes_before_receipt':budget},'qualification':'Progress only. All58 Burn remote source files read. No new integration/network/GPU run or production mutation. Previous12 CPU tests scoped by their original receipt; all3738 live source hashes/membership unchanged.'}
with (HERE/NAME).open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
assert doc.read_bytes().startswith(prefix) and sha(index)==old['git_index_sha256']
assert all(sha(ROOT/p)==h for p,h in {**guards,**record['source_sha256'],**old['auxiliary_read_sha256'],**old['runtime_input_sha256']}.items())
assert all(sha(ROOT/p)==h for p,h in old['shared_working_file_sha256'].items() if p!=old['document']['path'])
print(json.dumps({'receipt':NAME,'coverage':stats,'authority_entries':ids,'budget_bytes':budget,'next_read':next_read},indent=2))
