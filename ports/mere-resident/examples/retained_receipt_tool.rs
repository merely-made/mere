// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
//! Helper for the native retained-text receipt, on an isolated Djinn only.
//!
//! `grants` prints the owner grants the receipt writes into the isolated
//! profile's settings, which stand for the owner's yes: Graphshell's own
//! declaration, and a lock-only helper. `lock` locks the vault through the
//! custody door as that helper, the way a person's lock action would.
use castellan_client::custody_client::CustodyClient;
use mere_resident::applications::grant_for;
use ostiary::app_admission::AppId;
use ostiary::declaration::{AppDeclaration, CustodyOperation, CustodyUse, NamePattern};

const LOCKER: &str = "receipt-locker";

fn locker() -> AppDeclaration {
    AppDeclaration {
        routes: vec![NamePattern::exact("custody")],
        custody: CustodyUse {
            operations: [CustodyOperation::LockVault, CustodyOperation::Status].into(),
            ..CustodyUse::default()
        },
        ..AppDeclaration::default()
    }
}

#[tokio::main]
async fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("grants") => {
            let grants = vec![
                grant_for(&AppId::new(graphshell_door::declaration::APP), &graphshell_door::declaration::declaration(), None),
                grant_for(&AppId::new(LOCKER), &locker(), None),
            ];
            println!("{}", serde_json::to_string(&grants).unwrap());
        },
        Some("lock") => {
            let mut client = CustodyClient::open_declared(AppId::new(LOCKER), locker())
                .await
                .expect("the custody door admits the locker");
            client.lock_vault().await.expect("the vault locks");
            println!("locked");
        },
        _ => panic!("usage: retained_receipt_tool grants|lock"),
    }
}
