use chrono::{prelude::*, Duration};
use deadpool::managed::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use percentage::{PercentageDecimal, PercentageInteger};
use std::collections::HashMap;
use teloxide::prelude::*;
use tokio::sync::Mutex;

use crate::database::repository::AsyncRepository;

pub struct Ctx {
    pub text_trigger_timestamps: Mutex<HashMap<ChatId, DateTime<Utc>>>,
    pub duplicate_forward_timestamps: Mutex<HashMap<ChatId, DateTime<Utc>>>,
    pub media_timeout: Duration,
    pub media_being_sent_chance: PercentageInteger,
    pub similarity_threshold: PercentageDecimal,
    pub repository: AsyncRepository,
}

impl Ctx {
    pub fn new(
        media_timeout: Duration,
        media_being_sent_chance: PercentageInteger,
        similarity_threshold: PercentageDecimal,
        pool: Pool<AsyncDieselConnectionManager<diesel_async::AsyncPgConnection>>,
    ) -> Self {
        Ctx {
            text_trigger_timestamps: Mutex::new(HashMap::new()),
            duplicate_forward_timestamps: Mutex::new(HashMap::new()),
            media_timeout,
            media_being_sent_chance,
            similarity_threshold,
            repository: AsyncRepository::new(pool),
        }
    }
}
