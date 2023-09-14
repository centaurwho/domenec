use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{char, i64};
use nom::combinator::map;
use nom::IResult;
use nom::multi::{length_data, many0};
use nom::sequence::{delimited, pair, terminated};
use crate::bencode::{BEncoding, BEncodingType, DictionaryItem};

// This is a simple implementation of the bencode format using nom. It may not be as efficient as
// using a hand-written parser, but it is easier to write and maintain.
// TODO: use a hand-written parser for better performance and benchmark the difference



// Given a stream of bytes representing a bencoded string, return the decoded string
// FIXME: Use &[u8] instead of &str
pub fn decode(inp: &str) -> IResult<&str, BEncoding> {
    map(parse_dictionary, |x| BEncoding::new(x))(inp)
}

fn parse_type(inp: &str) -> IResult<&str, BEncodingType> {
    alt((
        parse_integer,
        parse_string,
        parse_list,
        parse_dictionary
    ))(inp)
}

fn parse_dictionary(inp: &str) -> IResult<&str, BEncodingType> {
    map(
        delimited(
            char('d'),
            many0(parse_dictionary_item),
            char('e'),
        ), BEncodingType::Dictionary,
    )(inp)
}

fn parse_dictionary_item(inp: &str) -> IResult<&str, DictionaryItem> {
    map(
        pair(
            parse_string_raw,
            parse_type,
        ), |(key, value)| DictionaryItem::new(key.to_string(), value),
    )(inp)
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
    map(parse_string_raw, |x| BEncodingType::String(x.to_string()))(inp)
}

fn parse_string_raw(inp: &str) -> IResult<&str, &str> {
    length_data(terminated(
        map(i64, |x| x as usize), char(':'),
    ))(inp)
}

fn parse_integer(inp: &str) -> IResult<&str, BEncodingType> {
    delimited(
        char('i'),
        map(i64, BEncodingType::Integer),
        char('e'),
    )(inp)
}

// TODO: add tests using real torrent file contents
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

    #[test]
    pub fn test_parse_dictionary() {
        assert_eq!(Ok(("", BEncodingType::Dictionary(vec![]))), parse_dictionary("de"));
        assert_eq!(Ok(("", BEncodingType::Dictionary(vec![
            DictionaryItem::new("a".to_string(), BEncodingType::Integer(123)),
        ]))), parse_dictionary("d1:ai123ee"));
        assert_eq!(Ok(("", BEncodingType::Dictionary(vec![
            DictionaryItem::new("a".to_string(), BEncodingType::List(vec![BEncodingType::String(String::from("hey"))])),
            DictionaryItem::new("b".to_string(), BEncodingType::List(vec![])),
        ]))), parse_dictionary("d1:al3:heye1:blee"));
        assert_eq!(Ok(("", BEncodingType::Dictionary(vec![
            DictionaryItem::new(String::from("inner"), BEncodingType::Dictionary(vec![
                DictionaryItem::new(String::from("a"), BEncodingType::Integer(345)),
                DictionaryItem::new(String::from("b"), BEncodingType::String(String::from("wow"))),
            ])),
            DictionaryItem::new(String::from("inner2"), BEncodingType::Dictionary(vec![])),
        ]))), parse_dictionary("d5:innerd1:ai345e1:b3:wowe6:inner2dee"));

        assert_eq!(
            Err(Err::Error(error_position!("abc", ErrorKind::Char))),
            parse_dictionary("abc")
        );
        assert_eq!(
            Err(Err::Error(error_position!("4:iteme", ErrorKind::Char))),
            parse_dictionary("d4:iteme")
        );
    }
}




