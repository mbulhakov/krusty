use bytes::Bytes;
use chrono::{prelude::*, Duration};
use rand::seq::SliceRandom;
use std::cmp::Ordering;
use teloxide::types::MessageId;
use teloxide::{prelude::*, types::InputFile, Bot};

use crate::database::repository::AsyncRepository;
use crate::database::types::{MediaInfo, MediaType};

use super::cache::media_data_by_name;

macro_rules! send_with_caption {
    ($request:expr, $caption:expr, $message_id:expr) => {{
        let r = $request.caption($caption.unwrap_or_default());
        send!(r, $message_id);
    }};
}

macro_rules! send {
    ($request:expr, $message_id:expr) => {{
        let r = $request.disable_notification(true);
        if $message_id.is_some() {
            r.reply_to_message_id($message_id.unwrap()).await?;
        } else {
            r.send().await?;
        }
    }};
}

pub async fn send_media(
    media: &MediaInfo,
    repository: &mut AsyncRepository,
    bot: Bot,
    chat_id: ChatId,
    message_id: Option<MessageId>,
    caption: Option<String>,
) -> anyhow::Result<()> {
    let data = media_data_by_name(repository, &media.name).await?;

    match media.type_ {
        MediaType::Voice => {
            send_with_caption!(
                bot.send_voice(
                    chat_id,
                    InputFile::memory(Bytes::from(data)).file_name(media.name.clone())
                ),
                caption,
                message_id
            )
        }
        MediaType::Picture => {
            send_with_caption!(
                bot.send_photo(
                    chat_id,
                    InputFile::memory(Bytes::from(data)).file_name(media.name.clone())
                ),
                caption,
                message_id
            )
        }
        MediaType::Video => {
            send_with_caption!(
                bot.send_video(
                    chat_id,
                    InputFile::memory(Bytes::from(data)).file_name(media.name.clone())
                ),
                caption,
                message_id
            )
        }
        MediaType::Animation => {
            send_with_caption!(
                bot.send_animation(
                    chat_id,
                    InputFile::memory(Bytes::from(data)).file_name(media.name.clone())
                ),
                caption,
                message_id
            )
        }
        MediaType::PlainText => send!(
            bot.send_message(
                chat_id,
                String::from_utf8(data).expect("Failed to convert from bytes"),
            ),
            message_id
        ),
        MediaType::Document => {
            send_with_caption!(
                bot.send_document(
                    chat_id,
                    InputFile::memory(Bytes::from(data)).file_name(media.name.clone())
                ),
                caption,
                message_id
            )
        }
        MediaType::VideoNote => send!(
            bot.send_video_note(
                chat_id,
                InputFile::memory(Bytes::from(data)).file_name(media.name.clone())
            ),
            message_id
        ),
        MediaType::Sticker => send!(
            bot.send_sticker(
                chat_id,
                InputFile::memory(Bytes::from(data)).file_name(media.name.clone())
            ),
            message_id.map(|x| x.0)
        ),
        MediaType::Unknown => log::error!("Unknown media file type, check DB"),
    }

    Ok(())
}

pub fn get_random_media_info(media_infos: &[MediaInfo]) -> Option<&MediaInfo> {
    media_infos.choose(&mut rand::thread_rng())
}

pub fn is_time_passed(datetime: &DateTime<Utc>, duration: &Duration) -> bool {
    Utc::now().signed_duration_since(*datetime).cmp(duration) == Ordering::Greater
}
