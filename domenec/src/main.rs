mod bencode_nom;
mod bencode;
mod bencode_fast;

fn main() {
    // "d1:ad2:xyd20:abcdefghij0123456789i555eee"
    let inp = "d1:ad2:xyd20:abcdefghij0123456789i555eeee";
    let decoded = bencode_nom::decode(inp);
    println!("using nom parser => {:?}", decoded);
    let decoded = bencode_fast::decode(inp.as_bytes());
    println!("using manuel parser => {:?}", decoded);
}
