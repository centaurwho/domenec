use std::collections::HashMap;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{char, i64};
use nom::combinator::map;
use nom::IResult;
use nom::multi::{length_data, many0};
use nom::sequence::{delimited, terminated};

#[derive(Debug, Eq, PartialEq)]
pub struct BEncoding {
    value: BEncodingType,
}

impl BEncoding {
    pub fn new(value: HashMap<String, BEncoding>) -> BEncoding {
        BEncoding {
            value: BEncodingType::Dictionary(value)
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum BEncodingType {
    Integer(i64),
    // TODO: no guarantee that this is a valid UTF-8 string
    String(String),
    List(Vec<BEncodingType>),
    Dictionary(HashMap<String, BEncoding>),
}

// Given a stream of bytes representing a bencoded string, return the decoded string
pub fn decode(inp: &str) -> IResult<&str, BEncodingType> {
    map(parse_dictionary, |x| x)(inp)
}

fn parse_type(inp: &str) -> IResult<&str, BEncodingType> {
    alt((
        parse_integer,
        parse_string,
        parse_list,
        // parse_dictionary
    ))(inp)
}

fn parse_dictionary(inp: &str) -> IResult<&str, BEncodingType> {
    Ok((inp, BEncodingType::Dictionary(HashMap::new())))
}

fn parse_list(inp: &str) -> IResult<&str, BEncodingType> {
    map(parse_items, |x| BEncodingType::List(x))(inp)
}

fn parse_items(inp: &str) -> IResult<&str, Vec<BEncodingType>> {
    alt((
        map(tag("le"), |_| vec![]),
        delimited(
            char('l'),
            many0(parse_type),
            char('e'),
        )))(inp)
}

fn parse_string(inp: &str) -> IResult<&str, BEncodingType> {
    map(
        length_data(terminated(
            map(i64, |x| x as usize), char(':'),
        )), |x| BEncodingType::String(String::from(x)),
    )(inp)
}

fn parse_integer(inp: &str) -> IResult<&str, BEncodingType> {
    delimited(
        char('i'),
        map(i64, BEncodingType::Integer),
        char('e'),
    )(inp)
}

#[cfg(test)]
mod test {
    use std::num::NonZeroUsize;

    use nom::{Err, Needed};
    use nom::Err::Incomplete;
    use nom::error::ErrorKind;
    use nom::error_position;

    use super::*;

    #[test]
    pub fn test_parse_integer() {
        assert_eq!(Ok(("", BEncodingType::Integer(123))), parse_integer("i123e"));
        assert_eq!(Ok(("", BEncodingType::Integer(-123))), parse_integer("i-123e"));
        assert_eq!(
            Err(Err::Error(error_position!("abc", ErrorKind::Char))),
            parse_integer("abc")
        );
        assert_eq!(
            Err(Err::Error(error_position!("abc", ErrorKind::Digit))),
            parse_integer("iabc")
        );
        assert_eq!(
            Err(Err::Error(error_position!("f", ErrorKind::Char))),
            parse_integer("i23f")
        );
    }

    #[test]
    pub fn test_parse_string() {
        assert_eq!(Ok(("", BEncodingType::String("abc".to_string()))), parse_string("3:abc"));
        assert_eq!(Ok(("", BEncodingType::String("".to_string()))), parse_string("0:"));
        assert_eq!(
            Err(Err::Error(error_position!("abc", ErrorKind::Digit))),
            parse_string("abc")
        );
        assert_eq!(
            Err(Err::Error(error_position!("abc", ErrorKind::Char))),
            parse_string("3abc")
        );
        assert_eq!(
            Err(Incomplete(Needed::Size(NonZeroUsize::new(1).unwrap()))),
            parse_string("3:ab")
        );
    }

    #[test]
    pub fn test_parse_list() {
        assert_eq!(Ok(("", BEncodingType::List(vec![]))), parse_list("le"));
        assert_eq!(Ok(("", BEncodingType::List(vec![BEncodingType::Integer(123)]))), parse_list("li123ee"));
        assert_eq!(Ok(("", BEncodingType::List(vec![BEncodingType::String("abc".to_string())]))), parse_list("l3:abce"));
        assert_eq!(Ok(("", BEncodingType::List(vec![BEncodingType::List(vec![])]))), parse_list("llee"));
        assert_eq!(Ok(("", BEncodingType::List(vec![
            BEncodingType::List(vec![BEncodingType::List(vec![])]),
            BEncodingType::List(vec![BEncodingType::List(vec![])]),
        ]))), parse_list("llleelleee"));
        assert_eq!(
            Err(Err::Error(error_position!("abc", ErrorKind::Char))),
            parse_list("abc")
        );
        assert_eq!(
            Err(Err::Error(error_position!("abc", ErrorKind::Char))),
            parse_list("labc")
        );
    }
}




