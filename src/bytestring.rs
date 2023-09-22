use std::fmt::Display;

// Custom ByteString wrapper to avoid String allocations.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ByteString(pub Vec<u8>);

pub trait ToByteString {
    fn to_byte_string(&self) -> ByteString;
}

impl ToByteString for &str {
    fn to_byte_string(&self) -> ByteString {
        ByteString(self.as_bytes().to_vec())
    }
}

impl ToByteString for &[u8] {
    fn to_byte_string(&self) -> ByteString {
        ByteString(self.to_vec())
    }
}

impl Display for ByteString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = String::from_utf8_lossy(&self.0);
        write!(f, "{}", s)
    }
}
