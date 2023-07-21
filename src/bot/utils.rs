use bytes::Bytes;
use chrono::{prelude::*, Duration};
use rand::Rng;
use std::cmp::Ordering;
use teloxide::types::MessageId;
use teloxide::{prelude::*, types::InputFile, Bot};

use crate::database::repository::AsyncRepository;
use crate::database::types::{MediaInfo, MediaType};

use super::cache::media_data_by_name;

macro_rules! send {
    ($request:expr, $caption:expr, $message_id:expr) => {
        if $message_id.is_some() {
            $request
                .caption($caption.unwrap_or_default())
                .disable_notification(true)
                .reply_to_message_id($message_id.unwrap())
                .await?;
        } else {
            $request
                .caption($caption.unwrap_or_default())
                .disable_notification(true)
                .send()
                .await?;
        }
    };
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
            let r = bot.send_voice(chat_id, InputFile::memory(Bytes::from(data)));
            send!(r, caption, message_id)
        }
        MediaType::Picture => {
            let r = bot.send_photo(chat_id, InputFile::memory(Bytes::from(data)));
            send!(r, caption, message_id)
        }
        MediaType::Video => {
            let r = bot.send_video(chat_id, InputFile::memory(Bytes::from(data)));
            send!(r, caption, message_id)
        }
    }

    Ok(())
}

pub fn get_random_media_info(media_infos: &[MediaInfo]) -> Option<&MediaInfo> {
    if media_infos.is_empty() {
        return None;
    }

    Some(&media_infos[rand::thread_rng().gen::<usize>() % media_infos.len()])
}

pub fn is_time_passed(datetime: &DateTime<Utc>, duration: &Duration) -> bool {
    Utc::now().signed_duration_since(*datetime).cmp(duration) == Ordering::Greater
}
