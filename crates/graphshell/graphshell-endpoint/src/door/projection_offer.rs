// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Portable messages for the application projection provider route.
use chirograph::{CarrierNotice, CarrierRequest, CarrierResponse, EndpointDescriptor};
use djinn::door::app_admission::{AppId, AppRouteId};
use serde::{Deserialize, Serialize};

pub const PROJECTION_PROVIDER_ROUTE: &str = "mere.projection-provider";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProviderCall {
    Offer { route: AppRouteId, label: String },
    Next { completed: Option<WorkResult> },
    Withdraw,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProviderAnswer {
    Registered { max_sessions: usize },
    Work { work: Work },
    Idle,
    Withdrawn,
}
/// Narrow facts asserted by the host after viewer admission. These are not
/// credentials and do not authenticate the host or viewer to the provider.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ViewerContext {
    pub session: String,
    pub subject: [u8; 32],
    pub application: Option<AppId>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Work {
    pub id: u64,
    pub session: String,
    pub operation: WorkOperation,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorkOperation {
    Open { context: ViewerContext },
    Request { request: CarrierRequest },
    PollNotice,
    Close,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorkResult {
    pub id: u64,
    pub session: String,
    pub result: WorkOutcome,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorkOutcome {
    Opened { descriptor: EndpointDescriptor },
    Response { response: CarrierResponse },
    Notice { notice: Option<CarrierNotice> },
    Closed,
    Failed { message: String },
}
