# Data Structures and Algorithms Project - Deciphering Substitution Ciphers

[![build](https://github.com/gustafla/substitution/actions/workflows/build.yml/badge.svg)](https://github.com/gustafla/substitution/actions/workflows/build.yml)
[![codecov](https://codecov.io/gh/gustafla/substitution/branch/master/graph/badge.svg?token=TKGUHNQHFV)](https://codecov.io/gh/gustafla/substitution)

[Specification (määrittelydokumentti)](doc/specification.md)

[Implementation (toteutusdokumentti)](doc/implementation.md)

## Weekly logs / viikkoraportit

- [Week 1](doc/week1_log.md)
- [Week 2](doc/week2_log.md)
- [Week 3](doc/week3_log.md)
- [Week 4](doc/week4_log.md)
- [Week 5](doc/week5_log.md)
- [Week 6](doc/week6_log.md)

## Overview

This project is written in [Rust](https://rust-lang.org) and uses Cargo as its
build and test system. Code coverage is currently generated using
[tarpaulin](https://github.com/xd009642/tarpaulin).

The command line interface starts from [main.rs](src/main.rs) (`fn main()`) and
the business logic and unit tests reside in [lib.rs](src/lib.rs).

## Building and running

To run the program, run `cargo run`. Command line arguments to the
application can be supplied after `--`, for example
`cargo run -- encrypt`. See `cargo run -- --help` for usage and options.

A dictionary file is needed. The default option is `/usr/share/dict/words`
but it can be changed with `--dictionary`.
If you run Arch Linux, install the package `words`.
If you run Ubuntu, install the package `wamerican` or `wbritish`.

To build a (best performance) release binary, run `cargo build --release`.
The output goes to `target/release/substitution`.

## Source code documentation

Source documentation is implemented with
[rustdoc](https://doc.rust-lang.org/rustdoc/index.html).

Currently the project's business logic API only has two functions, so the
generated documentation does not give much insight into the internals of this
project. Thus, prebuilt code documentation is not provided at this time.
You can build the documentation from the code by running `cargo doc`.
HTML documentation will be generated to `target/doc/substitution/(index.html)`.
The `--open` flag (`cargo doc --open`) also opens it in your browser.

The code is commented with a best effort approach.
Every function has at least a brief documentation comment.

## Testing and coverage

Tests can be run with `cargo test`. Optionally,
[cargo-nextest](https://nexte.st) is a faster and smarter test runner which you
can use just as well.

Test modules (`mod test`) have `#[cfg(test)]` attributes which instruct the
compiler to only compile them when building for testing.
Test cases in test modules have `#[test]` attributes.
See [Rust by Example](https://doc.rust-lang.org/rust-by-example/testing/unit_testing.html)
for a brief tutorial.

Code coverage can be measured with `cargo tarpaulin` (provided you have
tarpaulin installed), but Github Actions has been configured to do that and
upload the [results](https://codecov.io/gh/gustafla/substitution) to
codecov.io.

The test suite consists of unit tests for library functions ([lib.rs](src/lib.rs)).
Every function should have at least one corresponding unit test.
The encyption and decryption functions are tested with short sentences, longer
texts and also really long (100000 characters) randomly generated texts. The
encryption function has tests for output length and frequency profile, and
the decryption function currently only has a few tests for short inputs and
small dictionaries.

## Linting and style

A basic compiler check can be performed with `cargo check` but you should run
`cargo clippy` to get all lints and warnings in addition to compiler's built-in
checks. This is also performed in CI and the "build" badge reflects the status.

The codebase disallows all
[unsafe code](https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html),
missing documentation, and clippy's warning-level lints.
Extra clippy warnings are also generated from pedantic lints.

See the crate-level lint attributes in the beginning of [lib.rs](src/lib.rs).
