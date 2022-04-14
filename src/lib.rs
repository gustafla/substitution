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

struct Key {
    table: [u8; R],
    guesses: bitset::U64BitSet<4>,
}

impl Key {
    fn new() -> Self {
        Self {
            table: [0; R],
            guesses: bitset::U64BitSet::<4>::new(),
        }
    }

    fn index(input: u8) -> usize {
        usize::from(input - START)
    }

    fn attach(&mut self, input: u8, guess: u8) -> Result<(), ()> {
        if self.guesses.contains(guess) {
            return Err(());
        }
        let idx = Self::index(input);
        self.guesses.remove(self.table[idx]);
        self.table[idx] = guess;
        self.guesses.insert(guess);
        Ok(())
    }

    fn next_in_order(current_guess: u8) -> u8 {
        if current_guess >= END {
            0
        } else {
            current_guess + 1
        }
    }

    fn attach_next(&mut self, input: u8) -> Result<(), ()> {
        let idx = Self::index(input);
        let mut current_guess = self.table[idx];

        // Do first guess
        if current_guess == 0 {
            current_guess = START;
            if self.attach(input, current_guess).is_ok() {
                return Ok(());
            }
        };

        while {
            current_guess = Self::next_in_order(current_guess);
            if current_guess == 0 {
                return Err(());
            }
            self.attach(input, current_guess).is_err()
        } {}
        Ok(())
    }

    fn clear(&mut self, input: u8) {
        let idx = Self::index(input);
        self.guesses.remove(self.table[idx]);
        self.table[idx] = 0;
    }

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

fn validate(text: &[u8], dict: &trie::Set<R>) -> usize {
    let mut score = 0;
    for word in text.split(u8::is_ascii_whitespace) {
        score += dict.prefix_score(&bytes_to_key(word));
    }
    score
}

fn unique_chars(input: &[u8]) -> Vec<u8> {
    let mut uc = Vec::with_capacity(16);
    let mut set = bitset::U64BitSet::<4>::new();
    for c in input {
        if c.is_ascii_alphabetic() && !set.contains(*c) {
            uc.push(*c);
            set.insert(*c);
        }
    }
    uc
}

fn seek_words(from: &[u8], mut count: usize) -> &[u8] {
    let mut trim_start = true;
    for i in 0..from.len() {
        if from[i].is_ascii_whitespace() {
            if trim_start {
                continue;
            }
            count -= 1;
            if count == 0 {
                return &from[..i];
            }
        }
        trim_start = false;
    }
    from
}

fn decrypt_words(
    input: &[u8],
    output: &mut Vec<u8>,
    key: &mut Key,
    chars_set: &mut bitset::U64BitSet<1>,
    dict: &trie::Set<R>,
) -> Result<(), ()> {
    if input.is_empty() {
        return Ok(());
    }

    // Find 3 words
    let in_words = seek_words(input, 3);

    // Reserve translation scratch area in output
    let out_len = output.len();
    output.extend(in_words);

    // Generate list of currently relevant and unset chars
    let free_chars: Vec<u8> = unique_chars(in_words)
        .into_iter()
        .filter(|c| !chars_set.contains(c - START))
        .collect();

    'test: loop {
        key.translate(&mut output[out_len..]);

        if validate(&output[out_len..], dict) >= in_words.len() {
            println!(
                "Found likely words \"{}\"",
                String::from_utf8_lossy(&output[out_len..])
            );

            // Set current key in stone for next round
            free_chars.iter().for_each(|c| chars_set.insert(*c - START));

            if decrypt_words(&input[in_words.len()..], output, key, chars_set, dict).is_ok() {
                return Ok(());
            }

            // Deciphering ahead failed, current assumptions aren't right

            // Clear set characters that weren't previously set in stone before call
            free_chars.iter().for_each(|c| chars_set.remove(*c - START));
        }
        // Reset translation buffer
        (&mut output[out_len..]).copy_from_slice(in_words);

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
    output.truncate(output.len() - in_words.len());

    Err(())
}

/// Deciphers the string provided from CLI using statistics about english language.
pub fn decrypt(input: &str, dict: impl BufRead) -> Result<String, std::io::Error> {
    // Create a dictionary of words
    let dict = load_dict(dict)?;

    // Create a list of word slices
    let input = filter_input(input);

    // Create a key for deciphering
    let mut key = Key::new();

    // Allocate output
    let mut output = Vec::with_capacity(input.len());
    let mut chars_set = bitset::U64BitSet::<1>::new();

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
        assert_eq!(&decrypted, "hello world");
    }

    // #[test]
    // fn key_input_frequency_order() {
    //     let input = filter_input("aaaaa bbvvvbb oo e");
    //     let words: Vec<&[u8]> = input.split(u8::is_ascii_whitespace).collect();
    //     let key = Key::new(&words);
    //     dbg!(key.input_frequency_order);
    //     assert!(matches!(
    //         key.input_frequency_order,
    //         [b'a', b'b', b'v', b'o', b'e', ..]
    //     ));
    // }

    // #[test]
    // fn key_next_in_freq_order_covers_all_for_all() {
    //     for start_from in START..=END {
    //         let mut values_got = [0; R];
    //         let mut current = start_from;
    //         values_got[usize::from(current - START)] += 1;
    //         while let Some(next) = Key::next_in_freq_order(start_from, current) {
    //             current = next.get();
    //             println!("Got '{}'", char::from(current));
    //             values_got[usize::from(current - START)] += 1;
    //         }
    //         assert_eq!(values_got, [1; R]);
    //     }
    // }

    // fn assert_key_next_in_freq_order(start: u8, expected: &[u8]) {
    //     let mut current = NonZeroU8::new(start).unwrap();
    //     for chr in expected {
    //         match Key::next_in_freq_order(start, current.get()) {
    //             Some(val) => {
    //                 current = val;
    //                 println!("{} == {}", char::from(current.get()), char::from(*chr));
    //                 assert_eq!(current.get(), *chr);
    //             }
    //             None => {
    //                 assert_eq!(*chr, 0);
    //             }
    //         }
    //     }
    // }

    // #[test]
    // fn key_next_in_freq_order_looks_correct() {
    //     assert_key_next_in_freq_order(b'a', b"toen");
    //     assert_key_next_in_freq_order(b'o', b"antiehsrd");
    //     assert_key_next_in_freq_order(b'b', b"pkyvgjfxcqmzwuldrshinoate\0");
    // }
}
