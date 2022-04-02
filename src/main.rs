//! CLI program to decipher substitution ciphers.
//!
//! Run with --help for usage and options.

// Forbid unsafe code (https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html)
#![forbid(unsafe_code)]
// Disallow all missing docs and missing code examples for public items
#![deny(rustdoc::all)]
// Error from most clippy warnings (https://github.com/rust-lang/rust-clippy)
#![deny(clippy::all)]
// Warnings from pedantic clippy lints
#![warn(clippy::pedantic)]
// Warnings about missing Cargo.toml fields
#![warn(clippy::cargo)]
// More about lint levels https://doc.rust-lang.org/rustc/lints/levels.html

use clap::{Parser, Subcommand};
use color_eyre::Result;
use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    path::PathBuf,
};

/// Main command line argument structure
#[derive(Parser)]
#[clap(author, version, about)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
    /// File to use as input. Defaults to stdin if omitted
    path: Option<PathBuf>,
}

/// Subcommands that the user can run
#[derive(Subcommand)]
enum Command {
    /// Encrypt the input with a randomly generated key
    Encrypt,
    /// Decipher the input without a key
    Decrypt,
}

#[cfg(not(tarpaulin_include))]
fn main() -> Result<()> {
    // Install color_eyre's panic- and error report handlers
    color_eyre::install()?;

    // Parse CLI arguments and read the file
    let opts = Cli::parse();
    let content: Box<dyn Read> = match opts.path {
        Some(path) => Box::new(File::open(path)?),
        None => Box::new(std::io::stdin()),
    };
    let mut input = String::with_capacity(4096);
    BufReader::new(content).read_to_string(&mut input)?;

    // Open, lock and buffer stdout
    let stdout = std::io::stdout();
    let mut stdout = BufWriter::new(stdout.lock());

    // Run subcommand and write the result to stdout
    writeln!(
        stdout,
        "{}",
        match opts.command {
            Command::Decrypt => substitution::decrypt(&input),
            Command::Encrypt => substitution::encrypt(&input),
        }
    )?;

    Ok(())
}
