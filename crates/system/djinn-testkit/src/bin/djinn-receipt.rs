// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! `djinn-receipt verify <run-dir>`: recompute a record's evidence hashes.
//! `djinn-receipt summary <run-dir>`: print its Markdown summary.

use std::path::PathBuf;
use std::process::ExitCode;

use djinn_testkit::receipt::{RECEIPT_FILE, Receipt, summary_markdown, verify};

const USAGE: &str = "usage: djinn-receipt verify|summary <run-dir>";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [command, dir] = args.as_slice() else {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    };
    let dir = PathBuf::from(dir);
    match command.as_str() {
        "verify" => match verify(&dir) {
            Ok(checked) if checked.ok() => {
                println!(
                    "verified {} evidence files in {}",
                    checked.checked,
                    checked.receipt.display()
                );
                ExitCode::SUCCESS
            },
            Ok(checked) => {
                for problem in &checked.problems {
                    println!("MISMATCH {problem}");
                }
                ExitCode::FAILURE
            },
            Err(error) => {
                eprintln!("djinn-receipt: {error}");
                ExitCode::FAILURE
            },
        },
        "summary" => {
            let read = std::fs::read(dir.join(RECEIPT_FILE))
                .map_err(|error| error.to_string())
                .and_then(|bytes| {
                    serde_json::from_slice::<Receipt>(&bytes).map_err(|error| error.to_string())
                });
            match read {
                Ok(receipt) => {
                    print!("{}", summary_markdown(&receipt));
                    ExitCode::SUCCESS
                },
                Err(error) => {
                    eprintln!("djinn-receipt: {error}");
                    ExitCode::FAILURE
                },
            }
        },
        _ => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        },
    }
}
