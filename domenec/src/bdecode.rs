use linked_hash_map::LinkedHashMap;

use crate::error::DecodingError;

type Result<T> = std::result::Result<T, DecodingError>;

#[derive(Debug, Eq, PartialEq)]
pub enum BEncodingType {
    Integer(i64),
    String(String),
    List(Vec<BEncodingType>),
    Dictionary(LinkedHashMap<String, BEncodingType>),
}

pub struct BDecoder<'a> {
    bytes: &'a [u8],
    cursor: usize,
}

impl BDecoder<'_> {
    fn new(bytes: &[u8]) -> BDecoder {
        BDecoder { bytes, cursor: 0 }
    }

    fn decode(&mut self) -> Result<BEncodingType> {
        self.parse_type()
    }

    fn parse_str(&mut self) -> Result<String> {
        let len = self.read_num().or(Err(DecodingError::StringWithoutLength))?;
        self.expect_char(b':')?;
        // TODO: implement
        let start = self.cursor;
        let end = start + len as usize;
        if end > self.bytes.len() {
            self.cursor = self.bytes.len();
            return Err(DecodingError::EndOfFile);
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
                .map_err(|_| DecodingError::KeyWithoutValue(key.clone()))?;
            dict.insert(key, value);
        }
        self.expect_char(b'e')?;
        Ok(dict)
    }

    fn parse_type(&mut self) -> Result<BEncodingType> {
        match self.peek() {
            None => Err(DecodingError::Err),
            Some(b'i') => self.parse_int().map(BEncodingType::Integer),
            Some(b'l') => self.parse_list().map(BEncodingType::List),
            Some(b'd') => self.parse_dict().map(BEncodingType::Dictionary),
            Some(_) => self.parse_str().map(BEncodingType::String)
        }
    }

    fn read_num(&mut self) -> Result<i64> {
        let mut neg_const = 1;
        if self.peek() == Some(b'-') {
            neg_const = -1;
            self.cursor += 1;
        }
        // FIXME: Consider a cleaner early return here, not happy with the catchall
        match self.peek() {
            None => Err(DecodingError::EndOfFile),
            Some(chr) if !chr.is_ascii_digit() => Err(DecodingError::NotANumber),
            Some(chr) if neg_const == -1 && chr == b'0' => Err(DecodingError::NegativeZero),
            _ => Ok(())
        }?;
        let mut acc = 0;
        while let Some(v) = self.peek() {
            if v.is_ascii_digit() {
                acc = acc * 10 + (v - b'0') as i64;
                self.cursor += 1;
            } else {
                break;
            }
        };
        Ok(acc * neg_const)
    }

    fn expect_char(&mut self, expected: u8) -> Result<u8> {
        match self.peek() {
            None => Err(DecodingError::EndOfFile),
            Some(chr) if chr == expected => self.advance(),
            _ => Err(DecodingError::MissingIdentifier(expected as char)),
        }
    }

    fn peek(&mut self) -> Option<u8> {
        self.bytes.get(self.cursor).cloned()
    }

    fn advance(&mut self) -> Result<u8> {
        let v = self.bytes.get(self.cursor).cloned();
        self.cursor += 1;
        v.ok_or(DecodingError::EndOfFile)
    }
}

pub fn decode(inp: &[u8]) -> Result<BEncodingType> {
    let mut parser = BDecoder::new(inp);
    parser.decode()
}

