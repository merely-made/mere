// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
// Intentionally lacks Hardened Runtime, to prove the injection is functional.
#include <stdio.h>
#include <unistd.h>
int main(void) { printf("{\"launchProbeReady\":true,\"pid\":%d,\"ppid\":%d}\n", getpid(), getppid()); return 0; }
