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
// This library should not panic so docs about it aren't required
#![allow(clippy::missing_panics_doc)]
// Warnings about missing Cargo.toml fields
#![warn(clippy::cargo)]
// More about lint levels https://doc.rust-lang.org/rustc/lints/levels.html

// "Include" trie.rs
mod bitset;
mod trie;

use rand::prelude::*;
use std::io::BufRead;

/// The range of ASCII lowercase letters that will be used in dictionary
const START: u8 = b'a';
const END: u8 = b'z';
const R: trie::AlphabetSize = START.abs_diff(END) as trie::AlphabetSize + 1;

/// Key that stores details about an encryption or decryption process
struct Key {
    table: [u8; R],
    started_from: [u8; R],
    input_freq_index: [usize; R],
    lang_freq_index: [usize; R],
    lang_freq_order: [u8; R],
    guesses: bitset::BitSet64<4>,
}

impl Key {
    /// Create a random substitution key that can be used to encrypt a plaintext input
    fn random() -> Self {
        let mut table: Vec<u8> = (START..=END).collect();
        let mut rng = rand::thread_rng();
        table.shuffle(&mut rng);
        Self {
            table: table.try_into().unwrap(),
            started_from: [0; R],
            input_freq_index: [0; R],
            lang_freq_index: [0; R],
            lang_freq_order: [0; R],
            guesses: bitset::BitSet64::<4>::new(),
        }
    }

    /// Create an uninitialized substitution key that can be used to search for the correct key during decryption
    fn new(input: &[u8], lang_freq_order: [u8; R]) -> Self {
        // Count input characters
        let mut freqs = [0; R];
        for chr in input.iter().filter(|c| c.is_ascii_alphabetic()) {
            freqs[usize::from(*chr - START)] += 1;
        }
        // Sort by frequency
        let mut freqs: Vec<(u8, &usize)> = (START..).zip(freqs.iter()).collect();
        freqs.sort_unstable_by_key(|e| std::cmp::Reverse(e.1));
        // Create a table for each input character's frequency index
        let mut input_freq_index = [0; R];
        for chr in START..=END {
            let idx = freqs
                .iter()
                .enumerate()
                .find(|(_, (c, _))| *c == chr)
                .unwrap()
                .0;
            input_freq_index[usize::from(chr - START)] = idx;
        }

        // Create a table for each language character's frequency index
        let mut lang_freq_index = [0; R];
        for chr in START..=END {
            let idx = lang_freq_order
                .iter()
                .enumerate()
                .find(|(_, c)| **c == chr)
                .unwrap()
                .0;
            lang_freq_index[usize::from(chr - START)] = idx;
        }

        Self {
            table: [0; R],
            started_from: [0; R],
            input_freq_index,
            lang_freq_index,
            lang_freq_order,
            guesses: bitset::BitSet64::<4>::new(),
        }
    }

    /// ASCII character's table lookup index
    fn index(input: u8) -> usize {
        usize::from(input - START)
    }

    /// Set a guess for a given input character
    fn attach(&mut self, input: u8, guess: u8) -> Result<(), ()> {
        if self.guesses.contains(guess) {
            return Err(());
        }
        let idx = Self::index(input);
        self.guesses.remove(self.table[idx]);
        if self.table[idx] == 0 {
            self.started_from[idx] = guess;
        }
        self.table[idx] = guess;
        self.guesses.insert(guess);
        Ok(())
    }

    /// Get the next character in language frequency order
    fn next_in_freq_order(&self, start_guess: u8, current_guess: u8) -> u8 {
        use std::cmp::Ordering;
        let start_idx = self.lang_freq_index[usize::from(start_guess - START)];
        let current_idx = self.lang_freq_index[usize::from(current_guess - START)];
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
                    return 0;
                }
            }
            _ => return 0,
        };
        self.lang_freq_order[idx]
    }

    /// Set a next guess in language frequency order for input character
    fn attach_next(&mut self, input: u8) -> Result<(), ()> {
        let idx = Self::index(input);

        let (start_guess, mut current_guess) = match self.table[idx] {
            0 => {
                let freq_index = self.input_freq_index[usize::from(input - START)];
                let first_guess = self.lang_freq_order[freq_index];
                if self.attach(input, first_guess).is_ok() {
                    return Ok(());
                }
                (first_guess, first_guess)
            }
            current_guess => (self.started_from[idx], current_guess),
        };

        while {
            current_guess = self.next_in_freq_order(start_guess, current_guess);
            if current_guess == 0 {
                return Err(());
            }
            self.attach(input, current_guess).is_err()
        } {}
        Ok(())
    }

    /// Remove the current guess from a given input character
    fn clear(&mut self, input: u8) {
        let idx = Self::index(input);
        self.guesses.remove(self.table[idx]);
        self.table[idx] = 0;
    }

    /// Replace characters in text according to current key state.
    /// In other words, perform the substitution. Encrypt or decrypt.
    fn translate(&self, text: &mut [u8]) {
        for c in text {
            if c.is_ascii_alphabetic() {
                let translation = self.table[Self::index(*c)];
                if translation != 0 {
                    *c = translation;
                }
            }
        }
    }
}

