// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

#[cfg(feature = "forbidden-api")]
pub use ostiary::app_client::AppConnection;
pub use ostiary::wire::{AppHostMessage, AppMessage, BrowserChallenge};

#[cfg(test)]
mod tests {
    use ostiary::app_admission::AppHello;
    use ostiary::frame::read_cbor_message;

    /// The installed v2 hello as the application door frames it: one
    /// canonical CBOR map, its keys shortest first (engine split D225).
    #[test]
    fn installed_hello_decodes_from_native_endian_reference_frame() {
        let body = b"\xa3\x63app\x69turnstone\x65route\x67custody\x66schema\x78\x23mere.graphshell/app-broker-hello/v2";
        let mut frame = (body.len() as u32).to_ne_bytes().to_vec();
        frame.extend_from_slice(body);
        let hello: AppHello = read_cbor_message(&mut frame.as_slice()).unwrap().unwrap();
        let request = hello.accept().unwrap();
        assert_eq!(request.route.as_str(), "custody");
    }
}
