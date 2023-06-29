use crate::database::types;
use crate::schema::forwarded_messages;
use crate::schema::forwarded_messages::dsl::*;
use crate::schema::media;
use crate::schema::media::dsl::*;
use bb8::Pool;
use diesel::{insert_into, prelude::*};
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, RunQueryDsl};

pub struct Repository {
    pool: Pool<AsyncDieselConnectionManager<diesel_async::AsyncPgConnection>>,
}

impl Repository {
    pub fn new(
        pool: Pool<AsyncDieselConnectionManager<diesel_async::AsyncPgConnection>>,
    ) -> Repository {
        Repository { pool }
    }

    pub async fn tags_by_type(
        &mut self,
        t: types::TagType,
    ) -> Result<Vec<types::Tag>, diesel::result::Error> {
        use crate::schema::tags;
        use crate::schema::tags::dsl::*;

        let mut conn = self.pool.get().await.unwrap();

        tags.filter(type_.eq(t))
            .select((tags::text, tags::type_))
            .load::<types::Tag>(&mut *conn)
            .await
    }

    pub async fn media_info_by_tag_text(
        &mut self,
        t: &str,
    ) -> Result<Vec<types::MediaInfo>, diesel::result::Error> {
        use crate::schema::tag_to_media;
        use crate::schema::tags::dsl::*;

        let mut conn = self.pool.get().await.unwrap();

        tags.filter(text.eq(t))
            .inner_join(tag_to_media::table.inner_join(media::table))
            .select((media::name, media::type_))
            .load::<types::MediaInfo>(&mut *conn)
            .await
    }

    pub async fn media_data_by_name(&mut self, n: &str) -> Result<Vec<u8>, diesel::result::Error> {
        let mut conn = self.pool.get().await.unwrap();

        media
            .filter(name.eq(n))
            .select(media::data)
            .first::<Vec<u8>>(&mut *conn)
            .await
    }

    pub async fn forwarded_message_by_ids(
        &mut self,
        c_id: i64,
        f_c_id: i64,
        m_id: i32,
    ) -> Result<Option<types::ForwardedMessage>, diesel::result::Error> {
        let mut conn = self.pool.get().await.unwrap();

        let result = forwarded_messages
            .filter(chat_id.eq(c_id))
            .filter(forwarded_message_id.eq(m_id))
            .filter(forwarded_chat_id.eq(f_c_id))
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
            Err(e) => Err(e),
        }
    }

    pub async fn insert_forward_message(
        &mut self,
        message: &types::ForwardedMessage,
    ) -> Result<(), diesel::result::Error> {
        let mut conn = self.pool.get().await.unwrap();

        insert_into(forwarded_messages)
            .values(message)
            .execute(&mut *conn)
            .await?;
        Ok(())
    }

    pub async fn media_info_by_feature_type(
        &mut self,
        t: types::MediaFeatureType,
    ) -> Result<Vec<types::MediaInfo>, diesel::result::Error> {
        use crate::schema::media_to_feature;

        let mut conn = self.pool.get().await.unwrap();

        media
            .inner_join(media_to_feature::table)
            .filter(media_to_feature::feature_type.eq(t))
            .select((media::name, media::type_))
            .load::<types::MediaInfo>(&mut *conn)
            .await
    }
}
