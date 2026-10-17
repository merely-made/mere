"""Review changed hunks while proving unchanged blocks were already fully read.

No credit from filenames, grep or a partial earlier read. Old Git bytes must
reconstruct the exact SHA-256 of a complete acknowledged file. Every non-equal
line block of its replacement must be emitted and acknowledged. New files and
unreconstructable old bytes are refused, and require the ordinary whole reader.
"""
from pathlib import Path
import argparse,difflib,hashlib,json,subprocess
HERE=Path(__file__).resolve().parent; ROOT=HERE.parents[2]
sha=lambda data:hashlib.sha256(data).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8-sig'))
def save(n,v):(HERE/n).write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8')
def plan():
    assert not (HERE/'main_delta_plan.json').exists() and load('pending_read.json')==[]
    before=load('before_main_refresh_source_inventory.json'); oldcov=load('before_main_refresh_read_coverage.json')
    inv=load('source_inventory.json'); cov=load('read_coverage.json'); refresh=load('main_refresh_after_stickleback_receipt.json')
    rows=[]
    for p in sorted(refresh['inventory_changed']):
        info=inv['files'][p]; held=cov.get(p,{})
        if held.get('sha256')==info['sha256'] and held.get('through')==info['chars']:continue
        old=before['files'][p]; assert oldcov[p]['sha256']==old['sha256'] and oldcov[p]['through']==old['chars']
        raw=subprocess.check_output(['git','show',before['commit']+':'+p],cwd=ROOT)
        forms={'git_blob':raw,'checkout_crlf':raw.replace(b'\r\n',b'\n').replace(b'\n',b'\r\n')}
        matches=[(name,data) for name,data in forms.items() if sha(data)==old['sha256']]
        assert matches, ('old bytes cannot be reconstructed; use whole reader',p)
        mode,oldbytes=matches[0]; newbytes=(ROOT/p).read_bytes(); assert sha(newbytes)==info['sha256']
        a=oldbytes.decode('utf-8-sig').splitlines(keepends=True); b=newbytes.decode('utf-8-sig').splitlines(keepends=True)
        matcher=difflib.SequenceMatcher(None,a,b,autojunk=False)
        blocks=[]; equal_chars=0; changed_chars=0
        for tag,i,j,k,l in matcher.get_opcodes():
            if tag=='equal':
                assert a[i:j]==b[k:l]; equal_chars+=len(''.join(b[k:l]))
            else:
                changed_chars+=len(''.join(b[k:l]))
                lines=[f'@@ {tag}: old lines {i+1}:{j}, new lines {k+1}:{l} @@\n']
                lines+=[' '+x for x in a[max(0,i-3):i]]
                lines+=['-'+x for x in a[i:j]]
                lines+=['+'+x for x in b[k:l]]
                lines+=[' '+x for x in b[l:min(len(b),l+3)]]
                blocks.append(''.join(lines))
        assert equal_chars+changed_chars==info['chars'] and blocks
        rows.append({'path':p,'old_sha256':old['sha256'],'old_chars':old['chars'],'new_sha256':info['sha256'],
            'new_chars':info['chars'],'old_git_commit':before['commit'],'old_bytes_reconstruction':mode,
            'unchanged_previously_read_chars':equal_chars,'changed_new_chars':changed_chars,'blocks':blocks,'acknowledged_blocks':0})
    value={'basis_commit':inv['commit'],'old_inventory_sha256':sha((HERE/'before_main_refresh_source_inventory.json').read_bytes()),
        'old_coverage_sha256':sha((HERE/'before_main_refresh_read_coverage.json').read_bytes()),'rows':rows,
        'qualification':'Current file reading is composed from a hash-proven earlier complete read of every identical block plus explicit review of every changed block. This does not claim every current file was emitted in full again. New files require whole reads.'}
    save('main_delta_plan.json',value);save('main_delta_pending.json',[])
    print(json.dumps({'files':len(rows),'unchanged_previously_read_chars':sum(r['unchanged_previously_read_chars'] for r in rows),
        'changed_new_chars':sum(r['changed_new_chars'] for r in rows),'rendered_review_chars':sum(len(b) for r in rows for b in r['blocks'])}))
def read(limit):
    assert load('main_delta_pending.json')==[] and load('pending_read.json')==[]
    p=load('main_delta_plan.json'); pending=[]; count=0
    for r in p['rows']:
        for i in range(r['acknowledged_blocks'],len(r['blocks'])):
            block=r['blocks'][i]
            if count and count+len(block)>limit:break
            print('\n=== '+r['path']+' | old '+r['old_sha256']+' | current '+r['new_sha256']+' ===\n'+block)
            pending.append({'path':r['path'],'block':i,'sha256':sha(block.encode('utf-8'))});count+=len(block)
        if count>=limit or (pending and pending[-1]['path']==r['path'] and pending[-1]['block']<len(r['blocks'])-1):break
    save('main_delta_pending.json',pending);print(json.dumps({'emitted_hunk_chars':count,'blocks':len(pending),'acknowledgment':'pending'}))
def ack():
    p=load('main_delta_plan.json'); pending=load('main_delta_pending.json');cov=load('read_coverage.json'); completed=[]
    for item in pending:
        r=next(r for r in p['rows'] if r['path']==item['path'])
        assert sha((ROOT/r['path']).read_bytes())==r['new_sha256'] and item['block']==r['acknowledged_blocks']
        assert sha(r['blocks'][item['block']].encode('utf-8'))==item['sha256'];r['acknowledged_blocks']+=1
        if r['acknowledged_blocks']==len(r['blocks']):
            cov[r['path']]={'sha256':r['new_sha256'],'through':r['new_chars'],'method':'prior_complete_read_plus_exact_changed_hunks',
                'provenance':'main_delta_plan.json','prior_full_read_sha256':r['old_sha256']}
            completed.append(r['path'])
    save('read_coverage.json',cov);save('main_delta_plan.json',p);save('main_delta_pending.json',[])
    print(json.dumps({'acknowledged_hunks':len(pending),'completed_files':completed}))
def split():
    assert load('main_delta_pending.json')==[]
    p=load('main_delta_plan.json')
    for r in p['rows']:
        if r['acknowledged_blocks']:continue
        before=''.join(r['blocks']);blocks=[]
        for block in r['blocks']:
            piece=''
            for line in block.splitlines(keepends=True):
                if piece and len(piece)+len(line)>12000:blocks.append(piece);piece=''
                piece+=line
            if piece:blocks.append(piece)
        assert ''.join(blocks)==before;r['blocks']=blocks
    save('main_delta_plan.json',p);print('Unacknowledged review blocks split losslessly at line boundaries')
if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('action',choices=['plan','read','ack','split']);parser.add_argument('--chars',type=int,default=26000);a=parser.parse_args()
    if a.action=='plan':plan()
    elif a.action=='read':read(a.chars)
    elif a.action=='ack':ack()
    else:split()
