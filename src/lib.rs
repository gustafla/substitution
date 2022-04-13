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
mod bitset;
mod trie;

use rand::prelude::*;
use std::io::BufRead;
use std::num::NonZeroU8;

/// The range of ASCII lowercase letters that will be used in dictionary
const START: u8 = b'a';
const END: u8 = b'z';
const R: trie::AlphabetSize = START.abs_diff(END) as trie::AlphabetSize + 1;

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

static ENGLISH_FREQ_ORDER: [u8; R] = [
    b'e', b't', b'a', b'o', b'n', b'i', b'h', b's', b'r', b'd', b'l', b'u', b'w', b'm', b'c', b'f',
    b'g', b'y', b'p', b'b', b'k', b'v', b'j', b'x', b'q', b'z',
];

struct Key {
    table: [Option<NonZeroU8>; R],
    started_from: [u8; R],
    input_frequency_order: [u8; R],
}

impl Key {
    fn new(words: &[&[u8]]) -> Self {
        let mut freqs = [0; R];
        for word in words {
            for cchar in *word {
                freqs[usize::from(*cchar) - usize::from(START)] += 1;
            }
        }
        let mut freqs: Vec<(u8, usize)> = (0u8..)
            .zip(freqs.iter())
            .map(|(i, n)| (START + i, *n))
            .collect();
        freqs.sort_unstable_by_key(|e| std::cmp::Reverse(e.1));
        let mut input_frequency_order = [0u8; R];
        for (i, c) in freqs.iter().enumerate() {
            input_frequency_order[i] = c.0;
        }
        Self {
            table: [None; R],
            started_from: [0; R],
            input_frequency_order,
        }
    }

    fn index(input: u8) -> usize {
        usize::from(input - START)
    }

    fn attach(&mut self, input: u8, guess: NonZeroU8) -> Result<(), ()> {
        for value in self.table.into_iter().flatten() {
            if value == guess {
                return Err(());
            }
        }
        let idx = Self::index(input);
        if self.table[idx].replace(guess) == None {
            self.started_from[idx] = guess.get();
        }
        Ok(())
    }

    fn next_in_freq_order(start_guess: u8, current_guess: u8) -> Option<NonZeroU8> {
        use std::cmp::Ordering;
        let start_idx = ENGLISH_FREQ_ORDER
            .iter()
            .enumerate()
            .find(|(_, c)| **c == start_guess)
            .unwrap()
            .0;
        let current_idx = ENGLISH_FREQ_ORDER
            .iter()
            .enumerate()
            .find(|(_, c)| **c == current_guess)
            .unwrap()
            .0;
        let diff = start_idx.abs_diff(current_idx);
        let lower = (diff < start_idx).then(|| start_idx - diff - 1);
        let higher = (start_idx + diff < R).then(|| start_idx + diff);
        let idx = match (current_idx.cmp(&start_idx), lower, higher) {
            (Ordering::Less, _, Some(idx))
            | (Ordering::Less, Some(idx), None)
            | (Ordering::Equal | Ordering::Greater, Some(idx), _) => idx,
            (Ordering::Equal | Ordering::Greater, None, Some(idx)) => {
                if idx + 1 < R {
                    idx + 1
                } else {
                    return None;
                }
            }
            _ => return None,
        };
        NonZeroU8::new(ENGLISH_FREQ_ORDER[idx])
    }

    fn guess_first(&mut self, input: u8) -> Result<(), ()> {
        let idx = Self::index(input);
        if self.table[idx] == None {
            let freq_index = self
                .input_frequency_order
                .iter()
                .enumerate()
                .find(|(_, c)| **c == input)
                .unwrap()
                .0;
            let first_guess = NonZeroU8::new(ENGLISH_FREQ_ORDER[freq_index]).unwrap();
            if self.attach(input, first_guess).is_ok() {
                return Ok(());
            }
            self.attach_next(input, first_guess.get())
        } else {
            Ok(())
        }
    }

    fn attach_next(&mut self, input: u8, mut current_guess: u8) -> Result<(), ()> {
        let idx = Self::index(input);
        while {
            if let Some(guess) = Self::next_in_freq_order(self.started_from[idx], current_guess) {
                current_guess = guess.get();
            } else {
                return Err(());
            }
            self.attach(input, NonZeroU8::new(current_guess).unwrap())
                .is_err()
        } {}
        Ok(())
    }

