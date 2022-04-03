// Don't measure coverage, this is support code for CLI
#![cfg(not(tarpaulin_include))]

use color_eyre::eyre::Context;
use std::{
    fs::File,
    io::{Read, Stdin, Stdout, Write},
    path::PathBuf,
};

/// Inputs which the program can take
pub enum Input {
    File(File, PathBuf),
    Stdin(Stdin),
}

/// Conversion from optional file path to input types
impl TryFrom<Option<PathBuf>> for Input {
    type Error = color_eyre::Report;

    fn try_from(path: Option<PathBuf>) -> Result<Self, Self::Error> {
        Ok(if let Some(path) = path {
            // Has a file path (PathBuf) inside, try to open the file as read-only
            Self::File(
                File::open(&path).wrap_err(format!("Cannot open {} for input", &path.display()))?,
                path,
            )
        } else {
            // Does not have a file path, use stdin
            Self::Stdin(std::io::stdin())
        })
    }
}

/// Enable dynamic dispatch reads for Input
impl<'a> AsMut<dyn Read + 'a> for Input {
    fn as_mut(&mut self) -> &mut (dyn Read + 'a) {
        match self {
            Self::File(f, _) => f,
            Self::Stdin(s) => s,
        }
    }
}

/// Pretty-print for input types
impl std::fmt::Display for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::File(_, path) => path.display().fmt(f),
            Self::Stdin(_) => write!(f, "stdin"),
        }
    }
}

/// Outputs which the program can write to
pub enum Output {
    File(File, PathBuf),
    Stdout(Stdout),
}

/// Conversion from optional file path to output types
impl TryFrom<Option<PathBuf>> for Output {
    type Error = color_eyre::Report;

    fn try_from(path: Option<PathBuf>) -> Result<Self, Self::Error> {
        Ok(if let Some(path) = path {
            // Has a file path (PathBuf) inside, try to create the file as read-write
            Self::File(
                File::create(&path)
                    .wrap_err(format!("Cannot create {} for output", &path.display()))?,
                path,
            )
        } else {
            // Does not have a file path, use stdout
            Self::Stdout(std::io::stdout())
        })
    }
}

/// Enable dynamic dispatch writes for Output
impl<'a> AsMut<dyn Write + 'a> for Output {
    fn as_mut(&mut self) -> &mut (dyn Write + 'a) {
        match self {
            Self::File(f, _) => f,
            Self::Stdout(s) => s,
        }
    }
}

/// Pretty-print for output types
impl std::fmt::Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::File(_, path) => path.display().fmt(f),
            Self::Stdout(_) => write!(f, "stdout"),
        }
    }
}
