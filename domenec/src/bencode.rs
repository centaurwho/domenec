#[derive(Debug, Eq, PartialEq)]
pub struct BEncoding {
    value: BEncodingType,
}

impl BEncoding {
    pub fn new(value: BEncodingType) -> BEncoding {
        BEncoding { value }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct DictionaryItem(String, BEncodingType);

impl DictionaryItem {
    pub fn new(key: String, value: BEncodingType) -> DictionaryItem {
        DictionaryItem(key, value)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum BEncodingType {
    Integer(i64),
    // TODO: no guarantee that this is a valid UTF-8 string
    String(String),
    List(Vec<BEncodingType>),
    Dictionary(Vec<DictionaryItem>),

    // TODO: implement encoding
}