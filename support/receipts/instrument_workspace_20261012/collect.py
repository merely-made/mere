# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

"""Preserve actual Instrument captures and source-bound qualification receipts."""
from pathlib import Path
import hashlib
import json
import os
import re
import shutil
import subprocess
from PIL import Image

repo = Path(__file__).resolve().parents[3]
out = Path(__file__).parent
base = Path(os.environ.get('INSTRUMENT_PACKAGE', '/Users/markik/Code/output/instrument-workspace-20261012'))
for label in ['browser-r1', 'browser-r2', 'browser-r3', 'browser-r4', 'browser-r5', 'browser-r6',
              'native-instrument', 'native-instrument-r2', 'native-usability', 'native-instrument-r3', 'native-usability-r2', 'native-instrument-r4', 'native-usability-r3']:
    shutil.copytree(base / label, out / label, dirs_exist_ok=True)
for name in ['candidate.json', 'candidate-r2.json', 'candidate-r3.json', 'candidate-r4.json', 'candidate-r5.json', 'native-candidate.json',
             'native-candidate-r1.json', 'native-candidate-r2.json', 'native-candidate-r3.json', 'native-commands.json', 'native-commands-r1.json', 'native-commands-r2.json', 'native-commands-r3.json',
             'native-release.json', 'native-release-r1.json', 'native-release-r2.json', 'native-release-r3.json']:
    shutil.copy2(base / name, out / name)
for name in ['browser-r6-launch.log', 'native-instrument-r4-launch.log', 'native-usability-r3-launch.log']:
    shutil.copy2(base / name, out / name)
browser = json.loads((out / 'browser-r6/summary.json').read_text())
assert browser['headed'] and browser['page_errors'] == 0 and len(browser['runs']) == 8
assert browser['adapter']['vendor'] == 'amd' and browser['adapter']['architecture'] == 'gcn-5'
assert json.loads((out / 'browser-r6/release.json').read_text())['browser_closed']
native = []
for label in ['native-instrument-r4', 'native-usability-r3']:
    process = json.loads((out / label / 'process.json').read_text())
    receipt = (out / label / 'scenario.done').read_text()
    assert process['success'] and not process['timeout'] and receipt.startswith('RESULT ok\n')
    frames = re.search(r'frames: (\d+) captured, (\d+) blank, (\d+) distinct', receipt)
    presentation = re.search(r'presentation: (\d+) frames, (\d+) redraws, (\d+) unpresented', receipt)
    assert frames and int(frames[2]) == 0 and presentation and int(presentation[1]) > 0
    native.append({'label': label, 'process': process, 'captured': int(frames[1]),
                   'blank': int(frames[2]), 'presented': int(presentation[1]),
                   'redraws': int(presentation[2]), 'unpresented_redraws': int(presentation[3])})
artifacts = {}
for directory in ['browser-r6', 'native-instrument-r4', 'native-usability-r3']:
    for image in sorted((out / directory).glob('*.png')):
        with Image.open(image) as pixels:
            colors = pixels.convert('RGB').getcolors(pixels.width * pixels.height)
            assert colors and len(colors) >= 128, image
            artifacts[str(image.relative_to(out))] = {'sha256': hashlib.sha256(image.read_bytes()).hexdigest(),
                'width': pixels.width, 'height': pixels.height, 'unique_rgb': len(colors)}
with Image.open(out / 'native-instrument-r4/instrument_light.png') as light, Image.open(out / 'native-instrument-r4/instrument_dark.png') as dark:
    # A background pixel in the neutral titlebar stays put; a reader content
    # pixel changes with the actual draft specimen mode.
    frame = light.getpixel((20, 20)); assert frame == dark.getpixel((20, 20))
    reader_light = light.getpixel((1700, 500)); reader_dark = dark.getpixel((1700, 500))
    assert reader_light != reader_dark
native_manifest = json.loads((out / 'native-candidate.json').read_text())
source_changes = [p for p,h in native_manifest['source_sha256'].items()
                  if hashlib.sha256((repo / p).read_bytes()).hexdigest() != h]
assert not source_changes, source_changes
package = json.loads((out / 'candidate.json').read_text())
for p,h in package['source_sha256'].items():
    assert hashlib.sha256((repo / p).read_bytes()).hexdigest() == h, p
qualification = {
    'scope': 'Instrument shared presentation and actual Graphshell local-mere / Tabard workshop pilot',
    'compiler': 'rustc 1.97.1', 'build_source': native_manifest['build_source'],
    'browser': browser, 'native_source': native_manifest['source'],
    'native_binary_sha256': native_manifest['binary_sha256'], 'native': native,
    'runtime_input_binding': 'Browser and native source hashes unchanged after the final capture' ,
    'post_capture_native_source_changes': source_changes,
    'current_source': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=repo, text=True).strip(),
    'frame_independence_pixels': {'coordinate': [20,20], 'light_and_dark_frame': frame,
        'reader_coordinate': [1700,500], 'reader_light': reader_light, 'reader_dark': reader_dark},
    'artifacts': artifacts,
    'negative_receipts': {
        'browser-r1': 'Driver awaited visibility of inert semantic heading; closed without product acceptance',
        'browser-r2': 'Driver tried opening an already-open workbench; capture revealed missing host fallback font and was rejected',
        'browser-r3': 'Eight cases passed, but visual review found legacy floating controls covering the scene caption; layout renewed in r4',
        'native-instrument-r2 / native-usability': 'Scenarios passed but final baseline comparison rejected neutral rectangles over transparent graph targets. Revised utility-region control styling, CPU negative control, fresh builds and captures required',
        'native-instrument-r3 / native-usability-r2': 'Domain graph targets restored but primary Save accent lost to utility selector specificity; primary variant precedence renewed with a negative CPU control and final capture',
        'native-instrument': 'Fresh theme is unsaved; initial clean-state assertion was incorrect. Seven nonblank captures, no native acceptance. Corrected via real Save in r2',
        'cpu/negative/neutral-font-list.log': 'Neutral computed family gained an equivalent fallback; exact serialized-string expectation renewed as an all-neutral-family invariant',
        'cpu/negative/graph-hit-target-control.log': 'Restoring generic frame button rules fails the production graph target background alpha invariant; original corrected rules restored before successful builds',
        'cpu/negative/primary-variant-control.log': 'Restoring weak primary variant selector fails the exact Tabard accent assertion; corrected original restored before successful builds',
        'cpu/negative/test-import.log': 'New regression test imported wrong module name; corrected to the existing Genet Livery API'},
    'limits': ['Cross-app identity/source/domain action flow is recorded direction, not executed interop',
        'Browser reference entry uses current local IndexedDB storage; durable reservoir and identity resolver are separate admitted work',
        'Graph overview glyphs, domain node presentation, live carrier and app-wide entry adoption are outside this frame qualification',
        'Scripted production input and actual rendered captures; no new physical pointer or screen-reader traversal receipt',
        'Other consumer apps and read-only viewer runtime acceptance are not transferred from this pilot']}
(out / 'qualification.json').write_text(json.dumps(qualification, indent=2) + '\n')
print(json.dumps({'native_presented': sum(r['presented'] for r in native),
                  'native_captures': sum(r['captured'] for r in native), 'images': len(artifacts),
                  'frame_pixel': frame, 'reader_pixels': [reader_light, reader_dark]}))
