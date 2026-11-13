// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Declared-lens L2's metrics for the loader: D8's activation check, the
//! §3.5 golden selection records and one refusal per resolution rule, the
//! order-independence proptest, and the core pack's boot cost.

mod activation;
mod cost;
mod golden;
mod refusals;
pub(crate) mod support;
