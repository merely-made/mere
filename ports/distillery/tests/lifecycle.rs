// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Receiving control: the shared close sequence keeps every named failure.

use armillary::lifecycle::{CloseAction, CloseFuture, close_all};
use std::sync::{Arc, Mutex};

#[tokio::test]
async fn failed_close_does_not_skip_later_resources_or_lose_their_errors() {
    let observed = Arc::new(Mutex::new(Vec::new()));
    let actions: Vec<(&str, CloseAction<'static>)> = ["first", "second", "third"]
        .into_iter()
        .map(|name| {
            let observed = observed.clone();
            let action: CloseAction<'static> = Box::new(move || {
                Box::pin(async move {
                    observed.lock().unwrap().push(name);
                    if name == "second" {
                        Ok(())
                    } else {
                        Err(format!("{name} refused"))
                    }
                }) as CloseFuture<'static>
            });
            (name, action)
        })
        .collect();
    let report = close_all(actions).await;
    assert_eq!(*observed.lock().unwrap(), ["first", "second", "third"]);
    assert_eq!(report.failure("first"), Some("first refused"));
    assert_eq!(report.failure("second"), None);
    assert_eq!(report.failure("third"), Some("third refused"));
    assert_eq!(report.into_failures().len(), 2);
}
