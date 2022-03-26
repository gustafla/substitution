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
fn filter_input(input: String) -> Vec<u8> {
    input
        .chars()
        .filter_map(|c| {
            if c.is_ascii_alphabetic() || c.is_ascii_whitespace() {
                c.to_ascii_lowercase().try_into().ok()
            } else {
                None
            }
        })
        .collect()
}

/// Encrypt-subcommand and it's input argument from the CLI
#[derive(Parser, Clone)]
struct Encrypt {
    input: String,
}

impl Encrypt {
    /// Encrypts the string provided from CLI with a randomly generated substitution cipher.
    fn run(self) -> String {
        let input = filter_input(self.input);

        // Create a random substitution
        let mut cipher: Vec<u8> = (b'a'..=b'z').collect();
        let mut rng = rand::thread_rng();
        cipher.shuffle(&mut rng);

        // Encrypt
        let mut buf = String::with_capacity(input.len());
        for word in input.split(u8::is_ascii_whitespace) {
            for cchar in word.iter().map(|c| cipher[*c as usize - b'a' as usize]) {
                buf.push(cchar.into());
            }
            buf.push(' ');
        }

        // Remove trailing space. TODO, encrypt input in-place
        buf.pop();
        buf
    }
}

/// Decrypt-subcommand and it's input argument from the CLI
#[derive(Parser, Clone)]
struct Decrypt {
    input: String,
}

impl Decrypt {
    /// Deciphers the string provided from CLI using statistics about english language.
    fn run(self) -> String {
        let input = filter_input(self.input);

        static ENGLISH_FREQ_ORDER: [u8; 26] = [
            b'e', b't', b'a', b'o', b'n', b'i', b'h', b's', b'r', b'd', b'l', b'u', b'w', b'm',
            b'c', b'f', b'g', b'y', b'p', b'b', b'k', b'v', b'j', b'x', b'q', b'z',
        ];

        let mut freqs: Vec<usize> = vec![0; ('a'..='z').count()];
        for word in input.split(u8::is_ascii_whitespace) {
            for cchar in word {
                freqs[*cchar as usize - b'a' as usize] += 1;
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

        // Decrypt
        let mut buf = String::with_capacity(input.len());
        for word in input.split(u8::is_ascii_whitespace) {
            for cchar in word.iter().map(|c| decipher[*c as usize - b'a' as usize]) {
                buf.push(cchar.into());
            }
            buf.push(' ');
        }

        // Remove trailing space. TODO, decrypt input in-place
        buf.pop();
        buf
    }
}

/// Main command line argument structure
#[derive(Parser)]
#[clap(author, version, about)]
enum Opts {
    Encrypt(Encrypt),
    Decrypt(Decrypt),
}

#[cfg(not(tarpaulin_include))]
fn main() -> Result<()> {
    // Install color_eyre's panic- and error report handlers
    color_eyre::install()?;

    // Open, lock and buffer stdout
    let stdout = std::io::stdout();
    let mut stdout = std::io::BufWriter::new(stdout.lock());

    // Parse CLI arguments and run the subcommands
    let opts = Opts::parse();
    match opts {
        Opts::Decrypt(d) => writeln!(stdout, "{}", d.run())?,
        Opts::Encrypt(e) => writeln!(stdout, "{}", e.run())?,
    };
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashMap;
    use std::hash::Hash;

    #[test]
    fn filter_input_keeps_ascii_alphabetic_and_whitespace() {
        assert_eq!(filter_input("hello, world! 😊".into()), b"hello world ");
    }

    #[test]
    fn filter_input_transforms_to_lowercase() {
        assert_eq!(filter_input("Hello WORLD".into()), b"hello world");
    }

    #[test]
    fn encrypt_output_expected_lenght() {
        let e = Encrypt {
            input: "Moikka tiraprojekti!".into(),
        };
        dbg!(&e.input);
        let out = e.clone().run();
        dbg!(&out);
        assert_eq!(out.len(), e.input.len() - 1);
    }

    /// Counts how many times each possible value occurs in `of`.
    fn stats<'a, T: Eq + Hash>(stats: &mut HashMap<&'a T, usize>, of: impl Iterator<Item = &'a T>) {
        for c in of {
            *stats.entry(c).or_insert(0) += 1;
        }
    }

    /// Computes frequency profiles for both input and encrypted output, asserts that they match.
    fn assert_encrypt_expected_frequencies(input: String) {
        let mut input_freqs = HashMap::new();
        let mut output_freqs = HashMap::new();

        // Count stats about the input string
        let filtered_input = filter_input(input.clone());
        stats(&mut input_freqs, filtered_input.iter());
        dbg!(&input_freqs);

        // Encrypt the input
        let e = Encrypt { input };
        let out = e.run();

        // Count stats about the output string
        stats(&mut output_freqs, out.as_bytes().iter());
        dbg!(&output_freqs);

        // Count stats about input and output frequencies
        let mut input_stats = HashMap::new();
        let mut output_stats = HashMap::new();
        stats(&mut input_stats, input_freqs.values());
        stats(&mut output_stats, output_freqs.values());

        dbg!(&input_stats);
        dbg!(&output_stats);

        // Assert that character frequency profiles match
        assert_eq!(input_stats, output_stats);
    }

    #[test]
    fn encrypt_frequencies_simple() {
        assert_encrypt_expected_frequencies("Moikka tiraprojekti!".into());
        assert_encrypt_expected_frequencies("Hello World!".into());
        assert_encrypt_expected_frequencies("Returns a reference to the value corresponding to the key. The key may be any borrowed form of the map’s key type, but Hash and Eq on the borrowed form must match those for the key type.".into());
        assert_encrypt_expected_frequencies("Inserts a key-value pair into the map. If the map did not have this key present, None is returned. If the map did have this key present, the value is updated, and the old value is returned. The key is not updated, though; this matters for types that can be == without being identical. See the module-level documentation for more.".into());
    }

    #[test]
    fn encrypt_frequencies_random() {
        let input: Vec<u8> = (0..100000)
            .map(|_| rand::thread_rng().gen_range(b' '..=b'~'))
            .collect();
        dbg!(&input);
        assert_encrypt_expected_frequencies(String::from_utf8(input).unwrap());
    }

    #[test]
    fn decrypt_expected_lenght() {
        let e = Encrypt {
            input: "Moikka tiraprojekti!".into(),
        };
        let d = Decrypt {
            input: e.clone().run(),
        };
        dbg!(&e.input);
        dbg!(&d.input);
        let out = d.run();
        dbg!(&out);
        assert_eq!(out.len(), e.input.len() - 1);
    }
}
