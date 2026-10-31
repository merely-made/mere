#!/usr/bin/env bash
# Copyright 2026 Mark Alan Boykin
# SPDX-License-Identifier: MPL-2.0
# Install prebuilt binaries; compilation and passphrases stay out of this script.
set -euo pipefail
if [[ $# -lt 2 || $# -gt 4 ]]; then
    echo 'usage: install-linux.sh DJINN_BINARY VAULT_CLIENT_BINARY [MEMORY_MAX [CPU_QUOTA]]' >&2
    exit 2
fi
[[ $EUID -ne 0 ]] || { echo 'Install as the desktop user, not root.' >&2; exit 2; }
source_djinn=$(realpath -- "$1")
source_client=$(realpath -- "$2")
memory_max=${3:-768M}
cpu_quota=${4:-50%}
[[ $memory_max =~ ^[1-9][0-9]*[MG]$ && $cpu_quota =~ ^[1-9][0-9]*%$ ]] || {
    echo 'Use a memory limit such as 768M and a CPU quota such as 50%.' >&2; exit 2;
}
[[ -x $source_djinn && -x $source_client ]] || { echo 'Both binaries must be executable.' >&2; exit 2; }
systemctl --user show-environment >/dev/null
install_root=${XDG_DATA_HOME:-$HOME/.local/share}/djinn/bin
unit_dir=${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user
unit=$unit_dir/djinn.service
marker='# Djinn installed resident'
if [[ -e $unit ]] && ! head -n 1 "$unit" | grep -qxF "$marker"; then
    echo "Preserve the existing unit owned elsewhere: $unit" >&2; exit 2
fi
if systemctl --user is-active --quiet djinn.service; then
    echo 'Djinn is active. Stop it deliberately before replacing its binaries.' >&2; exit 2
fi
runtime=${XDG_RUNTIME_DIR:?Run from the desktop user session}
if [[ -e $runtime/graphshell-app.sock || -e $runtime/djinn/agent.sock ]]; then
    [[ -e $unit ]] || { echo 'A resident socket already exists with no owned unit.' >&2; exit 2; }
    # The resident currently leaves its app socket pathname after shutdown.
    # Remove only a same-user socket from this installed unit, with no listener.
    python3 - "$runtime/graphshell-app.sock" "$runtime/djinn/agent.sock" <<'PY'
import errno, os, socket, stat, sys
for path in sys.argv[1:]:
    try:
        before = os.lstat(path)
    except FileNotFoundError:
        continue
    if not stat.S_ISSOCK(before.st_mode) or before.st_uid != os.getuid():
        raise SystemExit(f"Preserve socket path with another owner or type: {path}")
    with socket.socket(socket.AF_UNIX) as probe:
        try:
            probe.connect(path)
        except OSError as error:
            if error.errno != errno.ECONNREFUSED:
                raise SystemExit(f"Cannot establish that socket is unused: {path}")
        else:
            raise SystemExit(f"Preserve a live resident socket: {path}")
    after = os.lstat(path)
    if (before.st_dev, before.st_ino) != (after.st_dev, after.st_ino):
        raise SystemExit(f"Socket changed during ownership check: {path}")
    os.unlink(path)
PY
fi
for name in djinn personae-vault; do
    link=$HOME/.local/bin/$name
    if [[ -e $link || -L $link ]] && [[ $(readlink "$link" || true) != "$install_root/$name" ]]; then
        echo "Preserve the existing command owned elsewhere: $link" >&2; exit 2
    fi
done
# systemd ExecStart is not shell syntax. Refuse characters requiring escaping.
[[ $install_root =~ ^/[a-zA-Z0-9_./-]+$ ]] || {
    echo 'The install root contains characters requiring systemd escaping.' >&2; exit 2;
}
mkdir -p "$install_root" "$unit_dir" "$HOME/.local/bin"
install -m 755 "$source_djinn" "$install_root/djinn"
install -m 755 "$source_client" "$install_root/personae-vault"
ln -sfn "$install_root/djinn" "$HOME/.local/bin/djinn"
ln -sfn "$install_root/personae-vault" "$HOME/.local/bin/personae-vault"
cat > "$unit" <<UNIT
$marker
[Unit]
Description=Djinn desktop resident and identity custody
After=graphical-session.target

[Service]
Type=simple
ExecStart=$install_root/djinn --installed --agent-endpoint %t/djinn/agent.sock --browser-endpoint %t/djinn/browser.sock
RuntimeDirectory=djinn
RuntimeDirectoryMode=0700
Restart=on-failure
RestartSec=5
KillSignal=SIGINT
TimeoutStopSec=20
LimitCORE=0
NoNewPrivileges=true
CPUQuota=$cpu_quota
MemoryHigh=$memory_max
MemoryMax=$memory_max

[Install]
WantedBy=default.target
UNIT
systemd-analyze --user verify "$unit"
systemctl --user daemon-reload
systemctl --user enable djinn.service
echo "Installed $unit. Start with: systemctl --user start --no-block djinn.service"
echo 'Djinn asks for the vault passphrase in its native dialog. It stores no passphrase here.'
echo "Dedicated SSH agent: $runtime/djinn/agent.sock"
echo 'Existing ssh-agent services and session environment are preserved.'