// TODO: Add tests for some real world examples
// TODO: Add benchmarks
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn expect_char() {
        let mut parser = BDecoder::new(b"abc");

        assert_eq!(parser.expect_char(b'a'), Ok(b'a'));
        assert_eq!(parser.cursor, 1);
        assert_eq!(parser.expect_char(b'a'), Err(DecodingError::MissingIdentifier('a')));
        assert_eq!(parser.cursor, 1);
    }

    #[test]
    pub fn test_parse_integer() {
        let parse_int = |inp: &str| {
            let mut decoder = BDecoder::new(inp.as_bytes());
            (decoder.parse_int(), decoder.cursor)
        };

        assert_eq!((Ok(123), 5), parse_int("i123e"));
        assert_eq!((Ok(-123), 6), parse_int("i-123e"));
        assert_eq!((Err(DecodingError::NegativeZero), 2), parse_int("i-0e"));
        assert_eq!((Err(DecodingError::MissingIdentifier('i')), 0), parse_int("abc"));
        assert_eq!((Err(DecodingError::NotANumber), 1), parse_int("iabc"));
        assert_eq!((Err(DecodingError::NotANumber), 2), parse_int("i-abc"));
        assert_eq!((Err(DecodingError::MissingIdentifier('e')), 3), parse_int("i23abc"));
        assert_eq!((Err(DecodingError::EndOfFile), 3), parse_int("i23"));
    }

    #[test]
    pub fn test_parse_string() {
        let parse_string = |inp: &str| {
            let mut decoder = BDecoder::new(inp.as_bytes());
            (decoder.parse_str(), decoder.cursor)
        };

        assert_eq!((Ok("abc".to_string()), 5), parse_string("3:abc"));
        assert_eq!((Ok("".to_string()), 2), parse_string("0:"));
        assert_eq!((Err(DecodingError::StringWithoutLength), 0), parse_string("abc"));
        assert_eq!((Err(DecodingError::MissingIdentifier(':')), 1), parse_string("3abc"));
        assert_eq!((Err(DecodingError::EndOfFile), 4), parse_string("3:ab"));
    }

    #[test]
    pub fn test_parse_list() {
        let parse_list = |inp: &str| {
            let mut decoder = BDecoder::new(inp.as_bytes());
            (decoder.parse_list(), decoder.cursor)
        };

        assert_eq!((Ok(vec![]), 2), parse_list("le"));
        assert_eq!((Ok(vec![BEncodingType::Integer(123)]), 7), parse_list("li123ee"));
        assert_eq!((Ok(vec![BEncodingType::String("abc".to_string())]), 7), parse_list("l3:abce"));
        assert_eq!((Ok(vec![BEncodingType::String("abc".to_string()), BEncodingType::String("defg".to_string())]), 13), parse_list("l3:abc4:defge"));
        assert_eq!((Ok(vec![BEncodingType::List(vec![])]), 4), parse_list("llee"));
        assert_eq!((Ok(vec![
            BEncodingType::List(vec![BEncodingType::List(vec![])]),
            BEncodingType::List(vec![BEncodingType::List(vec![])]),
        ]), 10), parse_list("llleelleee"));
        assert_eq!((Err(DecodingError::MissingIdentifier('l')), 0), parse_list("abc"));
        assert_eq!((Err(DecodingError::EndOfFile), 6), parse_list("l3:abc"));
    }

    #[test]
    pub fn test_parse_dictionary() {
        let parse_dictionary = |inp: &str| {
            let mut decoder = BDecoder::new(inp.as_bytes());
            (decoder.parse_dict(), decoder.cursor)
        };

        assert_eq!((Ok(LinkedHashMap::new()), 2), parse_dictionary("de"));

        let mut dct = LinkedHashMap::new();
        dct.insert("a".to_string(), BEncodingType::Integer(123));
        assert_eq!((Ok(dct), 10), parse_dictionary("d1:ai123ee"));

        let mut dct = LinkedHashMap::new();
        dct.insert("a".to_string(), BEncodingType::List(vec![BEncodingType::String(String::from("hey"))]));
        dct.insert("b".to_string(), BEncodingType::List(vec![]));
        assert_eq!((Ok(dct), 17), parse_dictionary("d1:al3:heye1:blee"));

        let mut dct = LinkedHashMap::new();
        let mut inner_dct = LinkedHashMap::new();
        inner_dct.insert("a".to_string(), BEncodingType::Integer(345));
        inner_dct.insert("b".to_string(), BEncodingType::String(String::from("wow")));
        dct.insert("inner".to_string(), BEncodingType::Dictionary(inner_dct));
        dct.insert("inner2".to_string(), BEncodingType::Dictionary(LinkedHashMap::new()));
        assert_eq!((Ok(dct), 37), parse_dictionary("d5:innerd1:ai345e1:b3:wowe6:inner2dee"));

        assert_eq!((Err(DecodingError::MissingIdentifier('d')), 0), parse_dictionary("abc"));
        assert_eq!((Err(DecodingError::KeyWithoutValue("item".to_string())), 7), parse_dictionary("d4:iteme"));
        assert_eq!((Err(DecodingError::EndOfFile), 8), parse_dictionary("d1:a2:bc"));
    }
}
