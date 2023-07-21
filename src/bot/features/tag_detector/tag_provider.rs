use mockall::automock;

use crate::bot::cache::tags;
use crate::database::{repository::AsyncRepository, types::*};

pub struct Tag {
    pub text: String,
    pub is_regexp: bool,
    pub for_whole_text: bool,
}

#[automock]
pub trait TagProvider {
    fn tags(&self) -> &[Tag];
}

pub struct RepositoryTagProvider {
    tags: Vec<Tag>,
}

impl RepositoryTagProvider {
    pub async fn new(repository: &mut AsyncRepository) -> anyhow::Result<Self> {
        let tags = tags(repository)
            .await?
            .into_iter()
            .map(|t| Tag {
                text: t.text,
                is_regexp: t.type_ == TagType::Regexp,
                for_whole_text: t.for_whole_text,
            })
            .collect();

        Ok(RepositoryTagProvider { tags })
    }
}

impl TagProvider for RepositoryTagProvider {
    fn tags(&self) -> &[Tag] {
        &self.tags
    }
}
