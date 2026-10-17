"""Prove current coverage from already-read identical spans and newly read hunks.

This never imports coverage from an unread old file. Both full source hashes,
the Git source basis, complete line mapping, emitted hunks and explicit later
acknowledgment are retained. The original linear reading tool is unchanged.
"""
from pathlib import Path
import argparse, difflib, hashlib, json, subprocess, sys
from datetime import datetime, timezone
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
sys.stdout.reconfigure(encoding='utf-8', newline='\n')
sha=lambda b:hashlib.sha256(b).hexdigest()
def read(n):return json.loads((HERE/n).read_text(encoding='utf-8-sig'))
def save(n,v):(HERE/n).write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8')
PENDING='pending_transition.json'

def prepare(path,base):
    assert read('pending_read.json')==[], 'Finish the linear pending read first'
    assert not (HERE/PENDING).exists(), 'Inspect and acknowledge the previous transition first'
    held=read('read_coverage.json')[path]
    inventory=read('source_inventory.json')['files'][path]
    raw=(ROOT/path).read_bytes();assert sha(raw)==inventory['sha256']
    oldgit=subprocess.check_output(['git','show',base+':'+path],cwd=ROOT)
    # Retain exact Git object identity and identify the old checkout's recorded
    # bytes. Only Git's ordinary LF/CRLF checkout conversion is admitted.
    candidates=[oldgit,oldgit.replace(b'\r\n',b'\n').replace(b'\n',b'\r\n')]
    matches=[b for b in candidates if sha(b)==held['sha256']]
    assert matches, 'Prior read hash cannot be reconstructed from this Git basis; reread the whole current file'
    old=matches[0].decode('utf-8-sig');new=raw.decode('utf-8-sig')
    assert held['through']==len(old), 'Only completely reviewed old source may carry forward'
    assert held['sha256']!=inventory['sha256'], 'No changed source to review'
    a,b=old.splitlines(keepends=True),new.splitlines(keepends=True)
    def offsets(lines):
        result=[0]
        for line in lines:result.append(result[-1]+len(line))
        return result
    oa,ob=offsets(a),offsets(b)
    matcher=difflib.SequenceMatcher(a=a,b=b,autojunk=False)
    opcodes=matcher.get_opcodes();mapped=[];changed=[];hunks=[]
    for tag,i,j,k,l in opcodes:
        if tag=='equal':
            assert ''.join(a[i:j])==''.join(b[k:l])
            mapped.append({'old_chars':[oa[i],oa[j]],'new_chars':[ob[k],ob[l]],
                           'text_sha256':sha(''.join(b[k:l]).encode('utf-8'))})
        else:
            changed.append({'kind':tag,'old_lines':[i+1,j],'new_lines':[k+1,l],
                            'old_chars':[oa[i],oa[j]],'new_chars':[ob[k],ob[l]]})
    for group in matcher.get_grouped_opcodes(4):
        lines=[]
        for tag,i,j,k,l in group:
            if tag=='equal':lines.extend(' '+line for line in b[k:l])
            else:
                lines.extend('-'+line for line in a[i:j])
                lines.extend('+'+line for line in b[k:l])
        hunks.append({'old_start_line':group[0][1]+1,'new_start_line':group[0][3]+1,
                      'text':''.join(lines)})
    represented=sum(m['new_chars'][1]-m['new_chars'][0] for m in mapped)+sum(c['new_chars'][1]-c['new_chars'][0] for c in changed)
    assert represented==len(new)
    certificate='transition_'+sha(path.encode())[:12]+'_'+inventory['sha256'][:12]+'.json'
    assert not (HERE/certificate).exists()
    pending={'path':path,'git_basis':base,'git_blob_sha256':sha(oldgit),
             'prior_coverage':held,'current_sha256':sha(raw),'current_chars':len(new),
             'unchanged_previously_read_spans':mapped,'changed_spans':changed,
             'emitted_hunks':hunks,'certificate':certificate,
             'rule':'Full old source previously read at the verified hash. Equal spans proven text-identical; every insertion/replacement/deletion emitted with context and requires a separate acknowledgment. This is retained coverage, not a claim that the whole new file was emitted again.'}
    save(PENDING,pending)
    print(json.dumps({'path':path,'old_sha256':held['sha256'],'new_sha256':sha(raw),
                      'old_reviewed_chars':len(old),'new_chars':len(new),
                      'unchanged_current_chars':sum(m['new_chars'][1]-m['new_chars'][0] for m in mapped),
                      'hunks':len(hunks),'certificate':certificate}))
    for index,hunk in enumerate(hunks,1):
        print(f"\n=== HUNK {index}; old line {hunk['old_start_line']}; new line {hunk['new_start_line']} ===")
        print(hunk['text'])
    print('Transition acknowledgment pending; coverage unchanged until ack')

def acknowledge():
    assert read('pending_read.json')==[]
    p=read(PENDING);coverage=read('read_coverage.json');path=p['path']
    assert coverage[path]==p['prior_coverage']
    assert sha((ROOT/path).read_bytes())==p['current_sha256']
    assert read('source_inventory.json')['files'][path]['sha256']==p['current_sha256']
    assert not (HERE/p['certificate']).exists()
    p['acknowledged_utc']=datetime.now(timezone.utc).isoformat()
    save(p['certificate'],p)
    coverage[path]={'sha256':p['current_sha256'],'through':p['current_chars'],
                    'transition_evidence':p['certificate']}
    save('read_coverage.json',coverage)
    (HERE/PENDING).unlink()
    print(json.dumps({'acknowledged':path,'certificate':p['certificate']}))

parser=argparse.ArgumentParser();commands=parser.add_subparsers(dest='command',required=True)
prep=commands.add_parser('prepare');prep.add_argument('--path',required=True);prep.add_argument('--base',required=True)
commands.add_parser('ack');args=parser.parse_args()
if args.command=='prepare':prepare(args.path,args.base)
else:acknowledge()
