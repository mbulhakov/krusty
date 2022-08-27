use krusty::prefetch;

fn main() {
    let data = prefetch::mp3();
    for key in data.keys() {
        println!("{}", key);
    }
}
