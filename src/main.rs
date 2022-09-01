use bytes::Bytes;
use krusty::prefetch::gachi::ogg;
use krusty::similar::find_similar;
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use teloxide::{prelude::*, types::InputFile};

struct Ctx {
    gachi: HashMap<String, Bytes>,
    chat_gachi_time: Mutex<HashMap<ChatId, Instant>>,
}

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

    let ctx = Arc::new(Ctx {
        gachi: gachi_ogg,
        chat_gachi_time: Mutex::new(HashMap::new()),
    });

    let handler = dptree::entry().branch(Update::filter_message().endpoint(answer));

    Dispatcher::builder(bot, handler)
        // Pass the shared state to the handler as a dependency.
        .dependencies(dptree::deps![ctx])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

async fn answer(
    message: Message,
    bot: AutoSend<Bot>,
    ctx: Arc<Ctx>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    log::trace!("Message from chat '{}'", message.chat.title().unwrap());

    if ctx.gachi.is_empty() {
        return Ok(());
    }

    {
        log::debug!("Locking gachi mutex");
        let mut chat_times = ctx.chat_gachi_time.lock().unwrap();
        if let Some(time) = chat_times.get(&message.chat.id) {
            if time.elapsed() < Duration::from_secs(30) {
                return Ok(());
            }
        }
        chat_times.insert(message.chat.id, Instant::now());
    }

    if let Some(text) = message.text() {
        if let Some(mut id) = find_similar(text) {
            id.push_str(".ogg");
            if let Some(ogg) = ctx.gachi.get(&id) {
                bot.send_voice(message.chat.id, InputFile::memory(ogg.clone()))
                    .reply_to_message_id(message.id)
                    .await?;
            } else {
                log::warn!("{} was not found", id);
            }
        }
    }

    Ok(())
}
