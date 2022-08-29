use krusty::prefetch::gachi::ogg;
use std::boxed::Box;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let data = ogg().await?;
    for key in data.keys() {
        println!("{}", key);
    }
    Ok(())
}
