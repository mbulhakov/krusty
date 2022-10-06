use crate::database::types;
use crate::schema::forwarded_messages;
use crate::schema::forwarded_messages::dsl::*;
use crate::schema::media;
use crate::schema::media::dsl::*;
use diesel::pg::PgConnection;
use diesel::{insert_into, prelude::*};

pub trait Repository {
    fn tags_by_type(
        &mut self,
        type_: types::TagType,
    ) -> Result<Vec<types::Tag>, diesel::result::Error>;

    fn media_info_by_tag_text(
        &mut self,
        name: &str,
    ) -> Result<Vec<types::MediaInfo>, diesel::result::Error>;

    fn media_data_by_name(&mut self, name: &str) -> Result<Vec<u8>, diesel::result::Error>;

    fn forwarded_message_by_ids(
        &mut self,
        chat_id: i64,
        forwarded_chat_id: i64,
        forwarded_message_id: i32,
    ) -> Result<Option<types::ForwardedMessage>, diesel::result::Error>;

    fn insert_forward_message(
        &mut self,
        message: &types::ForwardedMessage,
    ) -> Result<(), diesel::result::Error>;

    fn media_info_by_feature_type(
        &mut self,
        type_: types::MediaFeatureType,
    ) -> Result<Vec<types::MediaInfo>, diesel::result::Error>;
}

pub struct PostgresRepository {
    connection: PgConnection,
}

impl PostgresRepository {
    pub fn new(connection: PgConnection) -> PostgresRepository {
        PostgresRepository { connection }
    }
}

impl Repository for PostgresRepository {
    fn tags_by_type(
        &mut self,
        t: types::TagType,
    ) -> Result<Vec<types::Tag>, diesel::result::Error> {
        use crate::schema::tags;
        use crate::schema::tags::dsl::*;

        tags.filter(type_.eq(t))
            .select((tags::text, tags::type_))
            .load::<types::Tag>(&mut self.connection)
    }

    fn media_info_by_tag_text(
        &mut self,
        t: &str,
    ) -> Result<Vec<types::MediaInfo>, diesel::result::Error> {
        use crate::schema::tag_to_media;
        use crate::schema::tags::dsl::*;

        tags.filter(text.eq(t))
            .inner_join(tag_to_media::table.inner_join(media::table))
            .select((media::name, media::type_))
            .load::<types::MediaInfo>(&mut self.connection)
    }

    fn media_data_by_name(&mut self, n: &str) -> Result<Vec<u8>, diesel::result::Error> {
        media
            .filter(name.eq(n))
            .select(media::data)
            .first::<Vec<u8>>(&mut self.connection)
    }

    fn forwarded_message_by_ids(
        &mut self,
        c_id: i64,
        f_c_id: i64,
        m_id: i32,
    ) -> Result<Option<types::ForwardedMessage>, diesel::result::Error> {
        Ok(forwarded_messages
            .filter(chat_id.eq(c_id))
            .filter(forwarded_message_id.eq(m_id))
            .filter(forwarded_chat_id.eq(f_c_id))
            .select((
                forwarded_messages::chat_id,
                forwarded_messages::forwarded_message_id,
                forwarded_messages::message_url,
                forwarded_messages::forwarded_chat_id,
            ))
            .load::<types::ForwardedMessage>(&mut self.connection)?
            .first()
            .map(|x| x.to_owned()))
    }

    fn insert_forward_message(
        &mut self,
        message: &types::ForwardedMessage,
    ) -> Result<(), diesel::result::Error> {
        insert_into(forwarded_messages)
            .values(message)
            .execute(&mut self.connection)?;
        Ok(())
    }

    fn media_info_by_feature_type(
        &mut self,
        t: types::MediaFeatureType,
    ) -> Result<Vec<types::MediaInfo>, diesel::result::Error> {
        use crate::schema::media_to_feature;

        media
            .inner_join(media_to_feature::table)
            .filter(media_to_feature::feature_type.eq(t))
            .select((media::name, media::type_))
            .load::<types::MediaInfo>(&mut self.connection)
    }
}
