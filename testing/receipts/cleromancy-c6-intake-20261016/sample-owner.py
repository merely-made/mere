import subprocess,time,pathlib,os
b=pathlib.Path(__file__).parent
exe='/Users/markik/Code/repos/mere/target/build/debug/deps/embedded_reservoir_validation-51b09f2b4d40bfba'
with (b/'owner-cold-sampled.log').open('w') as f:
 p=subprocess.Popen([exe,'recorded_situation_and_weather_survive_three_owners_and_refuse_tamper','--nocapture','--test-threads=1'],stdout=f,stderr=subprocess.STDOUT)
 captured=False
 while p.poll() is None:
  s=(b/'owner-cold-sampled.log').read_text()
  if not captured and 'RELEASED mode=embedded' in s:
   ps=subprocess.check_output(['ps','-axo','pid,ppid,comm'],text=True)
   children=[int(x.split()[0]) for x in ps.splitlines() if len(x.split())>=3 and x.split()[1]==str(p.pid)]
   if children:
    with (b/'owner-sample-command.log').open('w') as out:
     subprocess.run(['/usr/bin/sample',str(children[-1]),'1','-file',str(b/'owner-cold-sample.txt')],stdout=out,stderr=subprocess.STDOUT)
    captured=True
  time.sleep(.05)
 print('exit',p.returncode,'sampled',captured)
