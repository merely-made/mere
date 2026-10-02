// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Where is each paired device now? Reads the running resident's paired-device
//! directory through the owner-only application door and prints it.

use djinn::resident_devices::{
    AddrKindV1, DEVICE_DIRECTORY_APP, DEVICE_DIRECTORY_ROUTE, DeviceDirectoryV1, PairedDeviceV1,
    read_directory,
};
use graphshell::native::app_admission::{AppId, AppRouteId};
use graphshell::native::app_client::AppBrokerClient;

const USAGE: &str = "usage: djinn-devices [--json]";

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut json = false;
    for argument in std::env::args().skip(1) {
        match argument.as_str() {
            "--json" => json = true,
            _ => return Err(USAGE.into()),
        }
    }
    let mut client = AppBrokerClient::open_route(
        AppId::new(DEVICE_DIRECTORY_APP),
        AppRouteId::new(DEVICE_DIRECTORY_ROUTE)?,
    )
    .await?;
    let directory = read_directory(&mut client).await?;
    client.close().await?;
    if json {
        println!("{}", serde_json::to_string_pretty(&directory)?);
    } else {
        print!("{}", render(&directory));
    }
    Ok(())
}

fn render(directory: &DeviceDirectoryV1) -> String {
    let mut out = format!("this device  {}\n", directory.local_node);
    if directory.devices.is_empty() {
        out.push_str("no paired devices\n");
    }
    for device in &directory.devices {
        out.push('\n');
        out.push_str(&render_device(device));
    }
    out
}

fn kind(kind: AddrKindV1) -> &'static str {
    match kind {
        AddrKindV1::Direct => "direct",
        AddrKindV1::Relay => "relay",
        AddrKindV1::Other => "other",
    }
}

fn render_device(device: &PairedDeviceV1) -> String {
    let label = if device.label.is_empty() {
        "(no label)"
    } else {
        &device.label
    };
    let state = match (device.connected, device.reachable) {
        (true, _) => "connected",
        (false, true) => "not connected (an address is known)",
        (false, false) => "not connected (no address known)",
    };
    let path: Vec<String> = device
        .path
        .iter()
        .map(|addr| {
            let active = if addr.active { " (active)" } else { "" };
            format!("{} {}{active}", kind(addr.kind), addr.addr)
        })
        .collect();
    let hint = match &device.hint {
        None => "none saved".to_string(),
        Some(hint) => match &hint.unreadable {
            Some(reason) => format!("unreadable: {reason}"),
            None => hint
                .addrs
                .iter()
                .map(|addr| format!("{} {}", kind(addr.kind), addr.addr))
                .collect::<Vec<_>>()
                .join(", "),
        },
    };
    format!(
        "{label}  {}\n  root     {}\n  pairing  {} (added {} ms)\n  now      {state}\n  path     {}\n  hint     {hint}\n",
        device.node_id,
        device.root.as_deref().unwrap_or("none (receive-only)"),
        device.pairing_id.as_deref().unwrap_or("none"),
        device.added_ms,
        if path.is_empty() {
            "none".to_string()
        } else {
            path.join(", ")
        },
    )
}
