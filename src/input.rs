//! Bounded input adapter. Parsing and comparison remain independent of I/O.
use std::io::{self, Read};

use crate::{parse_trace, Event};

#[derive(Debug)]
pub enum InputError {
    Io(io::Error),
    TooLarge { limit: usize },
    InvalidUtf8,
    InvalidTrace(String),
}

impl std::fmt::Display for InputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "could not read trace: {error}"),
            Self::TooLarge { limit } => write!(f, "trace exceeds {limit} bytes"),
            Self::InvalidUtf8 => write!(f, "trace is not UTF-8"),
            Self::InvalidTrace(error) => write!(f, "invalid trace: {error}"),
        }
    }
}

impl std::error::Error for InputError {}

pub fn read_trace(reader: impl Read, limit: usize) -> Result<Vec<Event>, InputError> {
    let mut bytes = Vec::new();
    reader
        .take((limit as u64).saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(InputError::Io)?;
    if bytes.len() > limit {
        return Err(InputError::TooLarge { limit });
    }
    let text = std::str::from_utf8(&bytes).map_err(|_| InputError::InvalidUtf8)?;
    parse_trace(text).map_err(InputError::InvalidTrace)
}
