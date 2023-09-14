use std::fmt;

use linked_hash_map::LinkedHashMap;

type Result<T> = std::result::Result<T, BencodeError>;


// TODO: Add some error kinds to differentiate between different errors
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum BencodeError {
    Err,
    MissingIdentifier(char),
    KeyWithoutValue(String),
    StringWithoutLength,
    NotANumber,
    EndOfFile,
}

impl fmt::Display for BencodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BencodeError::MissingIdentifier(chr) => {
                write!(f, "Expected identifier '{}'", chr)
            }
            BencodeError::KeyWithoutValue(key) => {
                write!(f, "Dictionary key '{}' without value", key)
            }
            BencodeError::EndOfFile => {
                write!(f, "Unexpected end of file")
            }
            BencodeError::StringWithoutLength => {
                write!(f, "Expected string length")
            }
            BencodeError::NotANumber => {
                write!(f, "Expected a number but ")
            }
            _ => {
                write!(f, "Unknown error during parsing")
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum BEncodingType {
    Integer(i64),
    String(String),
    List(Vec<BEncodingType>),
    Dictionary(LinkedHashMap<String, BEncodingType>),
}

pub struct BEncodingParser<'a> {
    bytes: &'a [u8],
    cursor: usize,
}

impl BEncodingParser<'_> {
    fn new(bytes: &[u8]) -> BEncodingParser {
        BEncodingParser { bytes, cursor: 0 }
    }

    fn parse(&mut self) -> Result<BEncodingType> {
        self.parse_type()
    }

    fn parse_str(&mut self) -> Result<String> {
        let len = self.read_num().or(Err(BencodeError::StringWithoutLength))?;
        self.expect_char(b':')?;
        // TODO: implement
        let start = self.cursor;
        let end = start + len as usize;
        if end > self.bytes.len() {
            return Err(BencodeError::EndOfFile);
        }
        self.cursor = end;
        Ok(String::from_utf8_lossy(&self.bytes[start..end]).to_string())
    }

    fn parse_int(&mut self) -> Result<i64> {
        self.expect_char(b'i')?;
        let i = self.read_num()?;
        self.expect_char(b'e')?;
        Ok(i)
    }

    fn parse_list(&mut self) -> Result<Vec<BEncodingType>> {
        self.expect_char(b'l')?;
        let mut list = Vec::new();
        while self.peek().filter(|&c| c != b'e').is_some() {
            list.push(self.parse_type()?);
        }
        self.expect_char(b'e')?;
        Ok(list)
    }

    fn parse_dict(&mut self) -> Result<LinkedHashMap<String, BEncodingType>> {
        self.expect_char(b'd')?;
        let mut dict = LinkedHashMap::new();
        while self.peek().filter(|&c| c != b'e').is_some() {
            let key = self.parse_str()?;
            let value = self.parse_type()
                .map_err(|_| BencodeError::KeyWithoutValue(key.clone()))?;
            dict.insert(key, value);
        }
        self.expect_char(b'e')?;
        Ok(dict)
    }

    fn parse_type(&mut self) -> Result<BEncodingType> {
        if let Some(byte) = self.peek() {
            let result = match byte {
                b'i' => BEncodingType::Integer(self.parse_int()?),
                b'l' => BEncodingType::List(self.parse_list()?),
                b'd' => BEncodingType::Dictionary(self.parse_dict()?),
                _ => BEncodingType::String(self.parse_str()?),
            };
            Ok(result)
        } else {
            Err(BencodeError::Err)
        }
    }

    fn read_num(&mut self) -> Result<i64> {
        // FIXME: This logic is simple but looks a bit too clunky, try an alternative
        let mut neg_const = 1;
        if self.peek() == Some(b'-') {
            neg_const = -1;
            self.safe_advance_and_discard();
        }
        // FIXME: We are peeking twice here, try to avoid it
        if let Some(chr) = self.peek() {
            if !chr.is_ascii_digit() {
                return Err(BencodeError::NotANumber);
            }
        }
        let mut acc = 0;
        while let Some(v) = self.peek() {
            if v.is_ascii_digit() {
                acc = acc * 10 + (v - b'0') as i64;
                self.safe_advance_and_discard();
            } else {
                break;
            }
        };
        Ok(acc * neg_const)
    }

    fn expect_char(&mut self, expected: u8) -> Result<u8> {
        match self.peek() {
            Some(chr) if chr == expected => self.advance(),
            _ => Err(BencodeError::MissingIdentifier(expected as char)),
        }
    }

    fn peek(&mut self) -> Option<u8> {
        self.bytes.get(self.cursor).cloned()
    }

    fn advance(&mut self) -> Result<u8> {
        let v = self.bytes.get(self.cursor).cloned();
        self.cursor += 1;
        v.ok_or(BencodeError::EndOfFile)
    }

    // FIXME: I am not happy with this
    fn safe_advance_and_discard(&mut self) {
        self.cursor += 1;
    }
}

