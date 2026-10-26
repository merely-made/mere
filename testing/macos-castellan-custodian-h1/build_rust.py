#!/usr/bin/env python3
# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0
"""Freeze a source-bound custodian static library; never operates on a vault."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess

ROOT=Path(__file__).resolve().parents[2]

def source_hashes():
    paths=[ROOT/'Cargo.toml',ROOT/'Cargo.lock',ROOT/'rust-toolchain.toml']
    for directory in ['ports/djinn/castellan','ports/djinn/custodian']:
        paths.extend(p for p in (ROOT/directory).rglob('*') if p.is_file() and p.suffix in ['.rs','.toml','.h'])
    return {str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(paths)}

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--target-dir',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args()
    before=source_hashes();output=args.output.resolve();output.mkdir(parents=True,exist_ok=False)
    command=['cargo','build','--offline','--locked','-p','djinn-custodian','--features','native-host','--lib']
    env={**os.environ,'CARGO_TARGET_DIR':str(args.target_dir.resolve())}
    with (output/'build.log').open('w') as log:
        subprocess.run(command,cwd=ROOT,env=env,stdout=log,stderr=subprocess.STDOUT,check=True)
    after=source_hashes()
    if after!=before:raise RuntimeError('Custody sources changed during compilation; build not qualified')
    library=args.target_dir.resolve()/'debug/libdjinn_custodian.a'
    shutil.copyfile(library,output/library.name)
    record={'schema':1,'source_root':str(ROOT),'source_sha256':after,
        'command':command,'features':['native-host'],'library':str(output/library.name),
        'library_sha256':hashlib.sha256((output/library.name).read_bytes()).hexdigest(),
        'rustc':subprocess.check_output(['rustc','-Vv'],cwd=ROOT,text=True),
        'cargo':subprocess.check_output(['cargo','-V'],cwd=ROOT,text=True),'real_consumers_allowed':False}
    (output/'build.json').write_text(json.dumps(record,indent=2)+'\n')
    print(json.dumps({'build':str(output/'build.json'),'library_sha256':record['library_sha256']}))

if __name__=='__main__':main()
