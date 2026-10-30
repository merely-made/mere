// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

#[cfg(feature = "forbidden-api")]
pub use djinn::door::app_client::AppConnection;
pub use djinn::door::wire::{AppHostMessage, AppMessage, BrowserChallenge};

#[cfg(test)]
mod tests {
    use djinn::door::app_admission::AppHello;
use djinn::door::frame::read_native_message;

    #[test]
    fn installed_hello_decodes_from_native_endian_reference_frame() {
        let body = br#"{"schema":"mere.graphshell/app-broker-hello/v2","app":"turnstone","route":"custody"}"#;
        let mut frame = (body.len() as u32).to_ne_bytes().to_vec();
        frame.extend_from_slice(body);
        let hello: AppHello = read_native_message(&mut frame.as_slice()).unwrap().unwrap();
        let request = hello.accept().unwrap();
        assert_eq!(request.route.as_str(), "custody");
    }
}
