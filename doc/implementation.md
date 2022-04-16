# Project Implementation (toteutusdokumentti)

The CLI is currectly completely finished with all relevant options, modes
and error reporting.

Statistical information about the ciphertext and langage of the plaintext
is used, and brute force + backtrack in the expected order or frequency has
been implemented and seems to work in some cases.

A dictionary with trie has been implemented.

While the project has technically met all requirements and estimates from
the [specification](specification.md), it is still inflexible and far
from optimal in performance.

## Current known limitations and issues:

- Unoptimal order of attack, decryption is slow if text doesn't start with
  uncommon patterns of has lots of common patterns.
- Inflexibility around words that are unknown to the dictionary.
  Such unknown words block the rest of the text from decrypting.