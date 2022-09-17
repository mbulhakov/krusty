use crate::database::types;
use diesel::pg::PgConnection;
use diesel::prelude::*;

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
        use crate::schema::media;
        use crate::schema::tag_to_media;
        use crate::schema::tags::dsl::*;

        tags.filter(text.eq(t))
            .inner_join(tag_to_media::table.inner_join(media::table))
            .select((media::name, media::type_))
            .load::<types::MediaInfo>(&mut self.connection)
    }

    fn media_data_by_name(&mut self, n: &str) -> Result<Vec<u8>, diesel::result::Error> {
        use crate::schema::media;
        use crate::schema::media::dsl::*;

        media
            .filter(name.eq(n))
            .select(media::data)
            .first::<Vec<u8>>(&mut self.connection)
    }
}
