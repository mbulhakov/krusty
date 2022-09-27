use crate::database::{repository::Repository, types::*};

pub trait TagProvider {
    fn ordinary_tags(&self) -> Vec<String>;
    fn regexp_tags(&self) -> Vec<String>;
}

pub struct RepositoryTagProvider {
    ordinary_tags: Vec<String>,
    regexp_tags: Vec<String>,
}

impl RepositoryTagProvider {
    pub fn new<T: Repository>(repository: &mut T) -> anyhow::Result<RepositoryTagProvider> {
        Ok(RepositoryTagProvider {
            ordinary_tags: repository
                .tags_by_type(TagType::Ordinary)?
                .iter()
                .map(|x| x.text.to_owned())
                .collect(),
            regexp_tags: repository
                .tags_by_type(TagType::Regexp)?
                .iter()
                .map(|x| x.text.to_owned())
                .collect(),
        })
    }
}

impl TagProvider for RepositoryTagProvider {
    fn ordinary_tags(&self) -> Vec<String> {
        self.ordinary_tags.clone()
    }
    fn regexp_tags(&self) -> Vec<String> {
        self.regexp_tags.clone()
    }
}
