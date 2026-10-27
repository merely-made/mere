#!/bin/sh
# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0
#
# Install Djinn's custodian for one desktop user as a socket-activated
# systemd system service under its own non-login account (custody isolation
# plan §5.2). Run as root:
#
#   install-linux.sh USER BUILT-BINARY [DJINN-VAULT-DIR]
#
# With DJINN-VAULT-DIR the user's existing passphrase vault is copied into
# the custodian's state directory and reopened to the same identity (the
# passphrase is read once from the terminal); the original stays where it is
# for the owner to retire. Without it, create a vault afterwards with
# `runuser -u djinn-custodian-USER -- /usr/local/libexec/djinn-custodian init
# --config /etc/djinn-custodian/USER.json`.
set -eu

[ "$(id -u)" = 0 ] || { echo 'Run as root.' >&2; exit 2; }
[ $# -ge 2 ] || { echo 'usage: install-linux.sh USER BUILT-BINARY [DJINN-VAULT-DIR]' >&2; exit 2; }
user=$1 binary=$2 adopt_from=${3:-}
case $user in *[!a-z0-9_-]*|'') echo 'The user name holds characters this installer refuses.' >&2; exit 2;; esac
uid=$(id -u "$user")
account="djinn-custodian-$user"
here=$(cd "$(dirname "$0")" && pwd)

# The protected mode needs Landlock (ABI 6 is probed again at every start),
# systemd credentials (%d, systemd 250) and polkit.
grep -q landlock /sys/kernel/security/lsm || { echo 'Landlock is not enabled in this kernel.' >&2; exit 2; }
systemd_version=$(systemctl --version | sed -n '1s/^systemd \([0-9]*\).*/\1/p')
[ "${systemd_version:-0}" -ge 250 ] || { echo 'systemd 250 or newer is required.' >&2; exit 2; }
[ -x /usr/bin/pkcheck ] || { echo 'polkit (pkcheck) is required for consent.' >&2; exit 2; }

if ! id -u "$account" >/dev/null 2>&1; then
    useradd --system --no-create-home --home-dir /nonexistent --shell /usr/sbin/nologin --user-group "$account"
fi

install -D -o root -g root -m 0755 "$binary" /usr/local/libexec/djinn-custodian
install -o root -g root -m 0644 "$here/packaging/linux/djinn-custodian@.socket" /etc/systemd/system/
install -o root -g root -m 0644 "$here/packaging/linux/djinn-custodian-agent@.socket" /etc/systemd/system/
install -o root -g root -m 0644 "$here/packaging/linux/djinn-custodian@.service" /etc/systemd/system/

owners=$(getent passwd | cut -d: -f1 | grep '^djinn-custodian-' | sed 's/^/unix-user:/' | tr '\n' ' ' | sed 's/ $//')
sed "s|@OWNERS@|$owners|" "$here/packaging/linux/com.merely.djinn.custody.policy.in" > /usr/share/polkit-1/actions/com.merely.djinn.custody.policy
chmod 0644 /usr/share/polkit-1/actions/com.merely.djinn.custody.policy

install -d -o root -g root -m 0755 /etc/djinn-custodian /etc/djinn-custodian/clients
state=/var/lib/djinn-custodian/$user
# Djinn's grants (custody isolation plan, H2 inventory). Djinn is a
# same-user principal, so each release or act waits for polkit once per
# unlock, and a guest delegation once per signature.
grants=$(sed -e 's/@CONSENT@/per-generation/g' -e 's/@SIGN_CONSENT@/per-operation/g' "$here/packaging/djinn-grants.json.in")
cat > "/etc/djinn-custodian/$user.json" <<EOF
{
  "vault": {"directory": "$state/vault", "data_root": "$state/data"},
  "owner": {"unix_uid": $uid},
  "consent": "polkit",
  "principals": [
    {"name": "djinn", "match": {"unix-uid": $uid}, "grants": $grants}
  ]
}
EOF
chmod 0644 "/etc/djinn-custodian/$user.json"
cat > "/etc/djinn-custodian/clients/$uid.json" <<EOF
{"endpoint": "/run/djinn-custodian/$user.sock", "server": {"unix_uids": [0]},
 "agent": "/run/djinn-custodian/$user-agent.sock"}
EOF
chmod 0644 "/etc/djinn-custodian/clients/$uid.json"

install -d -o "$account" -g "$account" -m 0700 /var/lib/djinn-custodian "$state" "$state/vault" "$state/data"
chown root:root /var/lib/djinn-custodian
chmod 0755 /var/lib/djinn-custodian
if [ -n "$adopt_from" ]; then
    import="$state/import"
    install -d -o "$account" -g "$account" -m 0700 "$import"
    for file in vault.json chosen-profile; do
        [ -f "$adopt_from/$file" ] && install -o "$account" -g "$account" -m 0600 "$adopt_from/$file" "$import/$file"
    done
    printf 'Passphrase of the vault in %s: ' "$adopt_from" >&2
    stty -echo; IFS= read -r passphrase; stty echo; echo >&2
    printf '%s\n' "$passphrase" | runuser -u "$account" -- /usr/local/libexec/djinn-custodian adopt \
        --config "/etc/djinn-custodian/$user.json" --from "$import"
    unset passphrase
    rm -rf "$import"
fi

systemd-analyze verify "/etc/systemd/system/djinn-custodian@.service" "/etc/systemd/system/djinn-custodian@.socket" \
    "/etc/systemd/system/djinn-custodian-agent@.socket" || true
systemctl daemon-reload
systemctl enable --now "djinn-custodian@$user.socket" "djinn-custodian-agent@$user.socket"
echo "Installed. Djinn finds the custodian through /etc/djinn-custodian/clients/$uid.json;"
echo "point SSH_AUTH_SOCK at /run/djinn-custodian/$user-agent.sock."
