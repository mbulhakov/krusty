use bytes::Bytes;
use chrono::{prelude::*, Duration};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use krusty::prefetch::gachi::ogg;
use krusty::similarity::{find_similar, token_provider};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::{convert::Infallible, net::SocketAddr};
use teloxide::{prelude::*, types::InputFile};

struct Ctx {
    gachi: HashMap<String, Bytes>,
    chat_gachi_time: Mutex<HashMap<ChatId, DateTime<Utc>>>,
    gachi_timeout_sec: i64,
}

fn is_time_passed(datetime: &DateTime<Utc>, duration: &Duration) -> bool {
    Utc::now().signed_duration_since(*datetime).cmp(duration) == Ordering::Greater
}

async fn handle(_: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from("")))
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

    let gachi_timeout_sec =
        env::var("GACHI_TIMEOUT_SEC").map_or_else(|_| 30, |x| x.parse().unwrap());
    let ignore_message_older_then_sec =
        env::var("IGNORE_MESSAGE_OLDER_THEN_SEC").map_or_else(|_| 60, |x| x.parse().unwrap());

    if gachi_timeout_sec > ignore_message_older_then_sec {
        panic!("Voice timeout is greater then 'ignore message older then': {gachi_timeout_sec} > {ignore_message_older_then_sec}");
    }

    let ctx = Arc::new(Ctx {
        gachi: gachi_ogg,
        chat_gachi_time: Mutex::new(HashMap::new()),
        gachi_timeout_sec,
    });

    let handler = Update::filter_message().branch(
        dptree::filter(move |msg: Message, _: Arc<Ctx>| {
            !is_time_passed(&msg.date, &Duration::seconds(ignore_message_older_then_sec))
        })
        .endpoint(answer),
    );

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let make_service = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle)) });
    let _ = Server::bind(&addr).serve(make_service);

    Dispatcher::builder(bot, handler)
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
            if !is_time_passed(time, &Duration::seconds(ctx.gachi_timeout_sec)) {
                return Ok(());
            }
        }
        chat_times.insert(message.chat.id, Utc::now());
    }

    let chat_id = message.chat.id;
    let message_id = message.id;
    let token_provider = token_provider::MessageTokenProvider::new(message);
    if let Some(mut id) = find_similar(&token_provider) {
        id.push_str(".ogg");
        if let Some(ogg) = ctx.gachi.get(&id) {
            bot.send_voice(chat_id, InputFile::memory(ogg.clone()))
                .reply_to_message_id(message_id)
                .await?;
        } else {
            log::warn!("{} was not found", id);
        }
    }

    Ok(())
}