/// Substitutes uppercase ASCII alphabetic (A-Z) characters with lowercase equivalents.
/// Replaces all other characters than ASCII alphabetic and whitespace with spaces.
fn filter_input(input: &str) -> Vec<u8> {
    input
        .chars()
        .map(|c| {
            if c.is_ascii_alphabetic() || c.is_ascii_whitespace() {
                c.to_ascii_lowercase().try_into().unwrap_or(b' ')
            } else {
                b' '
            }
        })
        .collect()
}

/// Encrypts the string provided from CLI with a randomly generated substitution cipher.
#[must_use]
pub fn encrypt(input: &str) -> String {
    let mut input = filter_input(input);

    // Create a random substitution
    let key = Key::random();

    // Encrypt
    key.translate(&mut input);

    String::from_utf8(input).unwrap()
}

/// Test if text matches well with the dictionary.
/// Returns a tuple `(given_score, max_score)`.
fn validate(text: &[u8], dict: &trie::Set<R, { START as usize }>) -> (usize, usize) {
    let mut score = 0;
    let mut max = 0;
    for word in text
        .split(u8::is_ascii_whitespace)
        .filter(|w| !w.is_empty())
    {
        score += dict.prefix_score(word).unwrap();
        max += word.len() + 1;
    }
    (score, max)
}

/// Returns a list of all unique alphabetic characters in input.
fn unique_chars(input: &[u8]) -> Vec<u8> {
    let mut uc = Vec::with_capacity(16);
    let mut set = bitset::BitSet64::<4>::new();
    for c in input {
        if c.is_ascii_alphabetic() && !set.contains(*c) {
            uc.push(*c);
            set.insert(*c);
        }
    }
    uc
}

/// Find the next word
fn seek_word(input: &[u8]) -> &[u8] {
    let mut trim_start = true;
    for i in 0..input.len() {
        if input[i].is_ascii_whitespace() {
            if trim_start {
                continue;
            }
            return &input[..i];
        }
        trim_start = false;
    }
    input
}

/// Recursive backtracking deciphering word by word
fn decrypt_words(
    input: &[u8],
    output: &mut Vec<u8>,
    key: &mut Key,
    chars_set: &mut bitset::BitSet64<1>,
    dict: &trie::Set<R, { START as usize }>,
) -> Result<(), ()> {
    // Find 1 word
    let in_word = seek_word(input);

    // Happy path end for recursion
    if in_word.is_empty() {
        return Ok(());
    }

    // Reserve translation scratch area in output
    let out_len = output.len();
    output.extend(in_word);

    // Generate list of currently relevant and unset chars
    let free_chars: Vec<u8> = unique_chars(in_word)
        .into_iter()
        .filter(|c| !chars_set.contains(c - START))
        .collect();

    // Set input chars in stone for next round so they won't be iterated
    free_chars.iter().for_each(|c| chars_set.insert(*c - START));

    'test: loop {
        key.translate(&mut output[out_len..]);

        let (score, max) = validate(&output[out_len..], dict);
        let treshold = match max {
            0..=4 => max,
            _ => max - 1,
        };
        if score >= treshold {
            /*println!(
                "Found likely words \"{}\"",
                String::from_utf8_lossy(&output[out_len..])
            );*/

            if decrypt_words(&input[in_word.len()..], output, key, chars_set, dict).is_ok() {
                return Ok(());
            }
        }

        // Reset translation buffer
        (&mut output[out_len..]).copy_from_slice(in_word);

        // Current key is wrong, try next
        // for chr in &free_chars {
        //     print!("{}", char::from(key.table[Key::index(*chr)]));
        // }
        // println!();
        for chr in &free_chars {
            match key.attach_next(*chr) {
                Ok(()) => continue 'test,
                Err(()) => {
                    key.clear(*chr);
                }
            }
        }
        break;
    }

    // Failed, so truncate scratch area off
    output.truncate(output.len() - in_word.len());

    // Clear set characters so that caller up in the stack can keep iterating it's key
    free_chars.iter().for_each(|c| chars_set.remove(*c - START));

    Err(())
}

