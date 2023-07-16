use diesel::prelude::*;
use diesel_derive_enum::*;

use crate::schema::forwarded_messages;

#[derive(Debug, PartialEq, Eq, DbEnum)]
#[DieselTypePath = "crate::schema::sql_types::TagType"]
pub enum TagType {
    Ordinary,
    Regexp,
}

#[derive(Debug, PartialEq, Eq, DbEnum, Clone)]
#[DieselTypePath = "crate::schema::sql_types::MediaType"]
pub enum MediaType {
    Voice,
    Video,
    Picture,
}

#[derive(Debug, PartialEq, Eq, DbEnum)]
#[DieselTypePath = "crate::schema::sql_types::MediaFeatureType"]
pub enum MediaFeatureType {
    TextTrigger,
    DuplicatedForwardedMessageDetection,
}

#[derive(Queryable)]
pub struct Tag {
    pub text: String,
    pub type_: TagType,
    pub for_whole_text: bool,
}

#[derive(Queryable, Clone)]
pub struct MediaInfo {
    pub name: String,
    pub type_: MediaType,
}

#[derive(Queryable, Clone, Insertable)]
#[diesel(table_name = forwarded_messages)]
pub struct ForwardedMessage {
    pub chat_id: i64,
    pub forwarded_message_id: i32,
    pub message_url: String,
    pub forwarded_chat_id: i64,
}
