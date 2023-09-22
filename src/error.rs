use std::fmt;
use crate::bytestring::ByteString;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DecodingError {
    MissingIdentifier(char),
    KeyWithoutValue(ByteString),
    StringWithoutLength,
    NotANumber,
    EndOfFile,
    NegativeZero,
    NegativeStringLen,
}

impl fmt::Display for DecodingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DecodingError::MissingIdentifier(chr) => write!(f, "Expected identifier '{}'", chr),
            DecodingError::KeyWithoutValue(key) => write!(f, "Dictionary key '{}' without value", key),
            DecodingError::EndOfFile => write!(f, "Unexpected end of file"),
            DecodingError::StringWithoutLength => write!(f, "Expected string length"),
            DecodingError::NotANumber => write!(f, "Expected a number but "),
            DecodingError::NegativeZero => write!(f, "Negative zero is not allowed. Use 0 instead"),
            DecodingError::NegativeStringLen => write!(f, "Negative string length is not allowed"),
        }
    }
}