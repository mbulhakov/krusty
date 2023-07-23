mod cache;
mod ctx;
mod features;
mod utils;

use chrono::Duration;
use deadpool::managed::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use percentage::{PercentageDecimal, PercentageInteger};
use std::sync::Arc;
use teloxide::prelude::*;

use self::ctx::Ctx;
use self::features::dupl_checker::send_media_if_forwarded_before;
use self::features::scheduled_messages::create_scheduler;
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

    let mut maybe_scheduler = create_scheduler(bot.clone(), ctx.repository.clone()).await;
    match &mut maybe_scheduler {
        Ok(scheduler) => {
            // Spawns own task on start, requires some sort of event loop to be active; Dispatcher does this job bellow
            if let Err(e) = scheduler.start().await {
                log::error!("Failed to start message scheduler: '{e}'");
            }
        }
        Err(e) => log::error!("Failed to create message scheduler: '{e}'"),
    }

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

    let _ = tokio::spawn(async move {
        Dispatcher::builder(bot.clone(), handler)
            .dependencies(dptree::deps![ctx])
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await
    })
    .await;
}
