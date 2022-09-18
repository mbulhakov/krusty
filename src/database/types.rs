use diesel::prelude::*;
use diesel_derive_enum::*;

#[derive(Debug, PartialEq, Eq, DbEnum)]
#[DieselTypePath = "crate::schema::sql_types::TagType"]
pub enum TagType {
    Ordinary,
    Regexp,
}

#[derive(Queryable)]
pub struct Tag {
    pub text: String,
    pub type_: TagType,
}

#[derive(Debug, PartialEq, Eq, DbEnum, Clone)]
#[DieselTypePath = "crate::schema::sql_types::MediaType"]
pub enum MediaType {
    Voice,
    Video,
    Picture,
}

#[derive(Queryable, Clone)]
pub struct MediaInfo {
    pub name: String,
    pub type_: MediaType,
}
