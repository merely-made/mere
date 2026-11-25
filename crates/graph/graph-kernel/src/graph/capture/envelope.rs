// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The kernel's 50-variant enum in the engine's journal envelope (engine
//! split D242). Its two resource writes become [`ResourceDelta`]s and every
//! other variant, the shown-resource binding included, rides as the surface
//! half. The enum itself is unchanged in step 1 of the kernel move.

use mere::model::{JournalDelta, ResourceDelta};

use super::CapturedDelta;

impl From<CapturedDelta> for JournalDelta<CapturedDelta> {
    fn from(delta: CapturedDelta) -> Self {
        match delta {
            CapturedDelta::ReplaySetResourceRecordById {
                resource_id,
                record,
            } => Self::Resource(ResourceDelta::ReplaySetResourceRecordById {
                resource_id,
                record,
            }),
            CapturedDelta::ReplaySetResourceEdgesByIds {
                from_resource_id,
                to_resource_id,
                edges,
            } => Self::Resource(ResourceDelta::ReplaySetResourceEdgesByIds {
                from_resource_id,
                to_resource_id,
                edges,
            }),
            surface => Self::Surface(surface),
        }
    }
}

impl From<ResourceDelta> for CapturedDelta {
    fn from(delta: ResourceDelta) -> Self {
        match delta {
            ResourceDelta::ReplaySetResourceRecordById {
                resource_id,
                record,
            } => Self::ReplaySetResourceRecordById {
                resource_id,
                record,
            },
            ResourceDelta::ReplaySetResourceEdgesByIds {
                from_resource_id,
                to_resource_id,
                edges,
            } => Self::ReplaySetResourceEdgesByIds {
                from_resource_id,
                to_resource_id,
                edges,
            },
        }
    }
}

impl From<JournalDelta<CapturedDelta>> for CapturedDelta {
    fn from(delta: JournalDelta<CapturedDelta>) -> Self {
        match delta {
            JournalDelta::Resource(delta) => delta.into(),
            JournalDelta::Surface(delta) => delta,
        }
    }
}
