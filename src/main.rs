//! CLI program to decipher substitution ciphers.
//!
//! Run with --help for usage and options.

#![deny(rustdoc::all)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]

use clap::Parser;
use color_eyre::Result;
use rand::prelude::*;
use std::io::Write;

/// Substitutes uppercase ASCII alphabetic (A-Z) characters with lowercase equivalents.
/// Leaves out all other characters than ASCII alphabetic and whitespace.
fn normalize_input(input: String) -> String {
    input
        .chars()
        .filter_map(|c| {
            if c.is_ascii_alphabetic() || c.is_ascii_whitespace() {
                Some(c.to_ascii_lowercase())
            } else {
                None
            }
        })
        .collect()
}

#[derive(Parser)]
struct Encrypt {
    input: String,
}

impl Encrypt {
    fn run(self) -> Result<()> {
        let input = normalize_input(self.input);

        // Create a random substitution
        let mut cipher: Vec<u8> = (b'a'..=b'z').collect();
        let mut rng = rand::thread_rng();
        cipher.shuffle(&mut rng);

        // Open, lock and buffer stdout
        let stdout = std::io::stdout();
        let mut stdout = std::io::BufWriter::new(stdout.lock());

        // Encrypt
        for word in input.split_ascii_whitespace() {
            for cchar in word.chars().map(|c| cipher[c as usize - b'a' as usize]) {
                stdout.write_all(&[cchar])?;
            }
            stdout.write_all(&[b' '])?;
        }
        stdout.write_all(&[b'\n'])?;

        Ok(())
    }
}

#[derive(Parser)]
struct Decrypt {
    input: String,
}

impl Decrypt {
    fn run(self) -> Result<()> {
        let input = normalize_input(self.input);

        static ENGLISH_FREQ_ORDER: [u8; 26] = [
            b'e', b't', b'a', b'o', b'n', b'i', b'h', b's', b'r', b'd', b'l', b'u', b'w', b'm',
            b'c', b'f', b'g', b'y', b'p', b'b', b'k', b'v', b'j', b'x', b'q', b'z',
        ];

        let mut freqs: Vec<usize> = vec![0; ('a'..='z').count()];
        for word in input.split_ascii_whitespace() {
            for cchar in word.chars() {
                freqs[cchar as usize - b'a' as usize] += 1;
            }
        }
        let mut freqs: Vec<(u8, usize)> = freqs
            .iter()
            .enumerate()
            .map(|(i, n)| (b'a' + i as u8, *n))
            .collect();
        freqs.sort_unstable_by_key(|e| e.1);

        let mut decipher: Vec<u8> = vec![0; 26];
        for i in 0..freqs.len() {
            decipher[(ENGLISH_FREQ_ORDER[i] - b'a') as usize] = freqs[i].0;
        }

        // Open, lock and buffer stdout
        let stdout = std::io::stdout();
        let mut stdout = std::io::BufWriter::new(stdout.lock());

        // Decrypt
        for word in input.split_ascii_whitespace() {
            for cchar in word.chars().map(|c| decipher[c as usize - b'a' as usize]) {
                stdout.write_all(&[cchar])?;
            }
            stdout.write_all(&[b' '])?;
        }
        stdout.write_all(&[b'\n'])?;

        Ok(())
    }
}

#[derive(Parser)]
#[clap(author, version, about)]
enum Opts {
    Encrypt(Encrypt),
    Decrypt(Decrypt),
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let opts = Opts::parse();
    match opts {
        Opts::Decrypt(d) => d.run()?,
        Opts::Encrypt(e) => e.run()?,
    };
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn normalize_input_keeps_ascii_alphabetic_and_whitespace() {
        assert_eq!(normalize_input("hello, world! 😊".into()), "hello world ");
    }

    #[test]
    fn normalize_input_transforms_to_lowercase() {
        assert_eq!(normalize_input("Hello WORLD".into()), "hello world");
    }
}
