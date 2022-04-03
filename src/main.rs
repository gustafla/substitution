//! CLI program to decipher substitution ciphers.
//!
//! Run with --help for usage and options.

// Forbid unsafe code (https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html)
#![forbid(unsafe_code)]
// Error from most clippy warnings (https://github.com/rust-lang/rust-clippy)
#![deny(clippy::all)]
// Warnings from pedantic clippy lints
#![warn(clippy::pedantic)]
// Don't measure the CLI binary's coverage in tarpaulin
#![cfg(not(tarpaulin_include))]

// "Include" src/io.rs in the main CLI here
mod io;

use clap::Parser;
use color_eyre::{
    eyre::{eyre, Context},
    Result,
};
use std::{
    io::{BufReader, BufWriter, Read, Write},
    path::PathBuf,
};

/// Main command line argument structure
#[derive(Parser)]
#[clap(author, version, about)]
#[clap(propagate_version = true)]
struct Cli {
    /// Overwrite the contents of the input file
    #[clap(long, short)]
    in_place: bool,
    /// File to write output to. Defaults to stdout if omitted
    #[clap(long, short)]
    output_file: Option<PathBuf>,
    /// Perform encrypt or decrypt
    mode: Mode,
    /// File to read as input. Defaults to stdin if omitted
    path: Option<PathBuf>,
}

/// Modes that the program can run in
enum Mode {
    /// Encrypt the input with a randomly generated key
    Encrypt,
    /// Decipher the input without a key
    Decrypt,
}

/// String value conversion for modes
impl std::str::FromStr for Mode {
    /// Returns pretty-printable error messages when input string is unknown
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Convert input to lowercase to be case-insensitive
        match s.to_ascii_lowercase().as_ref() {
            "encrypt" | "e" => Ok(Self::Encrypt),
            "decrypt" | "d" => Ok(Self::Decrypt),
            _ => Err(eyre!(
                "Unknown mode.\nTry one of 'e', 'encrypt', 'd', 'decrypt'."
            )),
        }
    }
}

fn main() -> Result<()> {
    // Install color_eyre's panic- and error report handlers
    color_eyre::install()?;

    // Parse CLI arguments and read the input
    let opts = Cli::parse();
    let mut text = String::with_capacity(4096);
    let mut input: io::Input = opts.path.clone().try_into()?;
    BufReader::new(input.as_mut())
        .read_to_string(&mut text)
        .wrap_err(format!("Cannot read from {}", input))?;

    // Open output
    let mut output: io::Output = opts.path.try_into()?;
    {
        // Buffer output to improve performance
        let mut output = BufWriter::new(output.as_mut());

        // Run and write the result out
        writeln!(
            output,
            "{}",
            match opts.mode {
                Mode::Decrypt => substitution::decrypt(&text),
                Mode::Encrypt => substitution::encrypt(&text),
            }
        )
    }
    .wrap_err(format!("Cannot write to {}", output))?;

    Ok(())
}
