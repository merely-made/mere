"""Reuse the bounded whole-owner runner with a separate immutable case/receipt."""
from pathlib import Path
HERE = Path(__file__).resolve().parent
source = (HERE / 'run_stickleback_group_retry_probe.py').read_text(encoding='utf-8')
source = source.replace("NAME = 'stickleback_group_retry_probe'", "NAME = 'stickleback_prune_erase_probe'")
source = source.replace('libauthority_success_group_retry_owner.rlib', 'libauthority_success_prune_erase_owner.rlib')
source = source.replace('authority-success-group-retry.exe', 'authority-success-prune-erase.exe')
source = source.replace("['identity', 'muniment']", "['muniment', 'p2panda_core', 'serde']")
source = source.replace('checks == 3', 'checks == 2')
source = source.replace('Fresh whole unchanged Stickleback owner over17exact retained suppliers; real identities, signed operation processing, MemoryBackend and real group encryption. Three public-API cases: successful first save, failed save followed by same-live-session retry, and failed save followed by reload-before-retry. Secret snapshots stay in memory and are neither printed nor retained. Injected callback refusal, not real disk failure or host-level data loss; no original suite or live network. Compile source hashes do not grant source read credit.',
    'Fresh whole unchanged Stickleback owner over17exact retained suppliers. Real signed three-operation corpus, real OperationProcessor/MemoryBackend, same policy in single-operation and native-drop batch paths. Single-operation prune+erase recreates a removed prefix header without its pointer; batch control leaves only the checkpoint. Neutral policy explicitly authorizes effects; no governance/destruction policy chosen, no actual user data or live network, original tests0. Counts describe exact observed paths, not every retention combination. Compile hashes do not grant read credit.')
exec(compile(source, str(Path(__file__)), 'exec'))
