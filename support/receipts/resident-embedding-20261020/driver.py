from pathlib import Path
import hashlib,json,os,subprocess,sys,time,re
OUT=Path(__file__).parent
artifacts=json.loads((OUT/'model-artifacts.json').read_text())
for name,expected in artifacts['artifacts'].items():
 data=(OUT/'all-MiniLM-L6-v2'/name).read_bytes()
 assert len(data)==expected['bytes'] and hashlib.sha256(data).hexdigest()==expected['sha256'],name
run=sys.argv[1]
assert re.fullmatch(r'[a-z0-9-]+',run)
root=OUT/('runtime-'+run);root.mkdir(exist_ok=False)
(root/'fixture-marker').write_text('embedding-envelope-20261020\n')
tmp=Path('/tmp/ee20261020');tmp.mkdir(exist_ok=True)
marker=tmp/'embedding-fixture-marker'
if marker.exists():assert marker.read_text()=='embedding-envelope-20261020\n'
else:marker.write_text('embedding-envelope-20261020\n')
executable=Path(sys.argv[2])
sha=hashlib.sha256(executable.read_bytes()).hexdigest()
results={}
for phase in ['prepare','cold','foreign']:
 env=os.environ.copy();env.update(EMBEDDING_RECALL_ROOT=str(root),EMBEDDING_RECALL_PHASE=phase,ESP_MINILM_DIR=str(OUT/'all-MiniLM-L6-v2'),TMPDIR=str(tmp)+'/')
 log=OUT/(run+'-'+phase+'.log');start=time.monotonic()
 with log.open('w') as f:
  result=subprocess.run(['/usr/bin/time','-l',str(executable),'--ignored','--exact','resident_embedding_process_fixture','--nocapture'],env=env,stdout=f,stderr=subprocess.STDOUT,timeout=360)
 results[phase]={'exit_code':result.returncode,'elapsed_seconds':round(time.monotonic()-start,3),'binary_sha256':sha,'log':log.name}
 (OUT/(run+'-processes.json')).write_text(json.dumps(results,indent=2)+'\n')
 if result.returncode:raise RuntimeError(f'{phase} failed; inspect {log}')
 receipt=json.loads((root/(phase+'-receipt.json')).read_text());results[phase]['pid']=receipt['pid']
 rss=re.search(r'(\d+)\s+maximum resident set size',log.read_text())
 if rss:results[phase]['maximum_resident_set_bytes']=int(rss[1])
 print(phase,results[phase],flush=True)
 if phase=='prepare':
  origin=root/'origin-corpus.json';assert origin.exists();digest=hashlib.sha256(origin.read_bytes()).hexdigest();origin.unlink()
  (OUT/(run+'-source-removal.json')).write_text(json.dumps({'origin_corpus_sha256':digest,'origin_path':str(origin),'absent_before_cold':not origin.exists(),'retained_store':str(root/'retained.redb')},indent=2)+'\n')
 (OUT/(run+'-processes.json')).write_text(json.dumps(results,indent=2)+'\n')
assert len({r['pid'] for r in results.values()})==3
prepared=json.loads((root/'prepare-receipt.json').read_text());cold=json.loads((root/'cold-receipt.json').read_text());foreign=json.loads((root/'foreign-receipt.json').read_text())
assert len(prepared['rows'])==len(cold['rows'])==12
metrics={}
for category in ['all','paraphrase','lexical_control']:
 rows=[r for r in prepared['rows'] if category=='all' or r['query']['kind']==category]
 metrics[category]={}
 for lane in ['lexical','vector','fused']:
  ranks=[r['ranks'][lane] for r in rows]
  metrics[category][lane]={'queries':len(rows),'top1':sum(rank==1 for rank in ranks),'recall_at_3':sum(rank is not None and rank<=3 for rank in ranks),'mrr':sum(0 if rank is None else 1/rank for rank in ranks)/len(rows)}
for a,b in zip(prepared['rows'],cold['rows']):
 assert a['rankings']==b['rankings'] and a['passages']==b['passages']
 assert a['embedding']['index']==b['embedding']['index'] and a['embedding']['envelope']==b['embedding']['envelope']
assert len(foreign['refusals'])==4
# Bounded fixture-store scan; never copy raw stores or custody material into Git.
needles=[b'"vectors"',b'"producer"',b'foreign/v1',b'https://retained.test/']
stores=list((root/'data').rglob('inference.redb'))
assert len(stores)==1,stores
store=stores[0]; raw=store.read_bytes()
store_scan={'path_relative_to_fixture':str(store.relative_to(root)),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'needles':{n.decode():raw.count(n) for n in needles}}
assert not any(store_scan['needles'].values()),store_scan
summary={'passed':True,'private_journal_scan':store_scan,'run':run,'processes':results,'metrics':metrics,'matching_cold_index_and_passages':True,'foreign_refusals':foreign['refusals'],'source_corpus_removed':not (root/'origin-corpus.json').exists(),'executable_sha256':sha,'limits':['Real CPU MiniLM embeddings; a tiny tensor-decoder fixture supplies the durable exact ask session; no trained generative-answer quality claim','Native resident/application IPC; no headed UI or second-device gate','Frozen synthetic retained corpus and fixed fusion weights; benchmark is bounded to these queries']}
(OUT/(run+'-qualification.json')).write_text(json.dumps(summary,indent=2)+'\n')
print(json.dumps(metrics,indent=2))
