#!/usr/bin/env python3
"""Execute already-built native and wasm-bindgen replay examples; retain exact inputs and traces."""
import argparse
import copy
import hashlib
import json
import pathlib
import platform
import subprocess
import sys
import tarfile

ROOT = pathlib.Path(__file__).resolve().parents[2]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def first_difference(left, right, prefix=None):
    """Compare existing state-witness entries without rehashing their values."""
    for (lt, lw), (rt, rw) in zip(left, right):
        if lt != rt:
            return {"tick": min(lt, rt), "label": "missing-tick"}
        le = {e['label']: e['digest'] for e in lw if prefix is None or e['label'].startswith(prefix)}
        re = {e['label']: e['digest'] for e in rw if prefix is None or e['label'].startswith(prefix)}
        for label in sorted(le.keys() | re.keys()):
            if le.get(label) != re.get(label):
                return {"tick": lt, "label": label, "left": le.get(label), "right": re.get(label)}
    if len(left) != len(right):
        extra = left[len(right):] or right[len(left):]
        return {"tick": extra[0][0], "label": "missing-tick"}
    return None


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--target-dir', type=pathlib.Path, required=True)
    parser.add_argument('--output', type=pathlib.Path, required=True)
    parser.add_argument('--case', action='append', choices=['binding', 'occurrence', 'occurrence-mix', 'grouped', 'schedule'],
                        help='run only this case; repeat to select several (default: all)')
    args = parser.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=True)
    target = args.target_dir.resolve()
    results = {"protocol": 1, "base_commit": subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
               "platform": platform.platform(), "node_arch": subprocess.check_output(['node', '-p', 'process.arch'], text=True).strip(), "node": subprocess.check_output(['node', '--version'], text=True).strip(),
               "wasm_bindgen": subprocess.check_output(['wasm-bindgen', '--version'], text=True).strip(), "cases": {}}
    source_paths = subprocess.check_output(['git', 'ls-files', '-co', '--exclude-standard',
        'crates/conatus/seiche', 'crates/canvas/pictograph', 'crates/system/state-witness',
        'support/seiche-replay', 'Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml'], cwd=ROOT, text=True).splitlines()
    results['source_sha256'] = {path: sha(ROOT / path) for path in sorted(set(source_paths)) if (ROOT / path).is_file()}
    results['worktree_status'] = subprocess.check_output(['git', 'status', '--short'], cwd=ROOT, text=True)
    (out / 'qualified-source.patch').write_bytes(subprocess.check_output(['git', 'diff', '--binary', 'HEAD'], cwd=ROOT))
    with tarfile.open(out / 'qualified-sources.tar.gz', 'w:gz') as archive:
        for path in results['source_sha256']:
            archive.add(ROOT / path, arcname=path)
    failures = []
    case_matrix = [
        ('binding', 'crates/conatus/seiche/examples/binding_replay.json', 'run_replay'),
        ('occurrence', 'crates/canvas/pictograph/examples/occurrence_replay.json', 'run_occurrence_replay'),
        ('occurrence-mix', 'crates/canvas/pictograph/examples/occurrence_mix_replay.json', 'run_occurrence_replay'),
        ('grouped', 'crates/canvas/pictograph/examples/grouped_replay.json', 'run_grouped_replay'),
        ('schedule', 'crates/canvas/pictograph/examples/schedule_replay.json', 'run_schedule_replay'),
    ]
    selected = set(args.case or [name for name, _, _ in case_matrix])
    for name, relative, export in case_matrix:
        if name not in selected:
            continue
        example = {'binding': 'binding_replay', 'occurrence': 'occurrence_replay',
                   'occurrence-mix': 'occurrence_replay', 'grouped': 'grouped_replay',
                   'schedule': 'schedule_replay'}[name]
        native = target / 'debug/examples' / example
        wasm = target / 'wasm32-unknown-unknown/debug/examples' / (example + '_wasm.wasm')
        pkg = out / (name + '-wasm')
        subprocess.run(['wasm-bindgen', str(wasm), '--target', 'nodejs', '--out-dir', str(pkg), '--out-name', 'replay'], check=True)
        fixture_path = ROOT / relative
        fixture = json.loads(fixture_path.read_text())
        wire = json.dumps(fixture, separators=(',', ':'))
        (out / (name + '-input.json')).write_text(wire + '\n')
        node_script = "const fs=require('node:fs');const m=require(process.argv[1]);process.stdout.write(m[process.argv[2]](fs.readFileSync(0,'utf8')));"

        def run(label, payload, wasm_runtime=False, refusal=None):
            cmd = ['node', '-e', node_script, str(pkg / 'replay.js'), export] if wasm_runtime else [str(native)]
            proc = subprocess.run(cmd, input=payload, text=True, capture_output=True, timeout=60)
            (out / f'{name}-{label}.stdout.json').write_text(proc.stdout)
            (out / f'{name}-{label}.stderr.txt').write_text(proc.stderr)
            if refusal:
                if proc.returncode == 0 or refusal not in proc.stderr:
                    raise AssertionError(f'{name}/{label}: expected refusal path {refusal!r}: {proc.stderr}')
                return {"refused": refusal, "exit": proc.returncode}
            if proc.returncode:
                raise RuntimeError(f'{name}/{label} exited {proc.returncode}: {proc.stderr}')
            return json.loads(proc.stdout)

        n1 = run('native-1', wire)
        n2 = run('native-2', wire)
        w1 = run('wasm-1', wire, True)
        w2 = run('wasm-2', wire, True)
        if name == 'binding':
            expected_ticks = list(range(1, len(fixture['ticks']) + 1))
        elif name in ('grouped', 'schedule'):
            expected_ticks = list(range(fixture['ticks'] + 1))
        else:
            expected_ticks = list(range(sum(fixture['tick_batches']) + 1))
        for label, value in [('native-1', n1), ('native-2', n2), ('wasm-1', w1), ('wasm-2', w2)]:
            if [tick for tick, _ in value['trace']] != expected_ticks:
                raise AssertionError(f'{name}/{label}: trace omitted or added ticks')
        differences = {"native_processes": first_difference(n1['trace'], n2['trace']),
                       "wasm_processes": first_difference(w1['trace'], w2['trace']),
                       "native_wasm": first_difference(n1['trace'], w1['trace'])}
        for key, difference in differences.items():
            if difference:
                failures.append(f'{name}/{key}: {difference}')
        if name == 'occurrence':
            for label, candidate in [('native-2', n2), ('wasm-1', w1), ('wasm-2', w2)]:
                for field in ('relationship_ids', 'ticks'):
                    if n1[field] != candidate[field]:
                        failures.append(f'{name}/{label}/{field}: metadata differs')
        if name in ('occurrence-mix', 'grouped'):
            for label, candidate in [('native-1', n1), ('native-2', n2), ('wasm-1', w1), ('wasm-2', w2)]:
                difference = first_difference(candidate['trace'], candidate['reopened_trace'])
                if difference:
                    failures.append(f'{name}/{label}/saved-recipe-reopen: {difference}')
                if [tick for tick, _ in candidate['reopened_trace']] != expected_ticks:
                    failures.append(f'{name}/{label}/reopened trace omitted or added ticks')
            if name == 'occurrence-mix':
                for label, candidate in [('native-2', n2), ('wasm-1', w1), ('wasm-2', w2)]:
                    for field in ('relationship_ids', 'ticks'):
                        if n1[field] != candidate[field]:
                            failures.append(f'{name}/{label}/{field}: metadata differs')
            else:
                authored_spec = json.loads(fixture['spec'])
                for label, candidate in [('native-1', n1), ('native-2', n2), ('wasm-1', w1), ('wasm-2', w2)]:
                    if candidate['saved_spec'] != authored_spec:
                        failures.append(f'{name}/{label}/saved spec differs from authored recipe')
                    if candidate['saved_spec']['root']['outer']['weight'] != 16.0:
                        failures.append(f'{name}/{label}/outer weight did not retain authored 16.0 f64')
                    if candidate['initial_group_sizes'] != [2, 2]:
                        failures.append(f'{name}/{label}/Canvas groups are not two groups of two')
                for label, candidate in [('native-2', n2), ('wasm-1', w1), ('wasm-2', w2)]:
                    for field in ('ticks', 'partition', 'saved_spec', 'initial_group_sizes'):
                        if n1[field] != candidate[field]:
                            failures.append(f'{name}/{label}/{field}: metadata differs')
        controls = {}
        if name == 'binding':
            changed = copy.deepcopy(fixture)
            changed['perturb'] = {"tick": 7, "dt": 1.0 / 30.0}
            for runtime in (False, True):
                label = 'dt-control-' + ('wasm' if runtime else 'native')
                value = run(label, json.dumps(changed), runtime)
                difference = first_difference(value['trace'], value['perturbation']['trace'], 'position:')
                if difference is None or difference['tick'] != 7:
                    raise AssertionError(f'{label}: physical divergence must begin at tick 7: {difference}')
                controls[label] = {"position_divergence": difference, "state_witness_divergence": value['perturbation']['first_divergence']}
            changed = copy.deepcopy(fixture)
            changed['spec']['root'] = {"node": "preset", "id": "unavailable"}
            for runtime in (False, True):
                label = 'unsupported-spec-' + ('wasm' if runtime else 'native')
                controls[label] = run(label, json.dumps(changed), runtime, refusal='spec')
        elif name in ('occurrence', 'occurrence-mix'):
            changed = copy.deepcopy(fixture)
            changed['include_relationships'] = False
            for runtime in (False, True):
                label = 'edge-free-' + ('wasm' if runtime else 'native')
                value = run(label, json.dumps(changed), runtime)
                difference = first_difference(n1['trace'] if not runtime else w1['trace'], value['trace'], 'position-bits:')
                if difference is None:
                    raise AssertionError(f'{label}: relationship must change position trace')
                controls[label] = difference
            for control, value, path in [('channels', {'mass': 'mass.pagerank'}, 'channels'),
                                         ('target', {'arrangement': 'arrangement.unsupported'}, 'dynamics.spec.target.arrangement')]:
                changed = copy.deepcopy(fixture)
                spec = json.loads(changed['spec'])
                spec[control] = value
                changed['spec'] = json.dumps(spec)
                for runtime in (False, True):
                    label = control + '-refusal-' + ('wasm' if runtime else 'native')
                    controls[label] = run(label, json.dumps(changed), runtime, refusal=path)
            if name == 'occurrence-mix':
                base = n1
                changed = copy.deepcopy(fixture)
                spec = json.loads(changed['spec'])
                spec['root']['parts'][0]['weight'] = 0.75
                spec['root']['parts'][1]['weight'] = 0.25
                changed['spec'] = json.dumps(spec)
                for runtime in (False, True):
                    label = 'share-divergence-' + ('wasm' if runtime else 'native')
                    value = run(label, json.dumps(changed), runtime)
                    difference = first_difference(base['trace'] if not runtime else w1['trace'], value['trace'], 'position-bits:')
                    if difference is None:
                        raise AssertionError(f'{label}: changed mix shares must alter positions')
                    controls[label] = difference

                changed = copy.deepcopy(fixture)
                spec = json.loads(changed['spec'])
                spec['root']['parts'][1]['id'] = 'stress.kamada-kawai'
                changed['spec'] = json.dumps(spec)
                for runtime in (False, True):
                    label = 'law-divergence-' + ('wasm' if runtime else 'native')
                    value = run(label, json.dumps(changed), runtime)
                    difference = first_difference(base['trace'] if not runtime else w1['trace'], value['trace'], 'position-bits:')
                    if difference is None:
                        raise AssertionError(f'{label}: changed mix law must alter positions')
                    controls[label] = difference

                changed = copy.deepcopy(fixture)
                spec = json.loads(changed['spec'])
                spec['root']['parts'][1]['id'] = 'still.default'
                changed['spec'] = json.dumps(spec)
                for runtime in (False, True):
                    label = 'kinematic-mix-refusal-' + ('wasm' if runtime else 'native')
                    controls[label] = run(label, json.dumps(changed), runtime, refusal='root.parts[1]')
        elif name == 'schedule':
            expected_stages = [0] * 8 + [1] * 16 + [None]
            for label, candidate in [('native-1', n1), ('native-2', n2), ('wasm-1', w1), ('wasm-2', w2)]:
                if candidate['stage_indices'] != expected_stages:
                    failures.append(f'{name}/{label}: stage transition differs: {candidate["stage_indices"]}')
                if candidate['reopened_stage_indices'] != expected_stages:
                    failures.append(f'{name}/{label}/fresh-reopen: stage transition differs')
                if [tick for tick, _ in candidate['reopened_trace']] != expected_ticks:
                    failures.append(f'{name}/{label}/fresh-reopen: trace omitted or added ticks')
                if candidate['saved_spec'] != json.loads(fixture['spec']):
                    failures.append(f'{name}/{label}: saved schedule differs from authored recipe')
                difference = first_difference(candidate['trace'], candidate['reopened_trace'])
                if difference:
                    failures.append(f'{name}/{label}/fresh-reopen: {difference}')
            for runtime, baseline in ((False, n1), (True, w1)):
                changed = copy.deepcopy(fixture)
                spec = json.loads(changed['spec'])
                spec['root']['stages'][0]['stop']['frames'] = 9
                changed['spec'] = json.dumps(spec)
                label = 'stop-shift-' + ('wasm' if runtime else 'native')
                value = run(label, json.dumps(changed), runtime)
                expected_shift = [0] * 9 + [1] * 16
                if value['stage_indices'] != expected_shift:
                    raise AssertionError(f'{label}: changed stop must move stage transition from tick 8 to tick 9')
                controls[label] = {'baseline_stage_indices': baseline['stage_indices'], 'changed_stage_indices': value['stage_indices'], 'first_shift_tick': 8}

                changed = copy.deepcopy(fixture)
                spec = json.loads(changed['spec'])
                spec['root']['stages'][0]['capture'] = 'pinned'
                changed['spec'] = json.dumps(spec)
                label = 'capture-control-' + ('wasm' if runtime else 'native')
                value = run(label, json.dumps(changed), runtime)
                role_difference = first_difference(baseline['trace'], value['trace'], 'role:')
                position_difference = first_difference(baseline['trace'], value['trace'], 'position-bits:')
                if role_difference is None or role_difference['tick'] < 8:
                    raise AssertionError(f'{label}: changed capture must alter roles at/after tick 8: {role_difference}')
                if position_difference is None or position_difference['tick'] <= 8:
                    raise AssertionError(f'{label}: changed capture must alter subsequent positions: {position_difference}')
                controls[label] = {'role_divergence': role_difference, 'subsequent_position_divergence': position_difference}

            changed = copy.deepcopy(fixture)
            spec = json.loads(changed['spec'])
            spec['root']['stages'][0]['node'] = {
                'node': 'grouped', 'partition': 'groups.bridges',
                'outer': {'node': 'preset', 'id': 'charge.barnes-hut'},
                'inner': {'node': 'preset', 'id': 'spring.rapier'}
            }
            changed['spec'] = json.dumps(spec)
            for runtime in (False, True):
                label = 'unsupported-stage-partition-' + ('wasm' if runtime else 'native')
                controls[label] = run(label, json.dumps(changed), runtime, refusal='root.stages[0].node.partition')
        elif name == 'grouped':
            for runtime, baseline in ((False, n1), (True, w1)):
                changed = copy.deepcopy(fixture)
                spec = json.loads(changed['spec'])
                spec['root']['partition'] = 'groups.coloring'
                changed['spec'] = json.dumps(spec)
                label = 'partition-divergence-' + ('wasm' if runtime else 'native')
                value = run(label, json.dumps(changed), runtime)
                difference = first_difference(baseline['trace'], value['trace'], 'position-bits:')
                if difference is None:
                    raise AssertionError(f'{label}: changed Canvas groups channel must alter positions')
                controls[label] = difference

                changed = copy.deepcopy(fixture)
                spec = json.loads(changed['spec'])
                spec['root']['inner']['id'] = 'stress.kamada-kawai'
                changed['spec'] = json.dumps(spec)
                label = 'law-divergence-' + ('wasm' if runtime else 'native')
                value = run(label, json.dumps(changed), runtime)
                difference = first_difference(baseline['trace'], value['trace'], 'position-bits:')
                if difference is None:
                    raise AssertionError(f'{label}: changed grouped inner law must alter positions')
                controls[label] = difference

            changed = copy.deepcopy(fixture)
            spec = json.loads(changed['spec'])
            spec['root']['partition'] = 'groups.bridges'
            changed['spec'] = json.dumps(spec)
            for runtime in (False, True):
                label = 'unsupported-partition-' + ('wasm' if runtime else 'native')
                controls[label] = run(label, json.dumps(changed), runtime, refusal='root.partition')

            # Malformed extreme f64s exercise the authored direct-spec guard in
            # both compiled runtimes. They must be rejected before Canvas runs.
            for weight in (1e300, 1e-300):
                changed = copy.deepcopy(fixture)
                spec = json.loads(changed['spec'])
                spec['root']['outer']['weight'] = weight
                changed['spec'] = json.dumps(spec)
                for runtime in (False, True):
                    label = 'outer-weight-refusal-' + format(weight, '.0e') + '-' + ('wasm' if runtime else 'native')
                    controls[label] = run(label, json.dumps(changed), runtime, refusal='root.outer.weight')
        results['cases'][name] = {"accepted_ticks": expected_ticks[-1], "witnesses": len(n1['trace']), "input_sha256": sha(out / (name + '-input.json')), "native_sha256": sha(native), "wasm_sha256": sha(wasm),
                                  "executed_wasm_sha256": sha(pkg / 'replay_bg.wasm'), "wasm_glue_sha256": sha(pkg / 'replay.js'),
                                  "fixture_sha256": sha(fixture_path), "differences": differences, "controls": controls}
    results['failures'] = failures
    results['qualified'] = not failures
    results['limits'] = ['Fresh-start replay for these fixtures on this native host and Node Wasm runtime.',
                         'The occurrence Mix case round-trips the serialized DynamicsSlot recipe before replay; this does not qualify Graphshell v4 persistence or checkpoint restoration.',
                         'No checkpoint restoration, cross-OS, browser UI, actor replay or consumer repin qualification.',
                         'Occurrence witnesses cover public positions, roles and source identities; Seiche witnesses also cover velocities and field time.',
                         'The direct Canvas grouped witness covers public groups-channel membership, roles and position bits only; it does not witness velocities, clocks or checkpoint state.',
                         'The direct Canvas schedule witness covers public positions, roles and schedule-stage readback only; it does not witness velocities, clocks or checkpoint state.']
    (out / 'replay-results.json').write_text(json.dumps(results, indent=2) + '\n')
    print(json.dumps(results, indent=2))
    return bool(failures)


if __name__ == '__main__':
    sys.exit(main())
