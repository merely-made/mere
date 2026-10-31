from pathlib import Path
import subprocess,json,hashlib
b=Path('/Users/markik/Code');root=b/'output/context-navigation-chrome';results=[]
for app in ('woodshed','knot-editor'):
 repo=b/'repos'/app;runtime=repo/'.cargo/local/context-navigation-chrome/app-side-01';runtime.mkdir(parents=True,exist_ok=False)
 scenario=root/(app+'-app-side.scn');scenario.write_text((root/(app+'-chrome.scn')).read_text().replace('settle 1800\n',''))
 binary=b/'targets/lanes/context-navigation-chrome/artifacts/debug'/('woodshed-genet' if app=='woodshed' else 'knot')
 cmd=['/usr/bin/python3',str(root/'run_native_direct.py'),'--binary',str(binary),'--prefix','WOODSHED' if app=='woodshed' else 'KNOT','--scenario',str(scenario),'--output',str(root/(app+'-app-side-01')),'--timeout','90']
 env={'WOODSHED_DATA_DIR':str(runtime),'WOODSHED_PRESENTATION_TIMEOUT_SECS':'60'} if app=='woodshed' else {'APPDATA':str(runtime),'KNOT_THEME_LIBRARY':str(runtime/'themes.json')}
 for k,v in env.items():cmd+=['--env',k+'='+v]
 if app=='knot-editor':cmd+=['--',str(repo/'scenarios/fixtures/context_navigation.djot')]
 with (root/(app+'-app-side-01-launch.log')).open('x') as log:code=subprocess.run(cmd,stdout=log,stderr=subprocess.STDOUT,timeout=110).returncode
 results.append({'app':app,'exit':code});print(results[-1],flush=True)
(root/'app-side-results.json').write_text(json.dumps(results,indent=2)+'\n')
