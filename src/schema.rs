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
    chats (id) {
        id -> Int4,
        chat_id -> Int8,
    }
}

diesel::table! {
    cron_jobs (id) {
        id -> Int4,
        #[max_length = 255]
        pattern -> Varchar,
        chat_id -> Nullable<Int8>,
        #[max_length = 255]
        caption -> Nullable<Varchar>,
        #[max_length = 255]
        description -> Nullable<Varchar>,
    }
}

diesel::table! {
    forwarded_messages (id) {
        id -> Int4,
        chat_id -> Int8,
        forwarded_message_id -> Int4,
        #[max_length = 255]
        message_url -> Varchar,
        forwarded_chat_id -> Int8,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::MediaType;

    media (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        #[sql_name = "type"]
        type_ -> MediaType,
        data -> Bytea,
    }
}

diesel::table! {
    media_to_cron_job (id) {
        id -> Int4,
        media_id -> Int4,
        cron_job_id -> Int4,
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
        #[max_length = 255]
        text -> Varchar,
        #[sql_name = "type"]
        type_ -> TagType,
        for_whole_text -> Bool,
    }
}

diesel::joinable!(media_to_cron_job -> cron_jobs (cron_job_id));
diesel::joinable!(media_to_cron_job -> media (media_id));
diesel::joinable!(media_to_feature -> media (media_id));
diesel::joinable!(tag_to_media -> media (media_id));
diesel::joinable!(tag_to_media -> tags (tag_id));

diesel::allow_tables_to_appear_in_same_query!(
    chats,
    cron_jobs,
    forwarded_messages,
    media,
    media_to_cron_job,
    media_to_feature,
    tag_to_media,
    tags,
);
