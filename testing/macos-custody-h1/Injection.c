// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
// Deliberately observable injection control, using only a synthetic marker.
#include <fcntl.h>
#include <stdlib.h>
#include <unistd.h>
__attribute__((constructor)) static void synthetic_injection_marker(void) {
    const char *path = getenv("H1_FIXTURE_INJECT_MARKER");
    if (!path) return;
    int fd = open(path, O_WRONLY | O_CREAT | O_EXCL | O_NOFOLLOW, 0600);
    if (fd >= 0) { (void)write(fd, "synthetic injection loaded\n", 27); close(fd); }
}
