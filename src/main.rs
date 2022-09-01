use bytes::Bytes;
use krusty::prefetch::gachi::ogg;
use krusty::similar::find_similar;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use teloxide::{prelude::*, types::InputFile};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting bot...");

    let bot = Bot::from_env().auto_send();

    let gachi_ogg: HashMap<String, Bytes> = match ogg().await {
        Ok(m) => m,
        Err(e) => {
            log::warn!("Failed to get gachi ogg: {}", e);
            HashMap::new()
        }
    };

    let gachi_ogg = Arc::new(gachi_ogg);

    let handler = Update::filter_message().branch(dptree::entry().endpoint(answer));

    Dispatcher::builder(bot, handler)
        // Pass the shared state to the handler as a dependency.
        .dependencies(dptree::deps![gachi_ogg])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

async fn answer(
    message: Message,
    bot: AutoSend<Bot>,
    gachi_ogg: Arc<HashMap<String, Bytes>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if gachi_ogg.is_empty() {
        return Ok(());
    }

    if let Some(mut id) = find_similar(message.text().unwrap()) {
        id.push_str(".ogg");
        if let Some(ogg) = gachi_ogg.get(&id) {
            bot.send_voice(message.chat.id, InputFile::memory(ogg.clone()))
                .await?;
        }
    }

    Ok(())
}
