# Data Structures and Algorithms Project - Deciphering Substitution Ciphers

[![build](https://github.com/gustafla/substitution/actions/workflows/build.yml/badge.svg)](https://github.com/gustafla/substitution/actions/workflows/build.yml)
[![codecov](https://codecov.io/gh/gustafla/substitution/branch/master/graph/badge.svg?token=TKGUHNQHFV)](https://codecov.io/gh/gustafla/substitution)

[Specification (määrittelydokumentti)](doc/specification.md)

## Weekly logs / viikkoraportit

- [Week 1](doc/week1_log.md)
- [Week 2](doc/week2_log.md)

## Overview

This project is written in [Rust](https://rust-lang.org) and uses Cargo as its
build and test system. Code coverage is currently generated using
[tarpaulin](https://github.com/xd009642/tarpaulin).

The command line interface starts from [main.rs](src/main.rs) (`fn main()`) and
currently all code resides in that single file.

## Building and running

To build a debug binary, run `cargo build`. The output goes to
`target/debug/substitution`.

To build a (best performance) release binary, run `cargo build --release`.
The output goes to `target/release/substitution`.

To build and run (both at once), run `cargo run`. Command line arguments to the
application can be supplied after `--`, for example
`cargo run -- encrypt Hello`.

## Source code documentation

Source documentation is implemented with
[rustdoc](https://doc.rust-lang.org/rustdoc/index.html).

Prebuilt code documentation is not provided at this time. You can build the
documentation from the code by running `cargo doc`.
HTML documentation will be generated to `target/doc/substitution/(index.html)`.
The `--open` flag (`cargo doc --open`) also opens it in your browser.
