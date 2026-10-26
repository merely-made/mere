#!/usr/bin/env python3
# Copyright 2026 Mark Alan Boykin
# SPDX-License-Identifier: MPL-2.0
"""Unprivileged bootstrap/export for the dedicated synthetic receiving account."""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import pwd
import shutil
import stat
import subprocess
import sys


def sha(data):return hashlib.sha256(data).hexdigest()


def artifact_parts(relative):
    parts=PurePosixPath(relative).parts
    if not parts or relative.startswith('/') or any(p in ('..','.','/') for p in parts):
        raise RuntimeError('Invalid receiving artifact path')
    return parts


def protected_path(path, *, directory=False):
    """Check each real root-owned component, including the leaf."""
    if not path.is_absolute() or '..' in path.parts:raise RuntimeError('Noncanonical protected path')
    current=Path('/')
    for part in path.parts[1:]:
        current=current/part;s=current.lstat()
        if stat.S_ISLNK(s.st_mode) or s.st_uid!=0 or s.st_mode&0o022:
            raise RuntimeError('Unprotected receiving package: '+str(current))
        if current!=path or directory:
            if not stat.S_ISDIR(s.st_mode):raise RuntimeError('Protected parent is not a directory')
        elif not stat.S_ISREG(s.st_mode):raise RuntimeError('Protected artifact is not regular')


def validate_package(package, expected):
    protected_path(package,directory=True)
    p=package/'manifest.json';protected_path(p);data=p.read_bytes()
    if sha(data)!=expected:raise RuntimeError('Reviewed manifest digest mismatch')
    m=json.loads(data)
    if m.get('receiver',{}).get('package')!=str(package):raise RuntimeError('Receiving package path mismatch')
    for relative,digest in m['bundle_sha256'].items():
        p=package.joinpath(*artifact_parts(relative));protected_path(p)
        if sha(p.read_bytes())!=digest:raise RuntimeError('Protected receiving artifact changed: '+relative)
    return m


def receiving_context(m):
    account=pwd.getpwuid(os.getuid());receiver=m['receiver']
    membership=subprocess.run(['/usr/bin/dsmemberutil','checkmembership','-U',account.pw_name,'-G','admin'],capture_output=True,text=True)
    gui=subprocess.run(['/bin/launchctl','print','gui/'+str(os.getuid())],capture_output=True,text=True)
    positive=receiver.get('signer_positive_control',{})
    signer_control=(positive.get('signing',{}).get('returncode')==0 and positive.get('verification',{}).get('returncode')==0
                    and positive.get('unsigned_binary_sha256')==m['bundle_sha256'].get('AttackerMintedCustodian.app/Contents/MacOS/Fixture'))
    result={'uid':os.getuid(),'user':account.pw_name,'home':account.pw_dir,'builder_uid':receiver['builder_uid'],
        'differentUID':os.getuid()>=501 and os.getuid()!=receiver['builder_uid'],
        'intendedAccount':account.pw_name==receiver['user'] and account.pw_dir==receiver['home'],
        'homeEnvironment':os.environ.get('HOME')==receiver['home'],
        'notSudoSession':not any(os.environ.get(k) for k in ['SUDO_UID','SUDO_USER']),
        'nonAdmin':membership.returncode==0 and 'is not a member' in membership.stdout,
        'membershipResult':membership.stdout.strip(),'guiSession':gui.returncode==0,
        'signerPositiveControl':signer_control}
    result['valid']=all(result[n] for n in ['differentUID','intendedAccount','homeEnvironment','notSudoSession','nonAdmin','guiSession','signerPositiveControl'])
    return result


def bootstrap(package,m):
    context=receiving_context(m)
    if not context['valid']:raise RuntimeError('Separate-account prerequisites failed: '+json.dumps(context))
    receiver=m['receiver'];destination=Path(receiver['fixture'])
    expected=Path(receiver['home'])/'Library/Application Support/Djinn H1 Fixtures'/m['runID']
    if destination!=expected:raise RuntimeError('Receiving destination mismatch')
    # This program always runs as the standard receiver, never with root rights.
    # Refuse redirection through a preexisting symlink or an existing run folder.
    current=Path(receiver['home'])
    for part in ['Library','Application Support','Djinn H1 Fixtures']:
        current=current/part
        if not current.exists():current.mkdir(mode=0o700)
        s=current.lstat()
        if not stat.S_ISDIR(s.st_mode) or s.st_uid!=os.getuid():raise RuntimeError('Unsafe receiving parent')
    destination.mkdir(mode=0o700)
    try:
        for relative,digest in m['bundle_sha256'].items():
            source=package.joinpath(*artifact_parts(relative));data=source.read_bytes()
            if sha(data)!=digest:raise RuntimeError('Receiving artifact changed')
            target=destination/relative;target.parent.mkdir(parents=True,exist_ok=True,mode=0o700)
            with target.open('xb') as f:f.write(data)
            target.chmod(0o700 if relative in m['executable_paths'] else 0o600)
        (destination/'manifest.json').write_bytes((package/'manifest.json').read_bytes())
        (destination/'vault-symlink').symlink_to(destination/'synthetic-state/custody/canary')
        (destination/'receiving-context.json').write_text(json.dumps(context,indent=2)+'\n')
    except BaseException:
        shutil.rmtree(destination);raise
    print(json.dumps({'fixture':str(destination),'context':context,'production_migration':False}))


def export_result(m):
    context=receiving_context(m)
    if not context['valid']:raise RuntimeError('Wrong receiving account/session')
    source=Path(m['receiver']['fixture'])/'protected-results.json'
    result=json.loads(source.read_text())
    if not result.get('syntheticOnly') or result.get('uid')!=os.getuid():raise RuntimeError('Not this receiver synthetic result')
    target=Path('/Users/Shared')/('djinn-h1-'+m['runID']+'-results.json')
    # Exact public result only; never copy state, a Keychain, or a private seed.
    fd=os.open(target,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o644)
    with os.fdopen(fd,'w') as f:json.dump(result,f,indent=2);f.write('\n')
    target.chmod(0o644)
    print(json.dumps({'exported':str(target),'sha256':sha(target.read_bytes()),'H1_production_migration_allowed':False}))


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--package',type=Path,required=True);p.add_argument('--manifest-sha256',required=True)
    p.add_argument('--export-result',action='store_true');args=p.parse_args()
    if os.geteuid()==0:raise RuntimeError('Receiving tools must never run as root')
    m=validate_package(args.package,args.manifest_sha256)
    if args.export_result:export_result(m)
    else:bootstrap(args.package,m)


if __name__=='__main__':
    try:main()
    except (RuntimeError,OSError,ValueError,KeyError) as e:
        print('Receiving setup refused: '+str(e),file=sys.stderr);sys.exit(1)
