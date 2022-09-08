use levenshtein::levenshtein;
use std::cmp::max;

fn score(x: &str, y: &str) -> f64 {
    println!("l({x}, {y}): {}", levenshtein(x, y));
    println!("{}, {}", x.len(), y.len());
    levenshtein(x, y) as f64 / max(x.len(), y.len()) as f64
}

fn main() {
    for _ in "Как вы заебали Я тебе за войсуху мультикам только готов вместо мультимапы"
        .to_lowercase()
        .split(' ')
        .map(|x| println!("{x}: {}", score(x, "кам")))
    {}
}
