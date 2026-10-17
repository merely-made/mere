"""One-shot full review checkpoint for page-load, framing and content-contract."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8'))
def save(n,v):(HERE/n).write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8')
def upsert(items,values,key):
 for value in values:
    at=next((i for i,x in enumerate(items) if x.get(key)==value[key]),None)
    if at is None:items.append(value)
    else:items[at]=value
old=load('fetch_review_receipt.json');probe=load('load_contract_probe_receipt.json')
assert probe['tests']==5 and probe['test_exit']==0 and probe['generated_output_removed']
suite=(HERE/'load_contract_suite.log').read_text(encoding='utf-8-sig')
for count in [5,11,10]:assert f'{count} passed; 0 failed; 0 ignored; 0 measured; 0 filtered out' in suite
assert load('pending_read.json')==[]
preserved={**old['preserved_production_and_test_hashes'],**old['source_sha256']}
assert all(sha(ROOT/p)==h for p,h in preserved.items())
assert all(sha(ROOT/p)==h for p,h in probe['source_sha256'].items())
assert all(sha(Path(v['path']))==v['sha256'] for v in probe['libraries'].values())
inventory,coverage=load('source_inventory.json')['files'],load('read_coverage.json')
prefixes=('crates/system/page-load/','crates/system/framing/','crates/system/content-contract/')
complete=sorted(p for p in inventory if p.startswith(prefixes))
assert len(complete)==8 and sum(inventory[p]['chars'] for p in complete)==88797
assert all(coverage[p]['sha256']==inventory[p]['sha256'] and coverage[p]['through']==inventory[p]['chars'] for p in complete)
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==1153 and stats['read_chars']==15407548
assert not stats['changed_since_inventory'] and not stats['pending_chunks']
notes,authority=load('review_notes.json'),load('authority_map.json')

def finding(n,title,source,evidence,scope,size,repair):
 return {'id':f'Q{n:03}','title':title,'source':source,
  'status':'reproduced through current Cargo-built public APIs; documented, not repaired',
  'evidence':evidence,'scope':scope,'verification':'load_contract_review_receipt.json','diagnostic':'load_contract_probe_receipt.json',
  'repair_triage':{'disposition':'documented; preserve production and original test source','size':size,'reason':repair}}
findings=[
finding(191,'PageLoad sends registered renderable Nex, Guppy, Titan and JSON Feed documents to download',
 'crates/system/page-load/src/lib.rs:is_download_response and PageLoad::accept_outcome; Inker default routing and Nematic engines',
 'Actual default EngineRoutePolicy plus EngineRegistry renders nonempty documents for application/x-nex, application/x-guppy, application/x-titan and application/feed+json using their registered engines. PageLoad accepts successful outcomes for the same address/MIME/body but returns Download and retains no document for all four. Text/gemini and application/gopher-menu controls remain renderable. The MIME list claims it selects types a document engine cannot render but does not consult the chosen engine or registry.',
 'Pure current library route/registry/render/load composition, no transport, host UI, browser or Titan write. Fetch source already read assigns these fixed protocol MIME types. Pelt source excerpt actually delegates terminal outcomes to PageLoad; the Pelt controller path was not executed here.',
 'shared renderability/classification owner decision can exceed one-hour diversion',
 'Let host-resolved engine capability and explicit attachment disposition determine download, or pass a resolved classification into the transport-free PageLoad. Preserve its zero-dependency role. Adding four more MIME exceptions would retain the competing authority and fail again as engines change.'),
finding(192,'A new same-address request inherits response metadata from the preceding request',
 'crates/system/page-load/src/lib.rs:begin, note_streamed, accept_outcome and acquired_at',
 'A retained response at time10 with an old redirect/MIME is followed by begin2 and a new nonstreaming body at time30. The new bytes receive time10 plus the preceding effective URL and MIME. Fresh progress at time40 updates its response URL but also retains time10. begin clears the stream but retains the document; metadata lookup keys only by address and does not distinguish the new request. Explicit forget_fetched is a working control.',
 'Public PageLoad API with synthetic timestamps and bodies. Current Pelt explicit Reload source calls forget_fetched first, so that inspected path is a real mitigating control; no bad Pelt reload, live source-capture timestamp or remote redirect was reproduced. A direct new request at the same address still violates the LoadedDocument admission-time meaning.',
 'request-bound observation correction; broader capture consumer review may exceed one hour',
 'Separate the last presented document from metadata observed for the active request. Retain old content for failure/Stop presentation while associating admission time, effective URL and MIME fallback with the current request epoch. Do not erase useful old content merely to make response identity current.'),
finding(193,'Scene transfer reuses a per-frame image key as permanent asset identity and restores old image data',
 'crates/system/content-contract/src/lib.rs:SceneTransferEncoder::encode_scene and SceneTransferDecoder::decode_scene; document-canvas paint-list image allocation; actual paint_list_render register_images',
 'Two valid document image packets lower through the real PaintList translator into scenes with the same recycled key: first is red1px, second is blue2px under another source URL. Reusing the documented per-worker encoder/decoder makes the second transfer succeed with the first red bytes and width1. A fresh-cache roundtrip preserves the blue bytes and width2. The encoder HashSet remembers only ImageKey; the decoder overwrites the new stripped scene metadata with cached dimensions/blob data. Actual producer keys restart for each list, so this is not an invented mutable immutable-blob ID.',
 'Real packet-to-paint-list-to-Scene-to-transfer/replay chain and exact resource bytes/dimensions, CPU only. No GPU upload/raster screenshot, browser worker or current product image corruption claim. Current Mere caller search finds the transfer cache definitions/tests rather than a live worker caller, so this is a public pipeline defect with consumer adoption unproven.',
 'contained descriptor-aware cache invalidation; shared asset identity migration may exceed one-hour diversion',
 'Compare the full current asset descriptor/content identity and resend or invalidate changed data, or establish a genuinely stream-wide immutable asset ID distinct from frame-local draw keys. Account for size, content revision and worker/cache reset together. Never use a per-list slot alone as proof the receiver already has this image.'),
]
upsert(notes['findings'],findings,'id')

def entry(n,domain,claim,implementations,assessment):
 return {'id':f'A{n:03}','domain':domain,'claim':claim,'implementations':implementations,
  'assessment':assessment,'state':'All8 inventoried page-load/framing/content-contract files read;26 original tests and5 generated observations pass',
  'verification':'load_contract_review_receipt.json'}
entries=[
entry(331,'Shared fetch vocabulary and sans-IO ownership','Fetch re-exports one transport-free page request/result vocabulary.',
 ['page-load owns Fetched, FetchOutcome, PageProgress, typed FetchFailure and PageLoad', 'Fetch re-exports those exact types', 'Pelt imports PageLoad and gates pending request before accept_outcome'],
 'Actual type reuse is present. PageLoad has no dependencies, transport or clock; host supplies timestamps and commands. This is a useful shared controller rather than a parallel fetch implementation. Raw-byte/source capture and decoded rendering remain distinct views.'),
entry(332,'Renderability versus protocol/engine selection','Unrenderable responses become downloads.',
 ['PageLoad hardcoded text/XML/Gopher MIME list', 'Inker format/scheme/MIME route policy', 'Nematic registered engines', 'Fetch fixed Nex/Guppy/Titan MIME shaping'],
 'Q191 proves this extra selector disagrees with four actually registered renderers. Existing binary/attachment/Gemini tests do not qualify registry-wide renderability. Preserve explicit attachment intent while moving engine capability admission to one resolved owner.'),
entry(333,'Request currentness versus retained response metadata','Each active request is exact and response metadata describes the admitted bytes.',
 ['begin resets active/stream/phase but retains document', 'accept_progress/outcome gate request ID', 'metadata fallback looks up retained document by URL only', 'Pelt explicit Reload calls forget_fetched'],
 'Exact stale-request rejection works, but Q192 shows metadata reuse crosses a request epoch at the same URL. The current Pelt Reload workaround is acknowledged. Keeping the last presented content and assigning new response provenance are different responsibilities.'),
entry(334,'Trusted state setters versus request ingress','Every transport answer is gated on the active request.',
 ['accept_progress and accept_outcome enforce the ID gate', 'note_fetched/note_streamed/fail are lower-level untagged state setters', 'surface lifecycle methods accept no request identity', 'finish/settle/stop guard current request'],
 'The broad claim is accurate for the accept ingress paths, not a capability guarantee of every public mutator. Host-owned initialization/backend feedback must be guarded by the caller. No stale current surface callback or production setter misuse was reproduced, so this is an ownership boundary qualification rather than a new bypass finding.'),
entry(335,'Exact prefixes, Stop and retained source','Streaming accumulates raw bytes before lossy UTF-8 decoding.',
 ['Complete prefix is re-decoded so split code points recover', 'Stop retires active request and keeps already presented prefix', 'Failure keeps last document', 'Final response bytes replace the prefix'],
 'Original split-UTF8/stale/Stop/conversation controls pass. Retained partial content is explicit policy, not proof acquisition completed. Each chunk clones and decodes the whole prefix; source-only cumulative work and snapshot-completeness metadata are larger follow-ups, not a measured performance or capture failure.'),
entry(336,'One postcard framing owner','F6 chose one shared magic/version helper.',
 ['Framing implements8-byte magic plus little-endian u16', 'Strict unframe checks kind and version before payload', 'Legacy helper is separately selected', 'State-witness is a real strict-framing caller in source'],
 'The small helper exists and original11 tests plus native controls pass. No local duplicate header implementation was found in these reviewed files. The governing plan explicitly says H1 landed while inventory/migration remain open; helper existence is not proof all stored/sent records use it.'),
entry(337,'Legacy framing versus schema identity','Old no-magic bytes read as version0.',
 ['split_or_legacy treats any nonmatching magic as legacy', 'unframe_or_legacy decodes positional payload at version0', 'Postcard accepts trailing bytes by explicit compatibility policy', 'Strict unframe preserves WrongSchema'],
 'Generated controls demonstrate a foreign framed blob can decode as a legacy u8 prefix and trailing bytes are accepted. Both are declared semantics, and the governing plan explicitly distinguishes strict/legacy goals. Do not call legacy admission authentication or byte-exhaustion validation; no new defect is filed for an intentional format choice.'),
entry(338,'Worker byte envelope identity','Command/update transport is one flat transferable envelope shape.',
 ['Separate private positional command and update enums', 'Both use raw postcard without framing', 'TransferBuffer is the same opaque Vec wrapper for both', 'A valid Scroll buffer also decodes as TextSelection update'],
 'The CPU collision proves direction/schema are supplied externally by the channel/caller, not self-identification. It does not prove production misdelivery or forged command authority. Data-formats F4/F6 require header adoption, and its plan still marks that migration open; record an unfulfilled planned gate rather than a false completion claim.'),
entry(339,'Worker transfer mechanics versus zero-copy wording','Transfer list moves a JS-owned buffer between browser agents.',
 ['into_array_buffer explicitly copies wasm memory to JS', 'postMessage supplies buffer in transfer list', 'from_array_buffer copies back into Vec', 'Native tests exercise byte codec only'],
 'The source correctly discloses the wasm/JS copy and does not promise all stages are zero-copy. No browser/worker execution was performed. JS detach/ownership delivery and restarted-worker cache lifetime require browser evidence before broader success claims.'),
entry(340,'Wire model duplication and positional compatibility','Wire and domain contribution shapes are intentionally decoupled.',
 ['Private mirrors for commands, fetched text, stats, links and graph contributions', 'GraphScope/ResolvedPermission reused directly', 'New ContributionV2 appended after legacy ordinal', 'Legacy edge conversion fills named lifecycle defaults'],
 'These DTO copies require lockstep maintenance but are explicit compatibility schemas, not a second durable graph authority. Current five tests preserve full V2 lifecycle/basis, multiple legacy edges, empty-versus-unknown fields and a hand-derived fixed old byte fixture. Do not remove legacy schemas just to reduce duplication.'),
entry(341,'Scene source keys versus transfer cache identity','Per-worker cache sends assets only once and reuses IDs.',
 ['Document-canvas restarts list image slots and generation', 'Paint translator deterministically flattens namespace/index to ImageKey', 'Encoder HashSet treats that key as permanent', 'Decoder restores cached dimensions and blob bytes by key'],
 'Q193 supplies the missing downstream reproduction for the earlier document producer key-reset candidate. Source asset slot, immutable blob identity, content revision and receiver cache state are different identities. Fresh-cache success does not establish repeated-frame correctness.'),
entry(342,'Two transferable rendering paths','Document packets and Scene snapshots carry their own supporting assets.',
 ['Document payload serializes a full FontTableWire', 'Scene path strips asset bytes and sends deltas through caches', 'FontTable decoder reuses FontInterner and face index', 'Scene decoder rehydrates only empty blobs'],
 'A full packet path and a cached scene path are intentional alternatives, not evidence one asset policy covers both. Font table ordering/identity depends on its producer interner. No additional font-face loss or actual GPU missing-resource failure was reproduced.'),
entry(343,'Cache delivery/error lifecycle','One encoder and decoder belong to one worker stream.',
 ['Encoder marks seen assets before serialization/post', 'Decoder stores asset records before inner Scene replay validation', 'Caches expose Default but no per-frame transaction/ack', 'No explicit eviction or stream revision field'],
 'Loss, failed post, stale/malformed inner scene, restarted worker and cache budgets need actual caller/lifecycle qualification. There is no observed production caller here or executed browser loss proof. Do not present a source-only resynchronization concern as a demonstrated live failure; Q193 is independently reproduced on an ordinary successful sequence.'),
entry(344,'Scene statistics versus transfer bytes','ContentSceneStats describes a stripped scene snapshot.',
 ['prepare_scene_snapshot replaces image/font data with empty blobs', 'encoded_bytes counts that snapshot', 'Actual assets and outer envelope are additional bytes', 'Scene stats can also be supplied by caller'],
 'These counters are not proof of total transferred payload size or cache residency. No explicit total-wire-size guarantee was found in the type; record the measurement scope instead of alleging a broken total byte counter.'),
entry(345,'Navigation/generation versus contribution/resource identity','Transport carries both render revision fields and semantic contributions.',
 ['Document/Scene/selection/stats updates include nav and viewport generation', 'Resource command carries URL/bytes without nav', 'Contribution update carries semantic graph facts without nav', 'Permissions are host-resolved DTOs in AttachScript command'],
 'Which updates may survive navigation is a caller admission/ownership decision; the byte codec does not authenticate it. No late resource application, unauthorized grant or stale graph materialization was executed. This requires actual actor/host review rather than adding speculative fields to every message.'),
entry(346,'Current gates and source coverage','Read coverage and test receipts establish bounded evidence.',
 ['3 page-load files25287 chars', '3 framing files9623 chars', '2 content-contract files53887 chars', '10+11+5 original tests and5 native observations'],
 'Three defects are reproduced; two additional observations qualify intentionally weak/externally supplied identity. Actual image data is compared after CPU replay, not rasterized pixels. Pelt, governing designs and renderer dependency excerpts are traced separately without whole-file coverage credit. Full Mere objective remains incomplete.'),
]
upsert(authority['entries'],entries,'id')
candidate=next(c for c in notes['candidates'] if c['title']=='Document producer resource and color conventions need downstream owner admission review')
candidate['downstream_followup']={'verification':'load_contract_review_receipt.json','diagnostic':'load_contract_probe_receipt.json',
 'evidence':'Q193 upgrades the per-list key concern to an actual packet/PaintList/Scene transfer replay mismatch. Reused cache restores prior bytes/dimensions; fresh-cache control preserves the new image. Invalid dimension upload and premultiplied placeholder rendering remain unproven; no GPU was run.'}
upsert(notes['candidates'],[
 {'title':'PageLoad prefix copying and retained completeness need workload/capture owner qualification',
  'source':'crates/system/page-load/src/lib.rs:note_streamed, stop and LoadedDocument',
  'note':'Each fragment clones the accumulated Vec and re-decodes its entire prefix; repeated small fragments therefore have cumulative superlinear copy/decode work. Host byte caps and live-replace cloning are separate. Retained prefixes can survive Stop/failure but LoadedDocument itself has no completeness marker. No CPU/memory benchmark or incorrectly persisted partial source capture was reproduced. Caller/capture lifecycle repair can exceed one-hour diversion.'},
 {'title':'Worker scene cache delivery, reset and bounded residency need a concrete host lifecycle',
  'source':'crates/system/content-contract/src/lib.rs:SceneTransferEncoder/Decoder and wasm post helpers',
  'note':'Encoder assumes successful delivered stream after marking asset IDs; decoder inserts assets before Scene replay; no acknowledgement, revision or eviction policy is local. Current Mere caller search found definitions/tests rather than a live transfer-cache user. No browser post failure, restarted worker or malformed-frame cache poison proof. Review the actual host before choosing reset/ack/atomicity semantics. Q193 is a separate ordinary-sequence success-path defect already reproduced.'},
 {'title':'Postcard content command/update header migration remains an open ruled gate',
  'source':'crates/system/content-contract/src/lib.rs; design_docs/2026-10-06_data_formats_brief.md:F4/F6; mere_docs/implementation_strategy/2026-10-06_postcard_framing_plan.md:H2/H3',
  'note':'Four codec calls still use raw postcard, no framing dependency/header. One legitimate Scroll command buffer also decodes as selection update under the other direction. This proves the envelope relies on external channel identity, not a production misdelivery or privilege issue. The design explicitly marks migration open rather than complete. Header adoption requires old-byte fixtures and synchronized callers; it can exceed one-hour diversion and is documented, not implemented during the read-only audit.'},
],'title')
next_path=next(p for p,v in sorted(inventory.items()) if p.startswith('crates/system/state-witness/') and coverage.get(p,{}).get('through',0)<v['chars'])
next_read={'path':next_path,'start':coverage.get(next_path,{}).get('through',0)}
notes['current_area']='Complete page-load3/3, framing3/3 and content-contract2/2 source review'
notes['next_area']='crates/system/state-witness'
notes['checkpoint']={'classification':'progress','goal_status':'active; full-source objective incomplete','coverage':stats,
 'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),'new_receipts':['load_contract_review_receipt.json'],
 'next_read':next_read,'production_edits_this_turn':False,'classification_of_previous_turn':'progress','coverage_checkpoint':stats,
 'diagnostic_outcomes':'Current combined library build succeeds;10 page-load,11 framing and5 content-contract original tests pass;5 native observations pass, including3 reproduced defects and2 qualified boundaries.',
 'user_instruction':'Audit all Mere source; preserve concurrent work; do not change production code.',
 'coverage_note':'8 complete files88797 chars added. Pelt, renderer and governing design excerpts traced without whole-file local reading credit.'}
upsert(notes['source_qualifications'],[{'area':'Complete page-load, framing and content-contract source review',
 'note':'All8 inventoried source/manifest/test files88797 chars read. Combined current library build with content-contract/scripted succeeds; original tests10+11+5 pass. Five current Cargo-built native CPU observations pass: actual default route/registry/load, request metadata, document image packet/paint-list/Scene transfer replay, cross-direction envelope and strict/legacy framing. Three new findings; intentional/external identity limits kept distinct. No network, browser worker, native UI/GPU, script execution or user persistence. Pelt Reload source mitigates same-address metadata retention by explicit forget; no bad Pelt reload claimed. Per-list image-key candidate upgraded to actual downstream replay proof; no raster image claimed. Native executable/PDB removed, stable target reused, no isolation.'}],'area')

doc=old['document']['path'];destination=ROOT/doc;before=destination.read_bytes()
assert sha(destination)==old['document']['sha256']
heading='### Read-only page-load, Framing and content-contract review, 2026-10-16'
assert heading.encode() not in before
addition='\n\n'+heading+'\n\nAll 8 inventoried files in page-load (3), Framing (3) and content-contract (2) are read, adding 88,797 characters. Repository coverage is 1,153/3,352 files and 15,407,548/36,854,820 characters. Full-source objective remains active. Production and original tests/concurrent work are unchanged. Resume at '+next_path+'. Authority entries A331-A346 distinguish actual shared state/vocabulary, competing MIME/currentness/asset identity, explicit DTO compatibility, planned header adoption and browser-only verification limits.\n\n'
for f in findings:
 addition+=f['id']+': '+f['title']+'. '+f['evidence']+'\n\n'+f['scope']+' Repair scope: '+f['repair_triage']['size']+'. '+f['repair_triage']['reason']+'\n\n'
addition+='PageLoad is a useful transport-free shared state owner and Fetch re-exports its exact vocabulary. Original stale-request, Stop, split-UTF8, retained-conversation and final-prefix tests pass. The MIME download table is nevertheless another independent parser/format decision and disagrees with actual registered renderers. Response bytes and decoded rendering are separate views; effective URL/admission-time fallback is not request-bound merely because terminal ingress uses the right ID. The public lower-level note/surface/fail methods are trusted state setters, not authenticated ingress. No stale live surface callback misuse was demonstrated. Prefix retention after Stop/failure is explicit behavior; it should not be confused with completed acquisition when a capture consumer reads LoadedDocument. Repeated prefix cloning/decoding and capture completeness need source/caller qualification, not an unmeasured performance or persistence claim.\n\n'
addition+='Framing has a real small shared helper with the chosen Wing layout. Fresh controls check magic/version before payload. Trailing bytes remain accepted and the separately selected legacy path interprets nonmatching magic as version0 raw bytes; a foreign framed blob can therefore decode as a legacy u8 prefix. Both are documented compatibility semantics, explicitly distinguished in the governing framing plan. They are not new defects or proof of schema authentication. F4/F6 require stored/sent postcard headers, but the plan accurately says only H1 is landed and H2/H3 migration is open. content-contract still uses raw positional enums and no Framing dependency. A legitimate Scroll command also decodes as a TextSelection update through the other public decoder: envelope direction/schema come from channel context, not the payload. No production misdelivery, forged authority or false completed-migration claim is asserted. Adoption needs old-byte compatibility fixtures and coordinated hosts, so it stays documented.\n\n'
addition+='The content command/update mirrors are explicitly separate wire schemas. Graph properties carry full current typed literal/statement metadata. The appended ContributionV2 and retained legacy ordinal preserve multiple edges and lifecycle/basis distinctions; fresh original tests include a hand-derived fixed old byte fixture. This is compatibility duplication requiring maintenance, not an additional durable graph authority. GraphScope and permission DTOs are reused directly. Navigation/revision fields differ by update purpose; whether resource or semantic contributions survive a navigation is a host policy not authenticated by the codec. No unauthorized grant, stale graph ingest or late resource application was executed.\n\n'
addition+='The image transfer failure upgrades the earlier document producer per-list key-reset candidate into actual downstream evidence. Each valid list is self-contained, but its resource slot is not permanent stream identity. A fresh cache roundtrips correctly while a reused per-worker cache silently restores earlier image bytes and dimensions on a successful later frame. Asset content revision, frame-local slot, immutable blob ID and receiver residency need distinct treatment. Document packets ship a full font table; Scene updates strip assets and rely on delta caches. These are different payload paths, so one cache claim cannot qualify both. Scene statistics count the stripped snapshot, not total outer/asset transfer bytes. Cache acknowledgements, malformed-frame state changes, restart/reset and residency limits remain larger source-only concerns; no live worker caller was found in current Mere search.\n\n'
addition+='Combined current Cargo library build succeeds with content-contract/scripted enabled. Original suites pass10 page-load,11 Framing and5 content-contract tests. Five native API observations pass, including three reproduced defects and two boundary qualifications. The image path is actual packet/PaintList/Scene serialization and replay with byte/dimension comparison; it is not a GPU raster screenshot. Browser wasm helpers explicitly copy between wasm and JS and then transfer the JS ArrayBuffer, so the source does not claim a copy-free entire path. No browser/worker, native UI, GPU, script, network or user persistence ran. Pelt, renderer dependency and governing format-plan excerpts are source traces, not whole-file reading credit or live host receipts. Probe executable/PDB removed; stable shared target reused, no isolated target, Cargo home or worktree. Evidence and source hashes are in the [load/contract review receipt](../../../support/receipts/authority-success-audit/load_contract_review_receipt.json).\n'
save('review_notes.json',notes);save('authority_map.json',authority)
destination.write_bytes(before+addition.encode())
diff=subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True,text=True)
(HERE/'load_contract_diff_check.txt').write_text(diff.stdout+diff.stderr,encoding='utf-8');assert diff.returncode==0
evidence=['load_contract_probe.rs','run_load_contract_probe.py','load_contract_probe_receipt.json','load_contract_probe_build.log','load_contract_probe_output.log',
 'load_contract_library_build.jsonl','load_contract_library_build_stderr.log','load_contract_suite.log','load_contract_diff_check.txt']
renderer=Path('C:/Users/mark_/.cargo/git/checkouts/netrender-a79f0762881dec01/f4f9c0d/paint_list_render/src')
traces=[ROOT/'ports/pelt/core/src/lib.rs',renderer/'lib.rs',renderer/'composite.rs',
 ROOT/'design_docs/2026-10-06_data_formats_brief.md',ROOT/'design_docs/mere_docs/implementation_strategy/2026-10-06_postcard_framing_plan.md']
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
 'kind':'Complete page-load, Framing and content-contract source reading; full Mere objective incomplete','production_edits_this_pass':False,
 'new_findings':[f['id'] for f in findings],'authority_entries':[e['id'] for e in entries],
 'coverage':stats,'added_complete_files':complete,'source_sha256':{p:sha(ROOT/p) for p in complete},
 'preserved_production_and_test_hashes':preserved,'next_read':next_read,
 'original_tests':{'page-load':10,'framing':11,'content-contract':5},'all_feature_library_test_exit':0,
 'diagnostic_tests':5,'diagnostic_exit':0,'generated_output_removed':True,'retained_isolation':[],
 'excerpt_only_traces':{str(p):sha(p) for p in traces},
 'document':{'path':doc,'sha256':sha(destination),'preserved_prefix_bytes':len(before),'preserved_prefix_sha256':hashlib.sha256(before).hexdigest()},
 'diff_check':0,'evidence_sha256':{p:sha(HERE/p) for p in evidence},
 'limits':['Full Mere objective incomplete; no production/original test changes.',
  'All8 files read; Pelt, renderer dependency and governing-plan excerpts are not whole-file local reading credit.',
  'Original10+11+5 tests and5 current Cargo-built observations pass; content-contract/scripted included in build.',
  'Three reproduced defects; cross-direction raw decoding and legacy/trailing framing behavior are separately qualified, not security findings.',
  'Image data comparison is actual CPU pipeline/replay, not raster pixels or live worker behavior; current Mere live transfer-cache consumer not established.',
  'Pelt explicit Reload calls forget_fetched, mitigating the same-address case for that inspected path; no bad host reload/capture executed.',
  'No network, browser/worker, script execution, native UI, GPU or user persistence.',
  'Helper landed and header migration is still explicitly open; no false completed-plan assertion.']}
assert all(sha(ROOT/p)==h for p,h in preserved.items())
assert all(sha(ROOT/p)==h for p,h in probe['source_sha256'].items())
save('load_contract_review_receipt.json',receipt)
print(json.dumps({'coverage':stats,'findings':len(notes['findings']),'authority_entries':len(authority['entries']),'next_read':next_read}))
