
mod bdecode;
mod bencode;
mod error;
mod bytestring;

fn main() {
    let inp = b"d1:ad2:xyd20:abcdefghij0123456789i555eeee";
    let decoded = bdecode::decode(inp).unwrap();
    println!("Decoded => {:?}", decoded);

    let encoded = bencode::encode(decoded);
    println!("Reencoded=> {:?}", String::from_utf8(encoded))
}
