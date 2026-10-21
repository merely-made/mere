# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

import hashlib,io,json,pathlib,tarfile,urllib.request,datetime,subprocess,sys
repo=pathlib.Path(__file__).resolve().parents[3]
base=pathlib.Path(sys.argv[1]) if len(sys.argv)>1 else pathlib.Path(__file__).parent
base.mkdir(parents=True,exist_ok=True)
def committed(revision,relative):
 return subprocess.check_output(["git","show",revision+":"+relative],cwd=repo)
pairs=[('hest','0.1.0','crates/cambium/hest-0_1','561188d5aaf5b98c5e18a845c56163f41a45ff23'),('command-menu','0.1.1','crates/cambium/command-menu','561188d5aaf5b98c5e18a845c56163f41a45ff23'),('hest','0.2.0','crates/cambium/hest','561188d5aaf5b98c5e18a845c56163f41a45ff23'),('cambium','0.5.0','crates/cambium/cambium','615b35e2e2af5436825d154eea52ae6c167cbfb3')]
def fetch(url):
 return urllib.request.urlopen(urllib.request.Request(url,headers={'User-Agent':'mere-hest-release/0.1'}),timeout=45).read()
results=[]
for name,version,relative,revision in pairs:
 api='https://crates.io/api/v1/crates/'+name+'/'+version
 data=json.loads(fetch(api))['version']
 assert not data['yanked']
 archive_url='https://static.crates.io/crates/'+name+'/'+name+'-'+version+'.crate'
 archive=fetch(archive_url)
 checksum=hashlib.sha256(archive).hexdigest()
 assert checksum==data['checksum']
 (base/(name+'-'+version+'.crate')).write_bytes(archive)
 files=[]
 with tarfile.open(fileobj=io.BytesIO(archive),mode='r:gz') as tar:
  prefix=name+'-'+version+'/'
  vcs=json.load(tar.extractfile(prefix+'.cargo_vcs_info.json'))
  assert vcs['git']['sha1']==revision,(name,vcs)
  assert not vcs['git'].get('dirty',False)
  assert vcs['path_in_vcs']==relative
  assert tar.extractfile(prefix+'Cargo.toml.orig').read()==committed(revision,relative+'/Cargo.toml')
  for entry in tar.getmembers():
   local=entry.name.removeprefix(prefix) if hasattr(str,'removeprefix') else entry.name[len(prefix):]
   if entry.isfile() and (local.startswith('src/') or local.startswith('tests/')):
    raw=tar.extractfile(entry).read()
    assert raw==committed(revision,relative+'/'+local),(name,local)
    files.append({'path':local,'sha256':hashlib.sha256(raw).hexdigest()})
 result={'name':name,'version':version,'api_url':api,'archive_url':archive_url,'checksum':checksum,'yanked':data['yanked'],'created_at':data['created_at'],'rust_version':data['rust_version'],'vcs':vcs,'source_files_match_release_tree':files}
 results.append(result)
 print(name,version,'verified',len(files),'source/test files',checksum)
(base/'registry-publication-proof.json').write_text(json.dumps({'verified_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'releases':results},indent=2)+'\n')
