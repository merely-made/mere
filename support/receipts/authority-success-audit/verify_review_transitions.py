"""Independently check every retained/read span and apply the emitted diff hunks."""
from pathlib import Path
import hashlib,json,subprocess
from datetime import datetime,timezone
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
sha=lambda b:hashlib.sha256(b).hexdigest()
dest=HERE/'review_transition_verification_receipt.json';assert not dest.exists()
rows=[]
for file in sorted(HERE.glob('transition_*.json')):
    c=json.loads(file.read_text(encoding='utf-8'))
    raw=(ROOT/c['path']).read_bytes();assert sha(raw)==c['current_sha256']
    git=subprocess.check_output(['git','show',c['git_basis']+':'+c['path']],cwd=ROOT)
    assert sha(git)==c['git_blob_sha256']
    candidates=[git,git.replace(b'\r\n',b'\n').replace(b'\n',b'\r\n')]
    old=next(b for b in candidates if sha(b)==c['prior_coverage']['sha256']).decode('utf-8-sig')
    new=raw.decode('utf-8-sig')
    assert c['prior_coverage']['through']==len(old) and c['current_chars']==len(new)
    spans=[]
    for mapped in c['unchanged_previously_read_spans']:
        a,b=mapped['old_chars'];x,y=mapped['new_chars']
        assert old[a:b]==new[x:y]
        assert sha(new[x:y].encode('utf-8'))==mapped['text_sha256']
        spans.append((x,y))
    before,after=old.splitlines(keepends=True),new.splitlines(keepends=True)
    emitted_new_lines=set()
    # Apply/check the emitted +/-/context text against both complete sources.
    # This is independent of the preparer's SequenceMatcher opcode creation.
    for hunk in c['emitted_hunks']:
        a,b=hunk['old_start_line']-1,hunk['new_start_line']-1
        for line in hunk['text'].splitlines(keepends=True):
            marker,text=line[0],line[1:]
            assert marker in ' +-'
            if marker in ' -':
                assert before[a]==text,(file.name,'old line',a+1)
                a+=1
            if marker in ' +':
                assert after[b]==text,(file.name,'new line',b+1)
                if marker=='+':emitted_new_lines.add(b)
                b+=1
    for changed in c['changed_spans']:
        x,y=changed['new_chars'];spans.append((x,y))
        first,last=changed['new_lines']
        assert set(range(first-1,last)).issubset(emitted_new_lines)
    cursor=0
    for x,y in sorted((x,y) for x,y in spans if y>x):
        assert x==cursor and y>=x
        cursor=y
    assert cursor==len(new),'Every current character must have exactly one source'
    rows.append({'certificate':file.name,'sha256':sha(file.read_bytes()),'path':c['path'],
        'old_fully_reviewed_chars':len(old),'current_chars':len(new),
        'unchanged_previously_read_chars':sum(y-x for x,y in [m['new_chars'] for m in c['unchanged_previously_read_spans']]),
        'new_changed_lines_inspected':len(emitted_new_lines),'hunks_verified':len(c['emitted_hunks'])})
assert len(rows)==4
assert not (HERE/'pending_transition.json').exists()
coverage=HERE/'read_coverage.json';before=sha(coverage.read_bytes())
negative=subprocess.run(['python',str(HERE/'review_transition.py'),'prepare',
    '--path','ports/distillery/probe/src/lib.rs','--base','70a153769ec5ef22b0af6a911a818518e073d946'],
    cwd=ROOT,capture_output=True,text=True)
assert negative.returncode!=0 and 'KeyError' in negative.stderr
assert sha(coverage.read_bytes())==before and not (HERE/'pending_transition.json').exists()
assert len(list(HERE.glob('transition_*.json')))==4
log=HERE/'review_transition_unread_control.log';log.write_text(negative.stdout+negative.stderr,encoding='utf-8')
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'verified_transitions':rows,
    'unread_old_source_refused_without_coverage_or_pending_write':True,
    'unread_control_exit':negative.returncode,'unread_control_log_sha256':sha(log.read_bytes()),
    'limits':'Retained coverage only: not a claim that unchanged current source was emitted again. Every old file was previously fully acknowledged at its reconstructed checkout hash; all new current characters are either text-identical mapped spans or printed and separately acknowledged insertion/replacement lines. Deleted lines were also emitted and checked.'}
dest.write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'receipt':dest.name,'transitions_verified':len(rows),
    'unchanged_reviewed_chars':sum(r['unchanged_previously_read_chars'] for r in rows),
    'changed_lines_inspected':sum(r['new_changed_lines_inspected'] for r in rows),
    'unread_source_control':'refused; coverage unchanged'},indent=2))
