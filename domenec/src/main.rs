mod bencode;

fn main() {
    let inp = "d1:ad2:id20:abcdefghij0123456789e";
    let decoded = bencode::decode(inp);
    println!("{:?}", decoded);
}
