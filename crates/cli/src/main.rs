// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

mod carrier;
mod protocol;

use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};
use clap::{ArgGroup, Parser, Subcommand, ValueEnum};
use mere::{ContextCache, dataset::IndexedDataset, formats::RdfFormat};
use protocol::{Invitation, read_query, read_small};

#[derive(Parser)]
#[command(
    name = "mere",
    version,
    about = "Query linked data on this machine or an invited peer"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run a SPARQL query and print its rows as JSON arrays in column order.
    #[command(group(ArgGroup::new("source").args(["graph", "peer"]).required(true)))]
    Query {
        /// Local RDF dataset to query.
        #[arg(long)]
        graph: Option<PathBuf>,
        /// Private invitation file from the graph owner (selects the carrier).
        #[arg(long)]
        peer: Option<PathBuf>,
        /// UTF-8 SPARQL query file.
        #[arg(long)]
        query: PathBuf,
        /// Loopback I2P SAM bridge (default 127.0.0.1:7656).
        #[arg(long, requires = "peer")]
        sam: Option<SocketAddr>,
    },
    /// Serve a read-only graph snapshot to holders of a fresh invitation.
    Serve {
        #[arg(long)]
        graph: PathBuf,
        /// Query file the owner permits. Requests must match its contents.
        #[arg(long)]
        allow_query: PathBuf,
        /// Write a private invitation for this session to a new file.
        #[arg(long)]
        invite: PathBuf,
        #[arg(long, value_enum)]
        via: Carrier,
        /// Reticulum TCP interface: a reachable local IP address and port.
        #[arg(long)]
        listen: Option<SocketAddr>,
        /// Loopback I2P SAM bridge (default 127.0.0.1:7656).
        #[arg(long)]
        sam: Option<SocketAddr>,
    },
}

#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Carrier {
    Iroh,
    Reticulum,
    I2p,
}

fn dataset(path: &Path) -> Result<IndexedDataset> {
    let format = path
        .extension()
        .and_then(|s| s.to_str())
        .and_then(RdfFormat::from_extension)
        .context("expected an RDF file extension such as .trig, .ttl, .nq, or .jsonld")?;
    let bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    mere::formats::parse::parse(format, &bytes, None, ContextCache::full())
        .map_err(|e| anyhow::anyhow!("parse {}: {e}", path.display()))
}

#[tokio::main]
async fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Query {
            graph,
            peer,
            query,
            sam,
        } => {
            let query = read_query(&query)?;
            let rows = match (graph, peer) {
                (Some(path), None) => protocol::evaluate(&dataset(&path)?, &query)?,
                (None, Some(file)) => {
                    let invitation: Invitation = serde_json::from_slice(&read_small(&file, 65536)?)
                        .context("invalid peer invitation")?;
                    invitation.validate()?;
                    carrier::query(&invitation, &query, sam).await?
                },
                _ => unreachable!("clap requires exactly one source"),
            };
            serde_json::to_writer(std::io::stdout().lock(), &rows)?;
            println!();
        },
        Command::Serve {
            graph,
            allow_query,
            invite,
            via,
            listen,
            sam,
        } => {
            ensure!(!invite.exists(), "choose a new invitation file");
            let service = protocol::Service::new(dataset(&graph)?, read_query(&allow_query)?)?;
            tokio::select! {
                result = carrier::serve(via, listen, sam, &invite, &service) => result?,
                signal = tokio::signal::ctrl_c() => signal?,
            }
        },
    }
    Ok(())
}
