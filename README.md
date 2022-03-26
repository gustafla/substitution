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

## Linting and style

A basic compiler check can be performed with `cargo check` but you should run
`cargo clippy` to get all lints and warnings in addition to compiler's built-in
checks. This is also performed in CI and the "build" badge reflects the status.

The codebase disallows all
[unsafe code](https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html),
missing documentation (and code examples), and clippy's warning-level lints.
Extra clippy warnings are also generated from pedantic lints.

See the crate-level lint attributes in the beginning of [main.rs](src/main.rs).
