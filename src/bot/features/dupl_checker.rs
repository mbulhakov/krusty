use anyhow::anyhow;
use chrono::prelude::*;
use std::sync::Arc;
use teloxide::{prelude::*, Bot};

use crate::{
    bot::{
        cache::media_info_by_feature_type,
        ctx::Ctx,
        utils::{get_random_media_info, is_time_passed, send_media},
    },
    database::types::{ForwardedMessage, MediaFeatureType},
};

pub async fn send_media_if_forwarded_before(
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

    let mut repository = ctx.repository.clone();
    let forwarded_message = repository
        .forwarded_message_by_ids(chat_id.0, forwarded_chat_id, forwarded_message_id)
        .await?;

    if let Some(forwarded_message) = forwarded_message {
        let media_infos = media_info_by_feature_type(
            &mut repository,
            MediaFeatureType::DuplicatedForwardedMessageDetection,
        )
        .await?;

        if let Some(media) = get_random_media_info(&media_infos) {
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
                media,
                &mut repository,
                bot,
                chat_id,
                Some(message_id),
                Some(forwarded_message.message_url),
            )
            .await?
        }
    } else {
        repository
            .insert_forward_message(&ForwardedMessage {
                chat_id: chat_id.0,
                forwarded_message_id,
                message_url: message_url.to_string(),
                forwarded_chat_id,
            })
            .await?;
    }

    Ok(())
}
