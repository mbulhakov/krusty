use krusty::similar::find_similar;

use std::fs;

fn main() {
    pretty_env_logger::init();

    let words = fs::read_to_string(r"D:\New Text Document.txt").unwrap();

    let res = find_similar(&words);

    res;
}
