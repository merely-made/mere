// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
// Intentionally lacks Hardened Runtime, to prove the injection is functional.
#include <stdio.h>
#include <unistd.h>
int main(void) { printf("{\"launchProbeReady\":true,\"pid\":%d,\"ppid\":%d}\n", getpid(), getppid()); return 0; }
