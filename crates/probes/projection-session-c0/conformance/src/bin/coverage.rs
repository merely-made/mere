//! How much WIT our generator binds: every interface in the WIT directories
//! given as arguments, then interfaces from random `wit-smith` packages.
//! An interface counts as bound when the generator emits code for it; the
//! first feature it refuses is tallied otherwise.
//!
//! `coverage <wit dir>... [--smith N]`

use std::collections::BTreeMap;

use wit_parser::{Resolve, decoding::DecodedWasm};

#[derive(Default)]
struct Tally {
    bound: usize,
    total: usize,
    refusals: BTreeMap<String, usize>,
}

impl Tally {
    fn interfaces(&mut self, resolve: &Resolve, packages: &[wit_parser::PackageId], show: bool) {
        for &p in packages {
            let pkg = &resolve.packages[p];
            for (name, &id) in &pkg.interfaces {
                self.total += 1;
                match c0_witgen::generate_interface(resolve, id) {
                    Ok(_) => {
                        self.bound += 1;
                        if show {
                            println!("  bound    {}/{name}", pkg.name);
                        }
                    },
                    Err(e) => {
                        let reason = reason(&e.to_string());
                        if show {
                            println!("  refused  {}/{name}: {reason}", pkg.name);
                        }
                        *self.refusals.entry(reason).or_default() += 1;
                    },
                }
            }
        }
    }

    fn report(&self, title: &str) {
        println!(
            "{title}: {} of {} interfaces bound ({:.0}%)",
            self.bound,
            self.total,
            100.0 * self.bound as f64 / self.total.max(1) as f64
        );
        let mut by_count: Vec<_> = self.refusals.iter().collect();
        by_count.sort_by(|a, b| b.1.cmp(a.1));
        for (reason, n) in by_count {
            println!("  {n:5}  {reason}");
        }
    }
}

/// The refused feature, without the parser's dump of it.
fn reason(message: &str) -> String {
    let tail = message.split("`: ").nth(1).unwrap_or(message);
    let tail = tail
        .strip_prefix("unsupported type definition ")
        .unwrap_or(tail);
    let cut = tail.find(['(', '{', '`']).unwrap_or(tail.len());
    let head = tail[..cut].trim();
    if message.contains("type definition") {
        format!("type definition {head}")
    } else {
        head.to_string()
    }
}

fn main() -> anyhow::Result<()> {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let smith = match args.iter().position(|a| a == "--smith") {
        Some(i) => {
            let n = args[i + 1].parse()?;
            args.drain(i..=i + 1);
            n
        },
        None => 0,
    };

    let mut corpus = Tally::default();
    for dir in &args {
        let mut resolve = Resolve::default();
        let (main, _) = resolve.push_path(dir)?;
        println!("{dir}");
        let packages: Vec<_> = resolve.packages.iter().map(|(id, _)| id).collect();
        let _ = main;
        corpus.interfaces(&resolve, &packages, true);
    }
    if !args.is_empty() {
        corpus.report("real WIT");
    }

    if smith > 0 {
        let mut random = Tally::default();
        let mut state = 0x9e37_79b9_7f4a_7c15u64;
        let mut generated = 0;
        for _ in 0..smith {
            let bytes: Vec<u8> = (0..8192)
                .map(|_| {
                    state ^= state << 13;
                    state ^= state >> 7;
                    state ^= state << 17;
                    state as u8
                })
                .collect();
            let mut u = arbitrary::Unstructured::new(&bytes);
            let Ok(wasm) = wit_smith::smith(&wit_smith::Config::default(), &mut u) else {
                continue;
            };
            let Ok(DecodedWasm::WitPackage(resolve, package)) = wit_parser::decoding::decode(&wasm)
            else {
                continue;
            };
            generated += 1;
            random.interfaces(&resolve, &[package], false);
        }
        random.report(&format!("wit-smith, {generated} packages"));
    }
    Ok(())
}
