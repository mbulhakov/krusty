use bytes::Bytes;
use krusty::prefetch::gachi::ogg;
use rand::Rng;
use std::collections::HashMap;
use std::convert::Into;
use std::error::Error;
use std::sync::Arc;
use teloxide::{prelude::*, types::InputFile, utils::command::BotCommands};

#[derive(BotCommands, Clone)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "Display this text")]
    Help,
    #[command(description = "Says hello to fucking slaves")]
    Hello,
    #[command(description = "Sends a voice mail to fucking slaves")]
    Gachi,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting throw dice bot...");

    let bot = Bot::from_env().auto_send();

    let gachi_ogg: Vec<Bytes> = match ogg().await {
        Ok(m) => m,
        Err(e) => {
            log::warn!("Failed to get gachi ogg: {}", e);
            HashMap::new()
        }
    }
    .values()
    .cloned()
    .collect();

    let gachi_ogg = Arc::new(gachi_ogg);

    let handler = Update::filter_message().branch(
        dptree::entry()
            // Filter commands: the next handlers will receive a parsed `Command`.
            .filter_command::<Command>()
            // If a command parsing fails, this handler will not be executed.
            .endpoint(answer),
    );

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
    command: Command,
    gachi_ogg: Arc<Vec<Bytes>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match command {
        Command::Help => {
            bot.send_message(message.chat.id, Command::descriptions().to_string())
                .await?
        }
        Command::Hello => {
            bot.send_message(message.chat.id, "Hello, ♂slaves♂!")
                .await?
        }
        Command::Gachi => {
            if gachi_ogg.is_empty() {
                bot.send_message(message.chat.id, "Fuck you, ♂slaves♂!")
                    .await?
            } else {
                let idx = rand::thread_rng().gen::<usize>() % gachi_ogg.len();
                bot.send_voice(message.chat.id, InputFile::memory(gachi_ogg[idx].clone()))
                    .await?
            }
        }
    };

    Ok(())
}