/// Read through a dictionary file and insert every word in a trie set
fn load_dict(from: impl BufRead) -> Result<trie::Set<R, { START as usize }>, std::io::Error> {
    let mut dict = trie::Set::<R, { START as usize }>::new();
    for line in from.lines() {
        let bytes = filter_input(&line?);
        for word in bytes
            .split(u8::is_ascii_whitespace)
            .filter(|w| !w.is_empty())
        {
            dict.insert(word).unwrap();
        }
    }
    Ok(dict)
}

static ENGLISH_FREQ_ORDER: [u8; R] = [
    b'e', b't', b'a', b'o', b'n', b'i', b'h', b's', b'r', b'd', b'l', b'u', b'w', b'm', b'c', b'f',
    b'g', b'y', b'p', b'b', b'k', b'v', b'j', b'x', b'q', b'z',
];

/// Deciphers the string provided from CLI using statistics about english language.
///
/// # Errors
///
/// `std::io::Error` will be returned if `dict` file fails to read.
pub fn decrypt(input: &str, dict: impl BufRead) -> Result<String, std::io::Error> {
    // Create a dictionary of words
    let dict = load_dict(dict)?;

    // Create a list of word slices
    let input = filter_input(input);

    // Create a key for deciphering
    let mut key = Key::new(&input, ENGLISH_FREQ_ORDER);

    // Allocate output
    let mut output = Vec::with_capacity(input.len());
    let mut chars_set = bitset::BitSet64::<1>::new();

    // Recursive deciphering
    match decrypt_words(&input, &mut output, &mut key, &mut chars_set, &dict) {
        Ok(()) => Ok(String::from_utf8(output).unwrap()),
        Err(()) => Ok(String::from_utf8(input).unwrap()),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashMap;
    use std::hash::Hash;

    #[test]
    fn filter_input_keeps_ascii_alphabetic_and_whitespace() {
        assert_eq!(filter_input("hello, world! 😊".into()), b"hello  world   ");
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
        assert_eq!(out.len(), input.len());
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
    fn decrypt_hello_world() {
        let input: String = "Hello world!".into();
        let encrypted = encrypt(&input);
        dbg!(&input);
        dbg!(&encrypted);
        let decrypted = decrypt(
            &encrypted,
            std::io::BufReader::new("hello\nworld\n".as_bytes()),
        )
        .unwrap();
        assert_eq!(&decrypted, "hello world ");
    }

    #[test]
    fn decrypt_hello_world_with_red_herrings() {
        let input: String = "  Hello    world! ".into();
        let encrypted = encrypt(&input);
        dbg!(&input);
        dbg!(&encrypted);
        let decrypted = decrypt(
            &encrypted,
            std::io::BufReader::new(
                "hello\nworld\nword\nhell\nhey\nwonderful\nforth\nnewly\nbytes\ninput\n".as_bytes(),
            ),
        )
        .unwrap();
        assert_eq!(&decrypted, "  hello    world  ");
    }

    #[test]
    fn key_input_frequency_order() {
        let input = filter_input("aaaaa bbvvvbb oo e");
        let key = Key::new(&input, ENGLISH_FREQ_ORDER);

        assert_eq!(key.input_freq_index[usize::from(b'a' - START)], 0);
        assert_eq!(key.input_freq_index[usize::from(b'b' - START)], 1);
        assert_eq!(key.input_freq_index[usize::from(b'v' - START)], 2);
        assert_eq!(key.input_freq_index[usize::from(b'o' - START)], 3);
        assert_eq!(key.input_freq_index[usize::from(b'e' - START)], 4);
    }

    #[test]
    fn key_next_in_freq_order_covers_all_for_all() {
        for start_from in START..=END {
            let mut values_got = [0; R];
            let mut current = start_from;
            let dummy = Key::new(b"", ENGLISH_FREQ_ORDER);
            while {
                println!("Got '{}'", char::from(current));
                values_got[usize::from(current - START)] += 1;
                current = dummy.next_in_freq_order(start_from, current);
                current != 0
            } {}
            assert_eq!(values_got, [1; R]);
        }
    }

    fn assert_key_next_in_freq_order(start: u8, expected: &[u8]) {
        let mut current = start;
        let dummy = Key::new(b"", ENGLISH_FREQ_ORDER);
        for chr in expected {
            match dummy.next_in_freq_order(start, current) {
                0 => {
                    assert_eq!(*chr, 0);
                }
                val => {
                    current = val;
                    println!("{} == {}", char::from(current), char::from(*chr));
                    assert_eq!(current, *chr);
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
