"""Remove only the optional Wasm name custom section; retain all other bytes."""
import hashlib,json,pathlib,sys
source,target,receipt=map(pathlib.Path,sys.argv[1:])
def leb(f):
 value=shift=0
 while True:
  raw=f.read(1)
  if not raw:raise ValueError('truncated LEB')
  byte=raw[0];value|=(byte&127)<<shift
  if byte<128:return value
  shift+=7
  if shift>35:raise ValueError('oversized section length')
def sections(path):
 with path.open('rb') as f:
  assert f.read(8)==b'\0asm\1\0\0\0'
  while True:
   start=f.tell();kind=f.read(1)
   if not kind:return
   length=leb(f);payload=f.tell();name=''
   if kind==b'\0':name=f.read(leb(f)).decode('utf-8')
   end=payload+length;assert end<=path.stat().st_size
   yield start,end,kind[0],name
   f.seek(end)
def digest_region(f,start,end,dest=None):
 f.seek(start);h=hashlib.sha256()
 while start<end:
  chunk=f.read(min(1024*1024,end-start));assert chunk
  if dest:dest.write(chunk)
  h.update(chunk);start+=len(chunk)
 return h.hexdigest()
records=[]
with source.open('rb') as src,target.open('wb') as dst:
 dst.write(src.read(8))
 for start,end,kind,name in sections(source):
  keep=not(kind==0 and name=='name')
  records.append({'id':kind,'name':name,'bytes':end-start,'sha256':digest_region(src,start,end,dst if keep else None),'kept':keep})
with target.open('rb') as f:
 actual=[digest_region(f,a,b) for a,b,_,_ in sections(target)]
assert actual==[x['sha256'] for x in records if x['kept']]
assert sum(not x['kept'] for x in records)==1
with source.open('rb') as f:before=digest_region(f,0,source.stat().st_size)
with target.open('rb') as f:after=digest_region(f,0,target.stat().st_size)
receipt.write_text(json.dumps({'source_sha256':before,'output_sha256':after,'source_bytes':source.stat().st_size,'output_bytes':target.stat().st_size,'all_retained_sections_byte_identical':True,'sections':records},indent=2)+'\n')
print(receipt.read_text())
