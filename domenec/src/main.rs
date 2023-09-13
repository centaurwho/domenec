mod bencode_nom;

fn main() {
    // "d1:ad2:xyd20:abcdefghij0123456789i555eee"
    let inp = "d1:ad2:xyd20:abcdefghij0123456789i555eeee";
    let decoded = bencode_nom::decode(inp);
    println!("{:?}", decoded);
}
