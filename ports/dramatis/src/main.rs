// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The Dramatis window, on the shared Cambium native host.
use cambium_genet_winit_host::{HostOptions, Init, run};
use dramatis_app::{
    State,
    views::{Logic, SHEET, root},
    worker::Worker,
};
use std::{cell::RefCell, rc::Rc};
fn main() {
    match std::env::args().skip(1).collect::<Vec<_>>().as_slice() {
        [flag] if flag == "--version" => {
            println!("Dramatis {}", env!("CARGO_PKG_VERSION"));
            return;
        },
        [flag] if flag == "--build-info" => {
            println!(
                "{}",
                serde_json::json!({
                    "product": "Dramatis", "version": env!("CARGO_PKG_VERSION"),
                    "source_sha256": env!("DRAMATIS_SOURCE_SHA256"),
                    "target": env!("DRAMATIS_BUILD_TARGET"),
                })
            );
            return;
        },
        [] => {},
        _ => {
            eprintln!("Usage: dramatis-ui [--version | --build-info]");
            std::process::exit(2);
        },
    }
    let slot = Rc::new(RefCell::new(None::<Worker>));
    let drain = slot.clone();
    let dispatch = slot.clone();
    let mut hooks = dramatis_app::host_hooks();
    let mut follow_focus = hooks.after_dispatch;
    hooks.after_wake = Box::new(move |ctx| {
        if let Some(worker) = drain.borrow().as_ref() {
            let replies = worker.drain();
            if !replies.is_empty() {
                ctx.runner.update(|s| {
                    for (reply, owned) in replies {
                        if owned || !s.busy {
                            s.adopt(reply);
                        }
                    }
                });
            }
        }
    });
    hooks.after_dispatch = Box::new(move |ctx| {
        follow_focus(ctx);
        let mut request = None;
        ctx.runner.update(|s| request = s.request.take());
        if let (Some(request), Some(worker)) = (request, dispatch.borrow().as_ref()) {
            worker.send(request);
        }
    });
    if let Some(path) = std::env::var_os("DRAMATIS_CAPTURE") {
        let path = std::path::PathBuf::from(path);
        let mode = std::env::var("DRAMATIS_CAPTURE_MODE").unwrap_or_else(|_| "unlocked".into());
        let armed = Rc::new(std::cell::Cell::new(false));
        let captured = Rc::new(std::cell::Cell::new(false));
        let signal = captured.clone();
        let mut frames = 0;
        hooks.frame = Box::new(|_| true);
        hooks.after_frame = Box::new(move |ctx| {
            frames += 1;
            if captured.get() {
                *ctx.close = true;
                return;
            }
            assert!(frames < 1200, "receipt did not capture its expected state");
            if armed.get() || ctx.presentation.is_none() {
                return;
            }
            let state = ctx.runner.state();
            let ready = match mode.as_str() {
                "absent" => state.snapshot.is_none() && state.notice.contains("pending"),
                "locked" => state
                    .snapshot
                    .as_ref()
                    .is_some_and(|s| s.lock == dramatis::view::VaultLockView::Locked),
                _ => state.snapshot.as_ref().is_some_and(|s| {
                    s.lock == dramatis::view::VaultLockView::Unlocked && !s.cards.is_empty()
                }),
            };
            if !ready {
                return;
            }
            armed.set(true);
            let destination = path.clone();
            let signal = signal.clone();
            *ctx.capture = Some(Box::new(move |capture| {
                let capture = capture.expect("presented frame capture refused");
                let (surface, view, w, h) =
                    (capture.surface, capture.view, capture.width, capture.height);
                let frame = cambium_genet_winit_host::read_frame(surface, view, w, h)
                    .expect("presented frame readback");
                image::save_buffer(
                    &destination,
                    &frame.rgba,
                    frame.width,
                    frame.height,
                    image::ColorType::Rgba8,
                )
                .expect("receipt PNG");
                signal.set(true);
            }));
        });
    }
    let initial_logical_size = if std::env::var_os("DRAMATIS_CAPTURE").is_some() {
        std::env::var("DRAMATIS_CAPTURE_SIZE")
            .ok()
            .and_then(|s| {
                let (w, h) = s.split_once('x')?;
                Some((w.parse().ok()?, h.parse().ok()?))
            })
            .unwrap_or((860.0, 720.0))
    } else {
        (860.0, 720.0)
    };
    let endpoint = dramatis_app::endpoint();
    run(
        HostOptions {
            title: "Dramatis".into(),
            initial_logical_size,
            ..Default::default()
        },
        move |_, _, wake| {
            *slot.borrow_mut() = Some(Worker::spawn(endpoint, wake.callback()));
            Init {
                state: State::default(),
                logic: root as Logic,
                sheet: SHEET.into(),
                fonts: vec![],
                images: vec![],
            }
        },
        hooks,
    )
    .expect("Dramatis window");
}
