// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One opaque recovered signed record offered to the existing mesh owner.

mod campaign;
mod carrier;
mod identity;
mod peer;

use std::io::Write;
use std::path::PathBuf;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, Error>;

fn main() {
    if let Err(error) = execute() {
        eprintln!("retained_receive: {error}");
        std::process::exit(2);
    }
}

fn execute() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let mut peer_path = None;
    let mut output = None;
    let mut quic = false;
    while let Some(argument) = args.next() {
        match argument.to_str() {
            Some("--retinue-peer") => {
                if peer_path.is_some() {
                    return Err("duplicate --retinue-peer".into());
                }
                peer_path = Some(PathBuf::from(args.next().ok_or("missing peer path")?));
            },
            Some("--output") => {
                if output.is_some() {
                    return Err("duplicate --output".into());
                }
                output = Some(PathBuf::from(args.next().ok_or("missing output path")?));
            },
            Some("--carrier") => {
                if quic || args.next().as_deref() != Some(std::ffi::OsStr::new("quic")) {
                    return Err("--carrier requires quic and may appear once".into());
                }
                quic = true;
            },
            Some("--help" | "-h") => {
                println!(
                    "retained_receive --retinue-peer PATH [--carrier quic] [--output NEW_PATH]"
                );
                return Ok(());
            },
            _ => return Err(format!("unknown argument: {}", argument.to_string_lossy()).into()),
        }
    }
    let peer_path = peer_path.ok_or("--retinue-peer PATH is required")?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let report = runtime.block_on(campaign::run(&peer_path, quic))?;
    let bytes = serde_json::to_vec_pretty(&report)?;
    if let Some(path) = output {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        file.write_all(&bytes)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        eprintln!("retained_receive: wrote {}", path.display());
    } else {
        let mut stdout = std::io::stdout().lock();
        stdout.write_all(&bytes)?;
        stdout.write_all(b"\n")?;
    }
    Ok(())
}

fn require(condition: bool, message: &str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(message.to_owned().into())
    }
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut value = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(value, "{byte:02x}").expect("writing a String cannot fail");
    }
    value
}

fn unhex(value: &str, maximum: usize) -> Result<Vec<u8>> {
    require(
        value.len() <= maximum * 2 && value.len().is_multiple_of(2),
        "hex length budget",
    )?;
    let nibble = |byte| match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => Err("expected lowercase hexadecimal"),
    };
    value
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| Ok(nibble(pair[0])? * 16 + nibble(pair[1])?))
        .collect()
}
