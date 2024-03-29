mod similarity;
mod tag_provider;
mod token_provider;

use chrono::prelude::*;
use percentage::PercentageInteger;
use rand::Rng;
use std::sync::Arc;
use teloxide::{prelude::*, Bot};

use crate::bot::cache::media_info_by_tag_text;
use crate::bot::ctx::Ctx;
use crate::bot::utils::{choose_random_media_info, is_time_passed, send_media};
use crate::database::repository::AsyncRepository;
use crate::database::types::MediaInfo;

use self::similarity::recognize_tag_in_tokens;
use self::tag_provider::RepositoryTagProvider;
use self::token_provider::MessageTokenProvider;

pub async fn send_media_on_text_trigger(
    message: Message,
    bot: Bot,
    ctx: Arc<Ctx>,
) -> anyhow::Result<()> {
    {
        log::debug!("Locking text trigger chat mutex");
        let mut chat_times = ctx.text_trigger_timestamps.lock().await;
        if let Some(time) = chat_times.get(&message.chat.id) {
            if !is_time_passed(time, &ctx.media_timeout) {
                log::debug!(
                    "There is a timeout for chat '{}', skipping",
                    message.chat.id
                );
                return Ok(());
            }
        }
        chat_times.insert(message.chat.id, Utc::now());
    }

    let mut repository = ctx.repository.clone();
    let tag_provider = RepositoryTagProvider::new(&mut repository).await?;

    let chat_id = message.chat.id;
    let message_id = message.id;
    let token_provider = MessageTokenProvider::new(message);
    if let Some(tag) =
        recognize_tag_in_tokens(&token_provider, &tag_provider, &ctx.similarity_threshold)
    {
        if let Some(media) = get_random_media_info_for_tag(&tag, &mut repository).await {
            if should_media_be_sent(&ctx.media_being_sent_chance) {
                send_media(
                    &media,
                    &mut repository,
                    bot,
                    chat_id,
                    Some(message_id),
                    None,
                )
                .await?
            } else {
                log::debug!("Match was found, but omitted due to low chance");
            }
        }
    }

    Ok(())
}

async fn get_random_media_info_for_tag(
    tag: &str,
    repository: &mut AsyncRepository,
) -> Option<MediaInfo> {
    let media_infos = match media_info_by_tag_text(repository, tag).await {
        Ok(res) => Some(res),
        Err(e) => {
            log::error!("{e}");
            None
        }
    };

    if let Some(media_infos) = media_infos {
        return choose_random_media_info(&media_infos).cloned();
    }

    log::warn!("No media associated with tag");
    None
}

fn should_media_be_sent(media_being_sent_chance: &PercentageInteger) -> bool {
    rand::thread_rng().gen_range(0..100) >= (100 - (*media_being_sent_chance).value())
}
