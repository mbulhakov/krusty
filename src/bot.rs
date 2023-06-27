use anyhow::anyhow;
use bytes::Bytes;
use chrono::{prelude::*, Duration};

use diesel::{Connection, PgConnection};
use rand::Rng;
use tokio::sync::Mutex;

use percentage::PercentageInteger;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use teloxide::types::MessageId;
use teloxide::{prelude::*, types::InputFile, Bot};

use crate::database::repository::{PostgresRepository, Repository};
use crate::database::types::{self, MediaFeatureType, MediaInfo, MediaType};
use crate::similarity::recognize_tag_in_tokens;
use crate::tag_provider::RepositoryTagProvider;
use crate::token_provider::MessageTokenProvider;

pub async fn start_bot(
    bot: teloxide::Bot,
    media_timeout: Duration,
    ignore_message_older_than: Duration,
    media_being_sent_chance: PercentageInteger,
) {
    let ctx = Arc::new(Ctx::new(media_timeout, media_being_sent_chance));

    let handler = Update::filter_message()
        .filter(|msg: Message, _: Arc<Ctx>| msg.chat.is_supergroup())
        .branch(
            dptree::filter(|msg: Message, _: Arc<Ctx>| {
                msg.forward_from_message_id().is_some() && msg.forward_from_chat().is_some()
            })
            .endpoint(send_media_if_forwarded_before),
        )
        .branch(
            dptree::filter(move |msg: Message, _: Arc<Ctx>| {
                !is_time_passed(&msg.date, &ignore_message_older_than)
            })
            .endpoint(send_media_on_text_trigger),
        );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![ctx])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

struct Ctx {
    text_trigger_timestamps: Mutex<HashMap<ChatId, DateTime<Utc>>>,
    duplicate_forward_timestamps: Mutex<HashMap<ChatId, DateTime<Utc>>>,
    media_timeout: Duration,
    media_being_sent_chance: PercentageInteger,
}

impl Ctx {
    pub fn new(media_timeout: Duration, media_being_sent_chance: PercentageInteger) -> Ctx {
        Ctx {
            text_trigger_timestamps: Mutex::new(HashMap::new()),
            duplicate_forward_timestamps: Mutex::new(HashMap::new()),
            media_timeout,
            media_being_sent_chance,
        }
    }
}

async fn send_media_on_text_trigger(
    message: Message,
    bot: Bot,
    ctx: Arc<Ctx>,
) -> anyhow::Result<()> {
    {
        log::debug!("Locking text trigger chat mutex");
        let mut chat_times = ctx.text_trigger_timestamps.lock().await;
        if let Some(time) = chat_times.get(&message.chat.id) {
            if !is_time_passed(time, &ctx.media_timeout) {
                return Ok(());
            }
        }
        chat_times.insert(message.chat.id, Utc::now());
    }

    let mut repository = PostgresRepository::new(connection());
    let tag_provider = RepositoryTagProvider::new(&mut repository)?;

    let chat_id = message.chat.id;
    let message_id = message.id;
    let token_provider = MessageTokenProvider::new(message);
    if let Some(tag) = recognize_tag_in_tokens(&token_provider, &tag_provider) {
        if let Some(media) = get_random_media_info_for_tag(&tag, &mut repository) {
            if should_media_be_sent(&ctx.media_being_sent_chance) {
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
    bot: Bot,
    ctx: Arc<Ctx>,
) -> anyhow::Result<()> {
    let chat_id = message.chat.id;
    let message_id = message.id;
    let message_url = message
        .url()
        .expect("Message link should be obtainable if the bot is used in supergroup");

    let forwarded_message_id = message
        .forward_from_message_id()
        .ok_or_else(|| anyhow!("Non-forwarded message is handled in 'forward-only' handler"))?;
    let forwarded_chat_id = message
        .forward_from_chat()
        .ok_or_else(|| anyhow!("Non-forwarded message is handled in 'forward-only' handler"))?
        .id
        .0;

    let mut repository = PostgresRepository::new(connection());
    let forwarded_message =
        repository.forwarded_message_by_ids(chat_id.0, forwarded_chat_id, forwarded_message_id)?;

    if let Some(forwarded_message) = forwarded_message {
        if let Some(media) = get_random_media_info_for_feature_type(
            MediaFeatureType::DuplicatedForwardedMessageDetection,
            &mut repository,
        ) {
            {
                log::debug!("Locking duplicate forward chat mutex");
                let mut chat_times = ctx.duplicate_forward_timestamps.lock().await;
                if let Some(time) = chat_times.get(&message.chat.id) {
                    if !is_time_passed(time, &ctx.media_timeout) {
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
            forwarded_chat_id,
        })?;
    }

    Ok(())
}

async fn send_media<T: Repository>(
    media: &MediaInfo,
    repository: &mut T,
    bot: Bot,
    chat_id: ChatId,
    message_id: MessageId,
    caption: Option<String>,
) -> anyhow::Result<()> {
    let data = repository.media_data_by_name(&media.name)?;
    match media.type_ {
        MediaType::Voice => {
            bot.send_voice(chat_id, InputFile::memory(Bytes::from(data)))
                .caption(caption.unwrap_or_default())
                .disable_notification(true)
                .reply_to_message_id(message_id)
                .await?;
        }
        MediaType::Picture => {
            bot.send_photo(chat_id, InputFile::memory(Bytes::from(data)))
                .caption(caption.unwrap_or_default())
                .disable_notification(true)
                .reply_to_message_id(message_id)
                .await?;
        }
        MediaType::Video => {
            bot.send_video(chat_id, InputFile::memory(Bytes::from(data)))
                .caption(caption.unwrap_or_default())
                .disable_notification(true)
                .reply_to_message_id(message_id)
                .await?;
        }
    }

    Ok(())
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

fn is_time_passed(datetime: &DateTime<Utc>, duration: &Duration) -> bool {
    Utc::now().signed_duration_since(*datetime).cmp(duration) == Ordering::Greater
}

fn connection() -> PgConnection {
    let uri = env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    log::debug!("PG uri: {}", uri);
    PgConnection::establish(&uri).expect("Failed to obtain connection")
}

fn should_media_be_sent(media_being_sent_chance: &PercentageInteger) -> bool {
    rand::thread_rng().gen_range(0..100) >= (100 - (*media_being_sent_chance).value())
}
