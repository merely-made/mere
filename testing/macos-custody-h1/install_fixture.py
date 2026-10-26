#!/usr/bin/env python3
# Copyright 2026 Mark Alan Boykin
# SPDX-License-Identifier: MPL-2.0
"""Narrow fixture-only installer. First copy this program to a root-owned path.

Run Python with -I. Pass the reviewed program and manifest SHA-256 from the
receiving receipt. No payload script or prepared executable runs as root.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import stat
import subprocess
import sys
import pwd
import shlex

BASE = Path('/Library/Application Support/Djinn H1 Fixtures')
MAX_FILE_BYTES = 64 * 1024 * 1024


def receiving_commands(m, manifest_hash):
    q=shlex.quote;receiver=m['receiver'];package=Path(receiver['package'])
    bootstrap=package/'receiver-tools/bootstrap_receiver.py';runner=package/'receiver-tools/run.py'
    return (
        '# Log in graphically as djinnh1 and open Terminal. Do not use sudo -u.\n'
        '# Keep this local account separate from Apple ID, signing keys and the signing account.\n'
        f'/usr/bin/python3 -I {q(str(bootstrap))} --package {q(str(package))} --manifest-sha256 {manifest_hash}\n'
        f'/usr/bin/python3 -I {q(str(runner))} run-protected --fixture {q(receiver["fixture"])} --manifest-sha256 {manifest_hash}\n\n'
        '# If registration requires approval, enable only this synthetic fixture in Login Items, then rerun the second command.\n'
        '# After a completed run, export only the public synthetic result for review.\n'
        f'/usr/bin/python3 -I {q(str(bootstrap))} --package {q(str(package))} --manifest-sha256 {manifest_hash} --export-result\n')


def verification_command(bundle, team, identifier):
    if not re.fullmatch('[A-Z0-9]{10}',team) or not re.fullmatch('[A-Za-z0-9.]+',identifier):
        raise RuntimeError('Invalid signing identity')
    requirement=f'anchor apple generic and certificate leaf[subject.OU] = "{team}" and identifier "{identifier}"'
    # codesign treats -R's argument as a filename unless a literal begins with '='.
    return ['/usr/bin/codesign','--verify','--deep','--strict','-R','='+requirement,str(bundle)]


def validate_root_path(path):
    current = Path('/')
    for part in path.parts[1:]:
        current = current / part
        s = current.lstat()
        if not stat.S_ISDIR(s.st_mode) or s.st_uid != 0 or s.st_mode & 0o022:
            raise RuntimeError('Unsafe protected parent: ' + str(current))


def directory_fd(path):
    """Resolve every component without following a symlink."""
    path = Path(path)
    if not path.is_absolute() or '..' in path.parts:
        raise RuntimeError('An absolute canonical fixture directory is required')
    fd = os.open('/', os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    try:
        for part in path.parts[1:]:
            following = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=fd)
            os.close(fd); fd = following
        return fd
    except BaseException:
        os.close(fd)
        raise


def source_bytes(root_fd, relative, uid):
    parts = PurePosixPath(relative).parts
    if not parts or any(p in ('..', '.', '/') for p in parts) or relative.startswith('/'):
        raise RuntimeError('Invalid prepared artifact path')
    fd = os.dup(root_fd)
    try:
        for part in parts[:-1]:
            child = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=fd)
            os.close(fd);fd = child
        leaf = os.open(parts[-1], os.O_RDONLY | os.O_NOFOLLOW, dir_fd=fd)
        try:
            s = os.fstat(leaf)
            if not stat.S_ISREG(s.st_mode) or s.st_uid != uid or s.st_size > MAX_FILE_BYTES:
                raise RuntimeError('Unsafe prepared artifact')
            chunks=[]; count=0
            while True:
                chunk=os.read(leaf,1024*1024)
                if not chunk:break
                count += len(chunk)
                if count > MAX_FILE_BYTES: raise RuntimeError('Oversized prepared artifact')
                chunks.append(chunk)
            return b''.join(chunks)
        finally:os.close(leaf)
    finally:os.close(fd)


def install(args):
    if os.geteuid()!=0: raise RuntimeError('Administrator authentication is required')
    # The interpreter must not execute a same-user-writable program as root.
    program=Path(__file__)
    s=program.lstat()
    if not stat.S_ISREG(s.st_mode) or s.st_uid!=0 or s.st_mode & 0o022:
        raise RuntimeError('Copy this reviewed installer to a root-owned, non-writable path first')
    validate_root_path(program.parent)
    if hashlib.sha256(program.read_bytes()).hexdigest()!=args.program_sha256:
        raise RuntimeError('Installer program digest mismatch')
    root_fd=directory_fd(args.fixture)
    staging=None
    try:
        manifest_bytes=source_bytes(root_fd,'manifest.json',args.uid)
        if hashlib.sha256(manifest_bytes).hexdigest()!=args.manifest_sha256:
            raise RuntimeError('Prepared manifest digest mismatch')
        m=json.loads(manifest_bytes)
        if m.get('profileRequired') or not m.get('profile',{}).get('group_authorized'):
            raise RuntimeError('A compatible provisioning profile is required')
        run=m['runID']
        if not re.fullmatch('[0-9a-f]{32}',run):raise RuntimeError('Invalid run identity')
        if m.get('owner_uid')!=args.uid or args.uid==0:raise RuntimeError('Fixture owner mismatch')
        receiver=m.get('receiver')
        if receiver:
            if receiver.get('user')!='djinnh1' or receiver.get('home')!='/Users/djinnh1' or receiver.get('builder_uid')!=args.uid:
                raise RuntimeError('Invalid dedicated receiver identity')
            account=pwd.getpwnam('djinnh1')
            if account.pw_uid<501 or account.pw_uid==args.uid or account.pw_dir!='/Users/djinnh1':
                raise RuntimeError('Separate receiver account is required')
            membership=subprocess.run(['/usr/bin/dsmemberutil','checkmembership','-U','djinnh1','-G','admin'],capture_output=True,text=True,check=True)
            if 'is not a member' not in membership.stdout:raise RuntimeError('Receiver must be a standard non-admin account')
        validate_root_path(BASE.parent)
        BASE.mkdir(mode=0o755,exist_ok=True)
        validate_root_path(BASE)
        destination=BASE/run
        if destination.exists():raise RuntimeError('Refusing an existing fixture installation')
        staging=BASE/('.staging-'+run)
        staging.mkdir(mode=0o700)
        expected={k:v for k,v in m['bundle_sha256'].items() if k.startswith('DjinnH1.app/')}
        if not expected:raise RuntimeError('No prepared bundle')
        for relative,expected_hash in expected.items():
            data=source_bytes(root_fd,relative,args.uid)
            if hashlib.sha256(data).hexdigest()!=expected_hash:raise RuntimeError('Artifact digest mismatch: '+relative)
            target=staging/relative
            target.parent.mkdir(parents=True,exist_ok=True,mode=0o755)
            fd=os.open(target,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
            with os.fdopen(fd,'wb') as f:f.write(data);f.flush();os.fsync(f.fileno())
            target.chmod(0o755 if relative.endswith('/Contents/MacOS/Fixture') else 0o644)
        app=staging/'DjinnH1.app'
        # Validate the exact installed role identity too, not only an arbitrary valid signature.
        for bundle,identifier in [(app,m['config']['controllerID']),
            (app/'Contents/Library/Helpers/Custodian.app',m['config']['custodianID']),
            (app/'Contents/XPCServices'/(m['config']['workerID']+'.xpc'),m['config']['workerID'])]:
            subprocess.run(verification_command(bundle,m['config']['team'],identifier),check=True)
        if receiver:
            # Public receiving inputs only. Never write to a user's home as root
            # and never execute the copied receiving tools with root authority.
            payload=staging/'ReceivingPayload';payload.mkdir(mode=0o700)
            for relative,expected_hash in m['bundle_sha256'].items():
                data=source_bytes(root_fd,relative,args.uid)
                if hashlib.sha256(data).hexdigest()!=expected_hash:raise RuntimeError('Receiving artifact digest mismatch: '+relative)
                target=payload/relative
                target.parent.mkdir(parents=True,exist_ok=True,mode=0o755)
                fd=os.open(target,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
                with os.fdopen(fd,'wb') as f:f.write(data);f.flush();os.fsync(f.fileno())
                target.chmod(0o755 if relative in m['executable_paths'] else 0o644)
            (payload/'manifest.json').write_bytes(manifest_bytes)
            (payload/'manifest.json').chmod(0o644)
            (staging/'RECEIVER-COMMANDS.txt').write_text(receiving_commands(m,args.manifest_sha256))
            (staging/'RECEIVER-COMMANDS.txt').chmod(0o644)
        # Only this private root-owned staging tree has been written. Publish atomically.
        for p in staging.rglob('*'):
            if p.is_dir():p.chmod(0o755)
        staging.chmod(0o755)
        os.rename(staging,destination);staging=None
        print(json.dumps({'installed':str(destination/'DjinnH1.app'),'root_owned':True,'payload_executed_as_root':False}))
    finally:
        os.close(root_fd)
        if staging is not None:shutil.rmtree(staging)


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--fixture',required=True);p.add_argument('--uid',type=int,required=True)
    p.add_argument('--program-sha256',required=True);p.add_argument('--manifest-sha256',required=True)
    args=p.parse_args()
    try:install(args)
    except (RuntimeError,OSError,ValueError,KeyError,subprocess.CalledProcessError) as e:
        print('Installation refused: '+str(e),file=sys.stderr);return 1
    return 0

if __name__=='__main__':sys.exit(main())
