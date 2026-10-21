#!/usr/bin/env python3
"""Verify the ordinary-view co-op packet and its committed source inputs."""
import argparse,hashlib,json,struct,subprocess
from pathlib import Path
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[4]
def read(path):return json.loads(path.read_text())
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def git_files(files,commit):
    proc=subprocess.Popen(['git','cat-file','--batch'],cwd=ROOT,stdin=subprocess.PIPE,stdout=subprocess.PIPE)
    try:
        for name,expected in files.items():
            proc.stdin.write((commit+':'+name+'\n').encode());proc.stdin.flush()
            header=proc.stdout.readline().decode().split();assert len(header)==3 and header[1]=='blob',name
            blob=proc.stdout.read(int(header[2]));assert proc.stdout.read(1)==b'\n'
            assert hashlib.sha256(blob).hexdigest()==expected,name
    finally:proc.stdin.close();proc.wait(timeout=10)
    assert proc.returncode==0

def main():
    global HERE
    parser=argparse.ArgumentParser();parser.add_argument('--commit');parser.add_argument('--packet',type=Path,default=HERE);args=parser.parse_args()
    HERE=args.packet.resolve()
    manifest=read(HERE/'artifact-sha256.json')
    for name,expected in manifest.items():
        path=(HERE/name).resolve();assert path.is_relative_to(HERE);assert sha(path)==expected,name
    inputs=read(HERE/'build-inputs.json')
    if args.commit:
        git_files(inputs['files'],args.commit);git_files(inputs['harness'],args.commit)
    else:
        for name,expected in (inputs['files']|inputs['harness']).items():assert sha(ROOT/name)==expected,name
    evidence=HERE/'accepted';q=read(evidence/'qualification.json');assert q['result']=='ok'
    assert q['all_owned_processes_stopped'] and q['custodian_launches']==4
    assert len(set(q['native_processes']))==7
    custodian_pids=[]
    for role in ['owner','guest']:
        previous_stop=None
        for launch in [1,2]:
            events=[json.loads(line) for line in (evidence/(role+'-resident')/('events-'+str(launch)+'.jsonl')).read_text().splitlines()]
            ready=next(event for event in events if event['event']=='ready')
            stopped=next(event for event in events if event['event']=='stopped')
            assert stopped['ok'] and stopped['reason']=='stop intent'
            assert ready['pid']==stopped['pid'] and ready['at_ms']<stopped['at_ms']
            if previous_stop is not None:assert previous_stop<ready['at_ms']
            previous_stop=stopped['at_ms'];custodian_pids.append(ready['pid'])
    assert len(set(custodian_pids))==4
    for key,name in [('binary_sha256','graphshell_mere'),('djinn_sha256','djinn'),('prepare_sha256','coop_existing_graph')]:assert q[key]==inputs['binaries'][name]
    assert q['original_invitation_sha256']==q['cold_invitation_sha256']
    chosen,private=q['graphs']['chosen'],q['graphs']['private'];assert chosen!=private
    captures={}
    for name in q['phases']:
        result=read(evidence/(name+'.json'));assert result['ok'] and not result['errors'],name
        assert int(result['final']['pid']) in q['native_processes']
        launch=read(evidence/(name+'-launch.json'));assert launch['binary_sha256']==q['binary_sha256']
        for capture in result['captures']:
            path=evidence/Path(capture['path']).name
            data=path.read_bytes();assert data[:8]==b'\x89PNG\r\n\x1a\n'
            assert struct.unpack('>II',data[16:24])==(capture['width'],capture['height'])
            assert capture['width']>200 and capture['height']>200
            fields=capture['fields'];captures[path.stem]=fields
            update=json.loads(fields['coop']);state=update['data']
            if name.startswith('guest'):
                assert fields['sessions']=='0' and fields['archives']=='0'
                if state:
                    assert state['graph']==chosen
                    assert all(n['address'].startswith('https://shared.invalid/') for n in state['nodes'])
                    ids={n['id'] for n in state['nodes']}
                    assert all(r['from'] in ids and r['to'] in ids for r in state['relations'])
    assert len(captures)==21,len(captures)
    def fields(name):return captures[name]
    def state(name):return json.loads(fields(name)['coop'])['data']
    assert fields('owner-private-during-coop')['attached']==private
    assert state('owner-private-during-coop') is None
    assert 'private.invalid' in fields('owner-private-during-coop')['graph-labels']
    assert state('guest-still-scoped')['graph']==chosen and len(state('guest-still-scoped')['nodes'])==2
    assert state('owner-returned-to-chosen')['graph']==chosen
    assert fields('guest-push')['relations']=='1'
    assert len(state('guest-push')['relations'])==1
    warm=state('guest-return');cold=state('owner-cold-retained')
    assert warm['owner']!=warm['guest']
    existing=q['existing_surface']
    assert any(n['id']==existing and n['title']=='Guest revised existing source' for n in warm['nodes'])
    assert any(a['id']=='personae:'+warm['guest'] and a['via']=='graphshell.coop' for a in warm['authors'])
    assert cold['graph']==chosen and cold['owner']==warm['owner'] and cold['guest']==warm['guest']
    assert cold['access']=='revoked' and cold['epoch']>warm['epoch']
    for key in ['nodes','relations','authors','revision']:assert cold[key]==warm[key],key
    final=state('owner-cold-edited');assert len(final['nodes'])==4
    assert final['authors'][-1]['id']=='personae:'+final['owner']
    assert 'Guest revised existing source' in fields('owner-cold-retained')['graph-labels']
    assert state('owner-revoked')['access']=='revoked'
    for name in ['guest-withdrawn','guest-cold-denied']:
        assert state(name) is None and fields(name)['nodes']=='0' and fields(name)['selected']==''
        assert fields(name)['record']=='' and fields(name)['record-data']==''
    assert 'Revoked' in fields('guest-cold-denied')['coop-status']
    before=read(evidence/'private-before.json');after=read(evidence/'private-after.json')
    for key in ['session','graph','facets']:assert before['graphs'][0][key]==after['graphs'][0][key],key
    def changes(record):return next(v['value'] for c in record['cards'] for v in c['values'] if v['label']=='Changes')
    assert changes(before)==changes(after)
    review=read(HERE/'capture-review.json');assert {r['capture'] for r in review}=={name+'.png' for name in captures}
    assert all(r['reviewed'] and r['observation'] and sha(evidence/r['capture'])==r['sha256'] for r in review)
    print(json.dumps({'result':'ok','source_files':len(inputs['files']),'public_artifacts':len(manifest),'headed_captures':len(captures),'native_processes':7,'commit':args.commit},indent=2))
if __name__=='__main__':main()
