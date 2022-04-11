//! Library for working with and reversing substitution ciphers.

// Forbid unsafe code (https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html)
#![forbid(unsafe_code)]
// Disallow all missing docs and rustdoc lints
#![deny(missing_docs)]
#![deny(rustdoc::all)]
// Error from most clippy warnings (https://github.com/rust-lang/rust-clippy)
#![deny(clippy::all)]
// Warnings from pedantic clippy lints
#![warn(clippy::pedantic)]
// Warnings about missing Cargo.toml fields
#![warn(clippy::cargo)]
// More about lint levels https://doc.rust-lang.org/rustc/lints/levels.html

// "Include" trie.rs
mod trie;

use rand::prelude::*;
use std::io::BufRead;

/// The range of ASCII lowercase letters that will be used in dictionary
const START: u8 = b'a';
const END: u8 = b'z';
const R: trie::AlphabetSize = START.abs_diff(END + 1) as trie::AlphabetSize;

/// Substitutes uppercase ASCII alphabetic (A-Z) characters with lowercase equivalents.
/// Leaves out all other characters than ASCII alphabetic and whitespace.
fn filter_input(input: &str) -> Vec<u8> {
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

/// Encrypts the string provided from CLI with a randomly generated substitution cipher.
#[must_use]
pub fn encrypt(input: &str) -> String {
    let input = filter_input(input);

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

/// Convert bytes from START..=END to indices in 0..R, discarding bytes which aren't in range
fn bytes_to_key(slice: &[u8]) -> trie::Key<R, u8> {
    let buf: Vec<u8> = slice
        .iter()
        .filter_map(|b| b.checked_sub(START))
        .filter(|b| *b <= END)
        .collect();
    trie::Key::<R, u8>::try_from(buf).unwrap()
}

/// Read through a dictionary file and insert every word in a trie set
fn load_dict(from: impl BufRead) -> Result<trie::Set<R>, std::io::Error> {
    let mut dict = trie::Set::<R>::new();
    for word in from.lines() {
        let bytes = filter_input(&word?);
        let key = bytes_to_key(&bytes);
        dict.insert(&key);
    }
    Ok(dict)
}

/// Deciphers the string provided from CLI using statistics about english language.
pub fn decrypt(input: &str, dict: impl BufRead) -> Result<String, std::io::Error> {
    static ENGLISH_FREQ_ORDER: [u8; 26] = [
        b'e', b't', b'a', b'o', b'n', b'i', b'h', b's', b'r', b'd', b'l', b'u', b'w', b'm', b'c',
        b'f', b'g', b'y', b'p', b'b', b'k', b'v', b'j', b'x', b'q', b'z',
    ];

    // Create a dictionary of words
    let dict = load_dict(dict)?;

    let input = filter_input(input);

    let mut freqs: Vec<usize> = vec![0; ('a'..='z').count()];
    for word in input.split(u8::is_ascii_whitespace) {
        for cchar in word {
            freqs[*cchar as usize - b'a' as usize] += 1;
        }
    }
    let mut freqs: Vec<(u8, usize)> = (0u8..)
        .zip(freqs.iter())
        .map(|(i, n)| (b'a' + i, *n))
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
    Ok(buf)
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
        let input: String = "Moikka tiraprojekti!".into();
        dbg!(&input);
        let out = encrypt(&input);
        dbg!(&out);
        assert_eq!(out.len(), input.len() - 1);
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
        let filtered_input = filter_input(&input);
        stats(&mut input_freqs, filtered_input.iter());
        dbg!(&input_freqs);

        // Encrypt the input
        let out = encrypt(&input);

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
        let input: String = "Moikka tiraprojekti!".into();
        let encrypted = encrypt(&input);
        dbg!(&input);
        dbg!(&encrypted);
        let decrypted = decrypt(&encrypted, std::io::BufReader::new(b"hello".as_slice())).unwrap();
        dbg!(&decrypted);
        assert_eq!(decrypted.len(), input.len() - 1);
    }
}
