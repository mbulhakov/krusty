use krusty::prefetch::gachi::mp3;
use std::boxed::Box;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let data = mp3().await?;
    for key in data.keys() {
        println!("{}", key);
    }
    Ok(())
}
