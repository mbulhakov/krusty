use bytes::Bytes;
use chrono::{prelude::*, Duration};
use rand::Rng;
use std::cmp::Ordering;
use teloxide::types::MessageId;
use teloxide::{prelude::*, types::InputFile, Bot};

use crate::database::repository::Repository;
use crate::database::types::{self, MediaInfo, MediaType};

pub async fn send_media(
    media: &MediaInfo,
    repository: &mut Repository,
    bot: Bot,
    chat_id: ChatId,
    message_id: MessageId,
    caption: Option<String>,
) -> anyhow::Result<()> {
    let data = repository.media_data_by_name(&media.name).await?;
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

pub async fn get_random_media_info_for_feature_type(
    type_: types::MediaFeatureType,
    repository: &mut Repository,
) -> Option<MediaInfo> {
    let media_infos = match repository.media_info_by_feature_type(type_).await {
        Ok(result) => Some(result),
        Err(e) => {
            log::error!("{e}");
            None
        }
    };

    if let Some(media_infos) = media_infos {
        return Some(media_infos[rand::thread_rng().gen::<usize>() % media_infos.len()].to_owned());
    }

    log::warn!("No media associated with type");
    None
}

pub fn is_time_passed(datetime: &DateTime<Utc>, duration: &Duration) -> bool {
    Utc::now().signed_duration_since(*datetime).cmp(duration) == Ordering::Greater
}
