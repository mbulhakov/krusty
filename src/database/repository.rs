use deadpool::managed::Pool;
use diesel::{insert_into, prelude::*};
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, RunQueryDsl};
use tracing_attributes::instrument;

use crate::database::types;
use crate::schema::forwarded_messages;
use crate::schema::forwarded_messages::dsl::*;
use crate::schema::media;
use crate::schema::media::dsl::*;

#[derive(Clone)]
pub struct AsyncRepository {
    pool: Pool<AsyncDieselConnectionManager<diesel_async::AsyncPgConnection>>,
}

impl AsyncRepository {
    pub fn new(pool: Pool<AsyncDieselConnectionManager<diesel_async::AsyncPgConnection>>) -> Self {
        AsyncRepository { pool }
    }

    #[instrument(skip(self))]
    pub async fn tags(&mut self) -> anyhow::Result<Vec<types::Tag>> {
        use crate::schema::tags;
        use crate::schema::tags::dsl::*;

        let mut conn = self.pool.get().await?;

        Ok(tags
            .select((tags::text, tags::type_, tags::for_whole_text))
            .load::<types::Tag>(&mut *conn)
            .await?)
    }

    #[instrument(skip(self))]
    pub async fn media_info_by_tag_text(
        &mut self,
        t: &str,
    ) -> anyhow::Result<Vec<types::MediaInfo>> {
        use crate::schema::tag_to_media;
        use crate::schema::tags::dsl::*;

        let mut conn = self.pool.get().await?;

        Ok(tags
            .filter(text.eq(t))
            .inner_join(tag_to_media::table.inner_join(media::table))
            .select((media::name, media::type_))
            .load::<types::MediaInfo>(&mut *conn)
            .await?)
    }

    #[instrument(skip(self))]
    pub async fn media_data_by_name(&mut self, n: &str) -> anyhow::Result<Vec<u8>> {
        let mut conn = self.pool.get().await?;

        Ok(media
            .filter(name.eq(n))
            .select(media::data)
            .first::<Vec<u8>>(&mut *conn)
            .await?)
    }

    #[instrument(skip(self))]
    pub async fn forwarded_message_by_ids(
        &mut self,
        c_id: i64,
        fwd_chat_id: i64,
        msg_id: i32,
    ) -> anyhow::Result<Option<types::ForwardedMessage>> {
        let mut conn = self.pool.get().await?;

        let result = forwarded_messages
            .filter(chat_id.eq(c_id))
            .filter(forwarded_message_id.eq(msg_id))
            .filter(forwarded_chat_id.eq(fwd_chat_id))
            .select((
                forwarded_messages::chat_id,
                forwarded_messages::forwarded_message_id,
                forwarded_messages::message_url,
                forwarded_messages::forwarded_chat_id,
            ))
            .first::<types::ForwardedMessage>(&mut conn)
            .await;

        match result {
            Ok(fm) => Ok(Some(fm)),
            Err(diesel::NotFound) => Ok(None),
            Err(e) => Err(anyhow::Error::new(e)),
        }
    }

    #[instrument(skip(self))]
    pub async fn insert_forward_message(
        &mut self,
        message: &types::ForwardedMessage,
    ) -> anyhow::Result<()> {
        let mut conn = self.pool.get().await?;

        insert_into(forwarded_messages)
            .values(message)
            .execute(&mut *conn)
            .await?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn media_info_by_feature_type(
        &mut self,
        t: types::MediaFeatureType,
    ) -> anyhow::Result<Vec<types::MediaInfo>> {
        use crate::schema::media_to_feature;

        let mut conn = self.pool.get().await?;

        Ok(media
            .inner_join(media_to_feature::table)
            .filter(media_to_feature::feature_type.eq(t))
            .select((media::name, media::type_))
            .load::<types::MediaInfo>(&mut *conn)
            .await?)
    }

    #[instrument(skip(self))]
    pub async fn cron_jobs(&mut self) -> anyhow::Result<Vec<types::CroneJob>> {
        use crate::schema::cron_jobs::dsl::*;

        let mut conn = self.pool.get().await?;

        Ok(cron_jobs.load::<types::CroneJob>(&mut *conn).await?)
    }

    #[instrument(skip(self))]
    pub async fn media_info_by_cron_job_id(
        &mut self,
        id_: i32,
    ) -> anyhow::Result<Vec<types::MediaInfo>> {
        use crate::schema::cron_jobs;
        use crate::schema::cron_jobs::dsl::*;
        use crate::schema::media_to_cron_job;

        let mut conn = self.pool.get().await?;

        Ok(cron_jobs
            .filter(cron_jobs::id.eq(id_))
            .inner_join(media_to_cron_job::table.inner_join(media::table))
            .select((media::name, media::type_))
            .load::<types::MediaInfo>(&mut *conn)
            .await?)
    }
}
