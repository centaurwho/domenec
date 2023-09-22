mod bdecode;
mod bencode;
mod error;
mod bytestring;

fn main() {
    let inp = "d1:ad2:xyd20:abcdefghij0123456789i555eeee";
    let decoded = bdecode::decode(inp.as_bytes());
    println!("using manuel parser => {:?}", decoded);
}