pub fn decode(inp: &[u8]) -> Result<BEncodingType> {
    let mut parser = BEncodingParser::new(inp);
    parser.parse()
}


// TODO: Also test cursor positions
#[cfg(test)]
mod test {

    use super::*;

    #[test]
    pub fn expect_char() {
        let mut parser = BEncodingParser::new(b"abc");
        assert_eq!(parser.expect_char(b'a'), Ok(b'a'));
        assert_eq!(parser.expect_char(b'a'), Err(BencodeError::MissingIdentifier('a')));
    }

    #[test]
    pub fn test_parse_integer() {
        let parse_int = |inp: &str| BEncodingParser::new(inp.as_bytes()).parse_int();

        assert_eq!(Ok(123), parse_int("i123e"));
        assert_eq!(Ok(-123), parse_int("i-123e"));
        assert_eq!(Err(BencodeError::MissingIdentifier('i')), parse_int("abc"));
        assert_eq!(Err(BencodeError::NotANumber), parse_int("iabc"));
        assert_eq!(Err(BencodeError::NotANumber), parse_int("i-abc"));
        assert_eq!(Err(BencodeError::MissingIdentifier('e')), parse_int("i23f"));
    }

    #[test]
    pub fn test_parse_string() {
        let parse_string = |inp: &str| BEncodingParser::new(inp.as_bytes()).parse_str();

        assert_eq!(Ok("abc".to_string()), parse_string("3:abc"));

        assert_eq!(Ok("".to_string()), parse_string("0:"));
        assert_eq!(Err(BencodeError::StringWithoutLength), parse_string("abc"));
        assert_eq!(Err(BencodeError::MissingIdentifier(':')), parse_string("3abc"));
        assert_eq!(Err(BencodeError::EndOfFile), parse_string("3:ab"));
    }

    #[test]
    pub fn test_parse_list() {
        let parse_list = |inp: &str| BEncodingParser::new(inp.as_bytes()).parse_list();

        assert_eq!(Ok(vec![]), parse_list("le"));
        assert_eq!(Ok(vec![BEncodingType::Integer(123)]), parse_list("li123ee"));
        assert_eq!(Ok(vec![BEncodingType::String("abc".to_string())]), parse_list("l3:abce"));
        assert_eq!(Ok(vec![BEncodingType::String("abc".to_string()), BEncodingType::String("defg".to_string())]), parse_list("l3:abc4:defge"));
        assert_eq!(Ok(vec![BEncodingType::List(vec![])]), parse_list("llee"));
        assert_eq!(Ok(vec![
            BEncodingType::List(vec![BEncodingType::List(vec![])]),
            BEncodingType::List(vec![BEncodingType::List(vec![])]),
        ]), parse_list("llleelleee"));
        assert_eq!(Err(BencodeError::MissingIdentifier('l')), parse_list("abc"));
        assert_eq!(Err(BencodeError::MissingIdentifier('e')), parse_list("l3:abc"));
        assert_eq!(Err(BencodeError::MissingIdentifier('l')), parse_list("abc"));
    }

    #[test]
    pub fn test_parse_dictionary() {
        let parse_dictionary = |inp: &str| BEncodingParser::new(inp.as_bytes()).parse_dict();

        assert_eq!(Ok(LinkedHashMap::new()), parse_dictionary("de"));

        let mut dct = LinkedHashMap::new();
        dct.insert("a".to_string(), BEncodingType::Integer(123));
        assert_eq!(Ok(dct), parse_dictionary("d1:ai123ee"));

        let mut dct = LinkedHashMap::new();
        dct.insert("a".to_string(), BEncodingType::List(vec![BEncodingType::String(String::from("hey"))]));
        dct.insert("b".to_string(), BEncodingType::List(vec![]));
        assert_eq!(Ok(dct), parse_dictionary("d1:al3:heye1:blee"));

        let mut dct = LinkedHashMap::new();
        let mut inner_dct = LinkedHashMap::new();
        inner_dct.insert("a".to_string(), BEncodingType::Integer(345));
        inner_dct.insert("b".to_string(), BEncodingType::String(String::from("wow")));
        dct.insert("inner".to_string(), BEncodingType::Dictionary(inner_dct));
        dct.insert("inner2".to_string(), BEncodingType::Dictionary(LinkedHashMap::new()));

        assert_eq!(Ok(dct), parse_dictionary("d5:innerd1:ai345e1:b3:wowe6:inner2dee"));

        assert_eq!(Err(BencodeError::MissingIdentifier('d')), parse_dictionary("abc"));
        assert_eq!(Err(BencodeError::KeyWithoutValue("item".to_string())), parse_dictionary("d4:iteme"));
    }
}
