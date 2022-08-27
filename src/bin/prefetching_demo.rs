use krusty::prefetch;
use std::boxed::*;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let data = prefetch::mp3()?;
    for key in data.keys() {
        println!("{}", key);
    }
    Ok(())
}
