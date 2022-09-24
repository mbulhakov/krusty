// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "media_feature_type"))]
    pub struct MediaFeatureType;

    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "media_type"))]
    pub struct MediaType;

    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "tag_type"))]
    pub struct TagType;
}

diesel::table! {
    forwarded_messages (id) {
        id -> Int4,
        chat_id -> Int8,
        forwarded_message_id -> Int4,
        message_url -> Varchar,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::MediaType;

    media (id) {
        id -> Int4,
        name -> Varchar,
        #[sql_name = "type"]
        type_ -> MediaType,
        data -> Bytea,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::MediaFeatureType;

    media_to_feature (id) {
        id -> Int4,
        media_id -> Int4,
        feature_type -> MediaFeatureType,
    }
}

diesel::table! {
    tag_to_media (tag_id, media_id) {
        tag_id -> Int4,
        media_id -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::TagType;

    tags (id) {
        id -> Int4,
        text -> Varchar,
        #[sql_name = "type"]
        type_ -> TagType,
    }
}

diesel::joinable!(media_to_feature -> media (media_id));
diesel::joinable!(tag_to_media -> media (media_id));
diesel::joinable!(tag_to_media -> tags (tag_id));

diesel::allow_tables_to_appear_in_same_query!(
    forwarded_messages,
    media,
    media_to_feature,
    tag_to_media,
    tags,
);
