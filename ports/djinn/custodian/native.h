/* Copyright 2026 Mark Alan Boykin
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 * SPDX-License-Identifier: MPL-2.0
 */
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
size_t djinn_custodian_initialize(const uint8_t *, size_t, const uint8_t *, size_t, bool, uint8_t *, size_t);
size_t djinn_custodian_dispatch(const uint8_t *, size_t, const uint8_t *, size_t, uint8_t *, size_t);
size_t djinn_custodian_confirm(const uint8_t *, size_t, bool, uint8_t *, size_t);
size_t djinn_custodian_unlock(const uint8_t *, size_t, uint8_t *, size_t);
