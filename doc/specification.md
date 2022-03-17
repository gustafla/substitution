# Project Specification (määrittelydokumentti)

- Curriculum / Opinto-ohjelma: Tietojenkäsittelytieteen kandidaatti (TKT)
- Implementation language: Rust
  - I also know Java, C, Javascript, Python, C++ and Haskell well enought to
    give peer reviews
- Documentation language: English
- The goal is to produce a CLI application which can crack a substitution
  cipher using a user-provided dictionary (such as `/usr/share/dict/finnish`).
- The project will use statistical information about the assumed langage of the
  plaintext, and brute force or backtrack in the expected order or frequency.
  A dictionary with trie will also be used.
- The worst case complexity will be exponential, but a long input which matches
  the dictionary well should result in a fast crack.
- https://en.wikipedia.org/wiki/Substitution_cipher