    fn guess_again(&mut self, input: u8) -> Result<(), ()> {
        if let Some(current_guess) = self.table[Self::index(input)] {
            self.attach_next(input, current_guess.get())
        } else {
            Ok(())
        }
    }

    fn translate(&self, word: &[u8]) -> Vec<u8> {
        word.into_iter()
            .map(|c| match self.table[usize::from(*c - START)] {
                Some(c) => c.get(),
                None => *c,
            })
            .collect()
    }
}

fn validate(words: &[&[u8]], dict: &trie::Set<R>) -> f32 {
    let mut score = 0.;
    for word in words {
        score += dict.prefix_score(&bytes_to_key(word)) as f32 / (word.len() + 1) as f32;
    }
    score / words.len() as f32
}

/// Deciphers the string provided from CLI using statistics about english language.
pub fn decrypt(input: &str, dict: impl BufRead) -> Result<String, std::io::Error> {
    // Create a dictionary of words
    let dict = load_dict(dict)?;

    // Create a list of word slices
    let input = filter_input(input);
    let words: Vec<&[u8]> = input.split(u8::is_ascii_whitespace).collect();
    let mut key = Key::new(&words);
    let mut itr_bs = bitset::U64BitSet::<4 /* 4 * 64 = 256 */>::new();

    for word in words {
        itr_bs.clear();
        for c in word {
            // maybe unnecessary
            if !itr_bs.contains(*c) {
                key.guess_first(*c).unwrap();
                itr_bs.insert(*c);
            }
        }
        'retries: for c in word {
            loop {
                let word = key.translate(word);
                if validate(&[&word], &dict) > 0.7 {
                    println!("Found likely word \"{}\"", String::from_utf8_lossy(&word));
                    break 'retries;
                }
                if key.guess_again(*c).is_err() {
                    break;
                }
            }
        }
        println!("Moving on");
    }

    unimplemented!()
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
        let input: String = "Hello world!".into();
        let encrypted = encrypt(&input);
        dbg!(&input);
        dbg!(&encrypted);
        let decrypted = decrypt(
            &encrypted,
            std::io::BufReader::new(b"hello\nworld\n".as_slice()),
        )
        .unwrap();
        dbg!(&decrypted);
        assert_eq!(decrypted.len(), input.len() - 1);
    }

    #[test]
    fn key_input_frequency_order() {
        let input = filter_input("aaaaa bbvvvbb oo e");
        let words: Vec<&[u8]> = input.split(u8::is_ascii_whitespace).collect();
        let key = Key::new(&words);
        dbg!(key.input_frequency_order);
        assert!(matches!(
            key.input_frequency_order,
            [b'a', b'b', b'v', b'o', b'e', ..]
        ));
    }

    #[test]
    fn key_next_in_freq_order_covers_all_for_all() {
        for start_from in START..=END {
            let mut values_got = [0; R];
            let mut current = start_from;
            values_got[usize::from(current - START)] += 1;
            while let Some(next) = Key::next_in_freq_order(start_from, current) {
                current = next.get();
                println!("Got '{}'", char::from(current));
                values_got[usize::from(current - START)] += 1;
            }
            assert_eq!(values_got, [1; R]);
        }
    }

    fn assert_key_next_in_freq_order(start: u8, expected: &[u8]) {
        let mut current = NonZeroU8::new(start).unwrap();
        for chr in expected {
            match Key::next_in_freq_order(start, current.get()) {
                Some(val) => {
                    current = val;
                    println!("{} == {}", char::from(current.get()), char::from(*chr));
                    assert_eq!(current.get(), *chr);
                }
                None => {
                    assert_eq!(*chr, 0);
                }
            }
        }
    }

    #[test]
    fn key_next_in_freq_order_looks_correct() {
        assert_key_next_in_freq_order(b'a', b"toen");
        assert_key_next_in_freq_order(b'o', b"antiehsrd");
        assert_key_next_in_freq_order(b'b', b"pkyvgjfxcqmzwuldrshinoate\0");
    }
}
