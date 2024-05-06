

fn main() {
    let s = "abc";

    let (x, y) = s.split_at(4);
    dbg!(x, y);
}