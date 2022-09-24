use bytes::Bytes;
use chrono::{prelude::*, Duration};
use diesel::pg::PgConnection;
use diesel::Connection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use rand::Rng;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::{convert::Infallible, net::SocketAddr};
use teloxide::{prelude::*, types::InputFile};

use krusty::database::repository::{PostgresRepository, Repository};
use krusty::database::types::{self, MediaFeatureType, MediaInfo, MediaType};
use krusty::similarity::recognize_tag_in_tokens;
use krusty::tag_provider::RepositoryTagProvider;
use krusty::token_provider::MessageTokenProvider;

struct Ctx {
    text_trigger_timestamps: Mutex<HashMap<ChatId, DateTime<Utc>>>,
    duplicate_forward_timestamps: Mutex<HashMap<ChatId, DateTime<Utc>>>,
    media_timeout_sec: i64,
}

fn is_time_passed(datetime: &DateTime<Utc>, duration: &Duration) -> bool {
    Utc::now().signed_duration_since(*datetime).cmp(duration) == Ordering::Greater
}

async fn handle(_: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from("")))
}

fn connection() -> PgConnection {
    let uri = env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    log::debug!("PG uri: {}", uri);
    PgConnection::establish(&uri).expect("Failed to obtain connection")
}

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

fn run_migrations() {
    let mut connection = connection();
    connection.run_pending_migrations(MIGRATIONS).unwrap();
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting bot...");
    let bot = Bot::from_env().auto_send();

    let media_timeout_sec =
        env::var("MEDIA_TIMEOUT_SEC").map_or_else(|_| 30, |x| x.parse().unwrap());
    let ignore_message_older_then_sec =
        env::var("IGNORE_MESSAGE_OLDER_THEN_SEC").map_or_else(|_| 60, |x| x.parse().unwrap());

    if media_timeout_sec > ignore_message_older_then_sec {
        panic!("Voice timeout is greater then 'ignore message older then': {media_timeout_sec} > {ignore_message_older_then_sec}");
    }

    run_migrations();

    let ctx = Arc::new(Ctx {
        text_trigger_timestamps: Mutex::new(HashMap::new()),
        duplicate_forward_timestamps: Mutex::new(HashMap::new()),
        media_timeout_sec,
    });

    let handler = Update::filter_message()
        .filter(|msg: Message, _: Arc<Ctx>| msg.chat.is_supergroup())
        .branch(
            dptree::filter(|msg: Message, _: Arc<Ctx>| msg.forward_from_message_id().is_some())
                .endpoint(send_media_if_forwarded_before),
        )
        .branch(
            dptree::filter(move |msg: Message, _: Arc<Ctx>| {
                !is_time_passed(&msg.date, &Duration::seconds(ignore_message_older_then_sec))
            })
            .endpoint(send_media_on_text_trigger),
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

fn get_random_media_info_for_tag<T: Repository>(
    tag: &str,
    repository: &mut T,
) -> Option<MediaInfo> {
    let media_infos = repository.media_info_by_tag_text(tag).unwrap();

    if !media_infos.is_empty() {
        return Some(media_infos[rand::thread_rng().gen::<usize>() % media_infos.len()].to_owned());
    }
    log::warn!("No media associated with '{}'", tag);
    None
}

fn get_random_media_info_for_feature_type<T: Repository>(
    type_: types::MediaFeatureType,
    repository: &mut T,
) -> Option<MediaInfo> {
    let media_infos = repository.media_info_by_feature_type(type_).unwrap();

    if !media_infos.is_empty() {
        return Some(media_infos[rand::thread_rng().gen::<usize>() % media_infos.len()].to_owned());
    }
    log::warn!("No media associated with type");
    None
}

async fn send_media<T: Repository>(
    media: &MediaInfo,
    repository: &mut T,
    bot: AutoSend<Bot>,
    chat_id: ChatId,
    message_id: i32,
    caption: Option<String>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let data = repository
        .media_data_by_name(&media.name)
        .expect("Data was not found for voice media");
    match media.type_ {
        MediaType::Voice => {
            bot.send_voice(chat_id, InputFile::memory(Bytes::from(data)))
                .caption(caption.unwrap_or_else(|| "".to_string()))
                .reply_to_message_id(message_id)
                .await?;
        }
        MediaType::Picture => {
            bot.send_photo(chat_id, InputFile::memory(Bytes::from(data)))
                .caption(caption.unwrap_or_else(|| "".to_string()))
                .reply_to_message_id(message_id)
                .await?;
        }
        MediaType::Video => {
            bot.send_video(chat_id, InputFile::memory(Bytes::from(data)))
                .caption(caption.unwrap_or_else(|| "".to_string()))
                .reply_to_message_id(message_id)
                .await?;
        }
    }

    Ok(())
}

fn should_media_sending_trigger() -> bool {
    let threshold =
        env::var("MEDIA_SEND_CHANCE_IN_PERCENT").map_or_else(|_| 50, |x| x.parse().unwrap());
    rand::thread_rng().gen_range(0..100) >= (100 - threshold)
}

async fn send_media_on_text_trigger(
    message: Message,
    bot: AutoSend<Bot>,
    ctx: Arc<Ctx>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    {
        log::debug!("Locking text trigger chat mutex");
        let mut chat_times = ctx.text_trigger_timestamps.lock().unwrap();
        if let Some(time) = chat_times.get(&message.chat.id) {
            if !is_time_passed(time, &Duration::seconds(ctx.media_timeout_sec)) {
                return Ok(());
            }
        }
        chat_times.insert(message.chat.id, Utc::now());
    }

    let mut repository = PostgresRepository::new(connection());
    let tag_provider = RepositoryTagProvider::new(&mut repository);

    let chat_id = message.chat.id;
    let message_id = message.id;
    let token_provider = MessageTokenProvider::new(message);
    if let Some(tag) = recognize_tag_in_tokens(&token_provider, &tag_provider) {
        if let Some(media) = get_random_media_info_for_tag(&tag, &mut repository) {
            if should_media_sending_trigger() {
                send_media(&media, &mut repository, bot, chat_id, message_id, None).await?
            } else {
                log::debug!("Match was found, but omitted due to low chance");
            }
        }
    }

    Ok(())
}

async fn send_media_if_forwarded_before(
    message: Message,
    bot: AutoSend<Bot>,
    ctx: Arc<Ctx>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let chat_id = message.chat.id;
    let message_id = message.id;
    let message_url = message
        .url()
        .expect("Message link should be obtained if the bot is used in supergroup");
    let forwarded_message_id = message
        .forward_from_message_id()
        .expect("Non-forwarded message is handled in 'forward-only' handler");

    let mut repository = PostgresRepository::new(connection());
    let forwarded_message = repository.forwarded_message_by_ids(chat_id.0, forwarded_message_id)?;

    if let Some(forwarded_message) = forwarded_message {
        if let Some(media) = get_random_media_info_for_feature_type(
            MediaFeatureType::DuplicatedForwardedMessageDetection,
            &mut repository,
        ) {
            {
                log::debug!("Locking duplicate forward chat mutex");
                let mut chat_times = ctx.duplicate_forward_timestamps.lock().unwrap();
                if let Some(time) = chat_times.get(&message.chat.id) {
                    if !is_time_passed(time, &Duration::seconds(ctx.media_timeout_sec)) {
                        return Ok(());
                    }
                }
                chat_times.insert(message.chat.id, Utc::now());
            }

            send_media(
                &media,
                &mut repository,
                bot,
                chat_id,
                message_id,
                Some(forwarded_message.message_url),
            )
            .await?
        }
    } else {
        repository.insert_forward_message(&types::ForwardedMessage {
            chat_id: chat_id.0,
            forwarded_message_id,
            message_url: message_url.to_string(),
        })?;
    }

    Ok(())
}
