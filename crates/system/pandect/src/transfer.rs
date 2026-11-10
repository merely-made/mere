// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The records a graph-selection transfer writes: its manifest, the blobs it
//! names and its receipt (engine split D70).
//!
//! Moved from the reference app's `transfer`, which keeps preparing and
//! applying a transfer. These are what crosses between devices, so the
//! resident reads them without linking the app.

use std::sync::LazyLock;

use eidetic::{
    Codicil, Hash, MereNativeFieldSpec, MereNativeSchemaBuilder, SchemaDefinition, SchemaRef,
    TypedPayload,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::access::AccessRecord;
use crate::product_codicil::schema_ref;

pub const TRANSFER_MANIFEST_SCHEMA: &str = "graphshell.transfer-manifest/v1";

static TRANSFER_RECEIPT_SCHEMA_REF: LazyLock<SchemaRef> =
    LazyLock::new(|| schema_ref(&transfer_receipt_schema()));

pub fn transfer_receipt_schema() -> SchemaDefinition {
    MereNativeSchemaBuilder::new("graphshell.TransferReceipt/v1")
        .description("The durable result of applying one graph selection transfer.")
        .field("transfer_id", MereNativeFieldSpec::String, true)
        .field(
            "operation",
            MereNativeFieldSpec::Enum {
                values: vec!["replicate".to_string(), "copy".to_string()],
            },
            true,
        )
        .field("source", MereNativeFieldSpec::Object, true)
        .field("destination", MereNativeFieldSpec::Object, true)
        .field("route", MereNativeFieldSpec::Object, true)
        .field("authorization_grant", MereNativeFieldSpec::String, true)
        .field("manifest_hash", MereNativeFieldSpec::String, true)
        .field("selection_hash", MereNativeFieldSpec::String, true)
        .field("blob_hashes", MereNativeFieldSpec::Array, true)
        .field("id_map", MereNativeFieldSpec::Array, true)
        .field("nodes", MereNativeFieldSpec::U64, true)
        .field("relations", MereNativeFieldSpec::U64, true)
        .field(
            "destination_access_records",
            MereNativeFieldSpec::Array,
            true,
        )
        .field("completed_at_ms", MereNativeFieldSpec::U64, true)
        .field(
            "result",
            MereNativeFieldSpec::Enum {
                values: vec!["completed".to_string()],
            },
            true,
        )
        .build()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TransferOperation {
    Replicate,
    Copy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AccessTransferPolicy {
    ExcludeSourceHistory,
    IncludeSourceHistory,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransferEndpointV1 {
    pub graph: String,
    pub persona: String,
    pub device: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransferRouteV1 {
    pub carrier: String,
    pub peer: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransferBlobV1 {
    pub node_id: Uuid,
    pub role: String,
    pub media_type: String,
    pub content_hash: Hash,
    pub byte_len: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransferManifestV1 {
    pub schema: String,
    pub transfer_id: Uuid,
    pub operation: TransferOperation,
    pub source: TransferEndpointV1,
    pub destination: TransferEndpointV1,
    pub route: TransferRouteV1,
    pub selection_schema: SchemaDefinition,
    pub selection: Codicil,
    pub blobs: Vec<TransferBlobV1>,
    pub access_policy: AccessTransferPolicy,
    pub access_records: Vec<AccessRecord>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransferredIdV1 {
    pub source: Uuid,
    pub destination: Uuid,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TransferResult {
    Completed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransferReceiptV1 {
    pub transfer_id: Uuid,
    pub operation: TransferOperation,
    pub source: TransferEndpointV1,
    pub destination: TransferEndpointV1,
    pub route: TransferRouteV1,
    pub authorization_grant: String,
    pub manifest_hash: Hash,
    pub selection_hash: Hash,
    pub blob_hashes: Vec<Hash>,
    pub id_map: Vec<TransferredIdV1>,
    pub nodes: u64,
    pub relations: u64,
    pub destination_access_records: Vec<Uuid>,
    pub completed_at_ms: u64,
    pub result: TransferResult,
}

impl TypedPayload for TransferReceiptV1 {
    fn schema_ref() -> SchemaRef {
        *TRANSFER_RECEIPT_SCHEMA_REF
    }
}
