// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

#[cfg(feature = "forbidden-api")]
pub use djinn::door::app_client::AppConnection;
pub use djinn::door::custody::{CustodyAnswer, CustodyCall};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_round_trips_without_native_connection_features() {
        let answer = CustodyAnswer::Released {
            master: [17; 32],
            keys: vec![],
        };
        let wire = serde_json::to_vec(&answer).unwrap();
        assert_eq!(
            serde_json::from_slice::<CustodyAnswer>(&wire).unwrap(),
            answer
        );
    }
}
