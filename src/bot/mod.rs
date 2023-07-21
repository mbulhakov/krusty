mod cache;
mod ctx;
mod features;
mod utils;

use chrono::Duration;
use deadpool::managed::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use futures::future;
use percentage::{PercentageDecimal, PercentageInteger};
use std::sync::Arc;
use teloxide::prelude::*;

use self::ctx::Ctx;
use self::features::dupl_checker::send_media_if_forwarded_before;
use self::features::scheduled_messages::start_messages_scheduling;
use self::features::tag_detector::send_media_on_text_trigger;
use self::utils::is_time_passed;

pub async fn start_bot(
    bot: teloxide::Bot,
    media_timeout: Duration,
    ignore_message_older_than: Duration,
    media_being_sent_chance: PercentageInteger,
    similarity_threshold: PercentageDecimal,
    pool: Pool<AsyncDieselConnectionManager<diesel_async::AsyncPgConnection>>,
) {
    let ctx = Arc::new(Ctx::new(
        media_timeout,
        media_being_sent_chance,
        similarity_threshold,
        pool,
    ));

    let scheduler_task = tokio::spawn(start_messages_scheduling(
        bot.clone(),
        ctx.repository.clone(),
    ));

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
    let listener_task = tokio::spawn(async move {
        Dispatcher::builder(bot.clone(), handler)
            .dependencies(dptree::deps![ctx])
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await
    });

    future::join_all([listener_task, scheduler_task]).await;
}
