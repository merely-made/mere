#!/bin/sh
# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0
#
# Install a signed djinn-custodian as a launchd agent for one owner (custody
# isolation plan §5.2). Run as an administrator, never in the account whose
# vault the custodian will hold, and sign in an account that account cannot
# use (H1-F1):
#
#   sudo install-macos.sh USER SIGNED-BINARY TEAM-ID DJINN-TEAM-ID
#
# The binary must carry TEAM-ID's signature, identifier
# com.merely.djinn.custodian and the hardened runtime without
# get-task-allow; the custodian refuses protected mode otherwise. Djinn
# must be signed as com.merely.djinn by DJINN-TEAM-ID. Unlock with the
# owner's existing vault happens through Djinn's start.
set -eu

[ "$(id -u)" = 0 ] || { echo 'Run with sudo from an administrator account.' >&2; exit 2; }
[ $# -eq 4 ] || { echo 'usage: install-macos.sh USER SIGNED-BINARY TEAM-ID DJINN-TEAM-ID' >&2; exit 2; }
user=$1 binary=$2 team=$3 djinn_team=$4
case "$user$team$djinn_team" in *[!A-Za-z0-9_.-]*|'') echo 'Refusing unexpected characters.' >&2; exit 2;; esac
uid=$(id -u "$user")
home=$(dscl . -read "/Users/$user" NFSHomeDirectory | sed 's/^NFSHomeDirectory: //')
here=$(cd "$(dirname "$0")" && pwd)
root="/Library/Application Support/Merely/djinn-custodian"

requirement="anchor apple generic and identifier \"com.merely.djinn.custodian\" and certificate leaf[subject.OU] = \"$team\""
codesign --verify --strict --verbose=2 -R="$requirement" "$binary"
codesign -d --verbose=4 "$binary" 2>&1 | grep -q 'flags=.*runtime' || { echo 'The binary lacks the hardened runtime.' >&2; exit 2; }
if codesign -d --entitlements - "$binary" 2>/dev/null | grep -q get-task-allow; then
    echo 'The binary carries get-task-allow.' >&2; exit 2
fi

install -d -o root -g wheel -m 0755 "/Library/Application Support/Merely" "$root" "$root/clients"
install -o root -g wheel -m 0755 "$binary" "$root/djinn-custodian"
state="$home/Library/Application Support/Merely/djinn-custodian"
install -d -o "$user" -g staff -m 0700 "$home/Library/Application Support/Merely" "$state" "$state/vault" "$state/data"
# Djinn's grants (custody isolation plan, H2 inventory). Djinn is matched
# by its code signature, not as "some process of this user", so its grants
# run unattended as H1's did.
grants=$(sed -e 's/@CONSENT@/none/g' -e 's/@SIGN_CONSENT@/none/g' "$here/packaging/djinn-grants.json.in")
cat > "$root/$user.json" <<EOF
{
  "vault": {"directory": "$state/vault", "data_root": "$state/data"},
  "owner": {"unix_uid": $uid},
  "consent": "deny",
  "principals": [
    {"name": "djinn", "match": {"signed": {"team": "$djinn_team", "identifier": "com.merely.djinn"}}, "grants": $grants}
  ]
}
EOF
cat > "$root/clients/$uid.json" <<EOF
{"endpoint": "$state/custody.sock",
 "server": {"signed": [{"team": "$team", "identifier": "com.merely.djinn.custodian"}]},
 "agent": "$state/agent.sock"}
EOF
chmod 0644 "$root/$user.json" "$root/clients/$uid.json"
sed -e "s|@USER@|$user|g" -e "s|@HOME@|$home|g" "$here/packaging/macos/com.merely.djinn.custodian.plist.in" \
    > /Library/LaunchAgents/com.merely.djinn.custodian.plist
chown root:wheel /Library/LaunchAgents/com.merely.djinn.custodian.plist
chmod 0644 /Library/LaunchAgents/com.merely.djinn.custodian.plist
plutil -lint /Library/LaunchAgents/com.merely.djinn.custodian.plist
echo "Installed. Log in as $user (or: launchctl bootstrap gui/$uid /Library/LaunchAgents/com.merely.djinn.custodian.plist)."
echo "Move or create the vault in $state/vault with: $root/djinn-custodian adopt|init --config '$root/$user.json'"
echo "Point SSH_AUTH_SOCK at $state/agent.sock."
