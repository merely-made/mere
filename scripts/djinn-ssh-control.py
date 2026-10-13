#!/usr/bin/env python3
# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0
"""Owner-local SSH setup client. All key generation and locking happen in Djinn.

Uses the existing Graphshell identity and custody routes. It never reads a vault,
imports a private key, or handles a passphrase. Public keys remain available via
personae-vault list/pub. SSH signing remains on the dedicated agent endpoint.
"""
import argparse
import base64
import json
import os
import socket
import struct


class Client:
    def __init__(self, endpoint, route):
        self.sock = None
        if os.name == "nt":
            self.io = open(endpoint, "r+b", buffering=0)
        else:
            self.sock = socket.socket(socket.AF_UNIX)
            self.sock.settimeout(20)
            self.sock.connect(endpoint)
            self.io = self.sock.makefile("rwb", buffering=0)
        self.number = 0
        self.write({"schema": "mere.graphshell/app-broker-hello/v2",
                    "app": "personae-vault", "route": route})
        challenge = self.read()
        if challenge.get("type") != "challenge":
            raise RuntimeError("Djinn refused the route hello")
        self.write({"type": "connect", "schema": "mere.graphshell/app-connect/v1",
                    "host_nonce": challenge["challenge"]["host_nonce"],
                    "client_nonce": base64.urlsafe_b64encode(os.urandom(32)).decode().rstrip("=")})
        if self.read().get("type") != "connected":
            raise RuntimeError("Djinn refused the owner-local session")

    def exact(self, size):
        data = bytearray()
        while len(data) < size:
            chunk = self.io.read(size - len(data))
            if not chunk:
                raise RuntimeError("Djinn closed the session")
            data.extend(chunk)
        return bytes(data)

    def read(self):
        size, = struct.unpack("<I", self.exact(4))
        if size > 16 * 1024 * 1024:
            raise RuntimeError("Oversized Djinn frame")
        return json.loads(self.exact(size))

    def write(self, value):
        data = json.dumps(value, separators=(",", ":")).encode()
        frame = memoryview(struct.pack("<I", len(data)) + data)
        while frame:
            written = self.io.write(frame)
            if not written:
                raise RuntimeError("Djinn closed the session")
            frame = frame[written:]

    def custody(self, call):
        self.number += 1
        self.write({"type": "call", "id": self.number, "call": call})
        answer = self.read()
        if answer.get("type") != "answer" or answer["id"] != self.number:
            raise RuntimeError("Unexpected custody response")
        if "Err" in answer["answer"]:
            raise RuntimeError(str(answer["answer"]["Err"]))
        return answer["answer"]["Ok"]

    def close(self):
        self.io.close()
        if self.sock:
            self.sock.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--endpoint", default=os.environ.get("GRAPHSHELL_APP_ENDPOINT"))
    commands = parser.add_subparsers(dest="command", required=True)
    commands.add_parser("status")
    commands.add_parser("lock")
    commands.add_parser("list")
    commands.add_parser("ca")
    commands.add_parser("pub").add_argument("slot")
    commands.add_parser("generate").add_argument("comment")
    args = parser.parse_args()
    if not args.endpoint:
        parser.error("Pass the resident's owner-only --endpoint")
    client = Client(args.endpoint, "custody")
    try:
        if args.command in ("generate", "list", "pub", "ca"):
            roster = client.custody({"call": "roster"})
            answer = client.custody({"call": "vault_command", "profile": roster["chosen"],
                                     "command": "generate-ssh" if args.command == "generate" else args.command,
                                     "args": [args.comment] if args.command == "generate" else
                                             [args.slot] if args.command == "pub" else []})
            print(answer["text"], end="")
        else:
            print(json.dumps(client.custody({"call": "status" if args.command == "status" else "lock_vault"})))
    finally:
        client.close()


if __name__ == "__main__":
    main()
