use linked_hash_map::LinkedHashMap;

use crate::bdecode::BEncodingType;
use crate::bytestring::ByteString;

pub(crate) fn encode(bencoded: BEncodingType) -> Vec<u8> {
    // TODO: Don't use vec. Try to find a bytes writer
    let mut buf = Vec::new();
    encode_type(bencoded, &mut buf);
    buf
}

fn encode_type(bencoding: BEncodingType, buf: &mut Vec<u8>) {
    match bencoding {
        BEncodingType::Integer(int) => { encode_int(int, buf); }
        BEncodingType::String(bytes) => { encode_bytestring(bytes, buf) }
        BEncodingType::List(list) => { encode_list(list, buf) }
        BEncodingType::Dictionary(dict) => { encode_dict(dict, buf) }
    };
}

fn encode_dict(dict: LinkedHashMap<ByteString, BEncodingType>, buf: &mut Vec<u8>) {
    buf.push(b'd');
    for (key, val) in dict.into_iter() {
        encode_bytestring(key, buf);
        encode_type(val, buf);
    }
    buf.push(b'e');
}

fn encode_list(list: Vec<BEncodingType>, buf: &mut Vec<u8>) {
    buf.push(b'l');
    for item in list {
        encode_type(item, buf);
    }
    buf.push(b'e')
}

fn encode_bytestring(bs: ByteString, buf: &mut Vec<u8>) {
    encode_num(bs.0.len() as i64, buf);
    buf.push(b':');
    buf.extend(bs.0.iter());
}

fn encode_int(int: i64, buf: &mut Vec<u8>) {
    buf.push(b'i');
    // FIXME: Performance is likely slow
    encode_num(int, buf);
    buf.push(b'e');
}

fn encode_num(int: i64, buf: &mut Vec<u8>) {
    buf.extend(int.to_string().bytes());
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn encode_int_zero() {
        let mut v = Vec::new();
        encode_int(0, &mut v);
        assert_eq!(b"i0e".to_vec(), v)
    }

    #[test]
    fn encode_int_positive_number() {
        let mut v = Vec::new();
        encode_int(1234, &mut v);
        assert_eq!(b"i1234e".to_vec(), v);

        encode_int(567, &mut v);
        assert_eq!(b"i1234ei567e".to_vec(), v);
    }

    #[test]
    fn encode_int_negative_number() {
        let mut v = Vec::new();
        encode_int(-123, &mut v);
        assert_eq!(b"i-123e".to_vec(), v);

        encode_int(-45, &mut v);
        assert_eq!(b"i-123ei-45e".to_vec(), v);

        encode_int(67, &mut v);
        assert_eq!(b"i-123ei-45ei67e".to_vec(), v);
    }

    #[test]
    fn test_encode_bytestring() {
        let mut v = Vec::new();
        encode_bytestring(ByteString(b"abcd".to_vec()), &mut v);
        assert_eq!(b"4:abcd".to_vec(), v);

        encode_bytestring(ByteString(b"123".to_vec()), &mut v);
        assert_eq!(b"4:abcd3:123".to_vec(), v);

        encode_bytestring(ByteString(b"\n\r\t\\/,".to_vec()), &mut v);
        assert_eq!(b"4:abcd3:1236:\n\r\t\\/,".to_vec(), v);
    }

    #[test]
    fn encode_list_empty() {
        let mut v = Vec::new();
        encode_list(Vec::new(), &mut v);
        assert_eq!(b"le".to_vec(), v);
    }

    fn encode_list_flat() {
        let mut v = Vec::new();
        encode_list(vec![
            BEncodingType::String(ByteString(b"abc".to_vec())),
            BEncodingType::Integer(345),
            BEncodingType::String(ByteString(b"def".to_vec())),
        ], &mut v);
        assert_eq!(b"l3:abci345e3:defe".to_vec(), v);
    }

    #[test]
    fn encode_list_inner() {
        let mut v = Vec::new();
        encode_list(vec![
            BEncodingType::Integer(345),
            BEncodingType::List(vec![
                BEncodingType::String(ByteString(b"inner".to_vec())),
                BEncodingType::Integer(999),
                BEncodingType::List(vec![
                    BEncodingType::Integer(10000)
                ])
            ]),
            BEncodingType::String(ByteString(b"def".to_vec())),
            BEncodingType::List(vec![]),
        ], &mut v);
        assert_eq!(b"li345el5:inneri999eli10000eee3:deflee".to_vec(), v);
    }

    #[test]
    fn encode_dict_empty() {
        let mut v = Vec::new();
        encode_dict(LinkedHashMap::new(), &mut v);
        assert_eq!(b"de".to_vec(), v);
    }

    #[test]
    fn encode_dict_flat() {
        let mut v: Vec<u8> = Vec::new();
        let mut dict = LinkedHashMap::new();
        dict.insert(ByteString(b"item1".to_vec()), BEncodingType::Integer(123));
        dict.insert(ByteString(b"item2".to_vec()), BEncodingType::String(ByteString(b"value".to_vec())));
        encode_dict(dict, &mut v);
        assert_eq!(b"d5:item1i123e5:item25:valuee".to_vec(), v);
    }

    #[test]
    fn encode_dict_layered() {
        let mut v: Vec<u8> = Vec::new();
        let mut dict = LinkedHashMap::new();
        dict.insert(ByteString(b"item1".to_vec()), BEncodingType::Integer(123));
        dict.insert(ByteString(b"item2".to_vec()), BEncodingType::String(ByteString(b"value".to_vec())));

        let mut inner_dict = LinkedHashMap::new();
        inner_dict.insert(ByteString(b"inneritem1".to_vec()), BEncodingType::Integer(888));
        let mut innermost_dict = LinkedHashMap::new();
        innermost_dict.insert(ByteString(b"core".to_vec()), BEncodingType::Integer(50000));
        inner_dict.insert(ByteString(b"inneritem2".to_vec()), BEncodingType::Dictionary(innermost_dict));

        dict.insert(ByteString(b"inner".to_vec()), BEncodingType::Dictionary(inner_dict));

        encode_dict(dict, &mut v);
        assert_eq!(b"d5:item1i123e5:item25:value5:innerd10:inneritem1i888e10:inneritem2d4:corei50000eeee".to_vec(), v);
    }
}