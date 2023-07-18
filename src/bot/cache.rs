use cached::proc_macro::cached;
use cached::{SizedCache, TimedCache, TimedSizedCache};

use crate::database::{repository::Repository, types};

#[cached(
    type = "TimedCache<String, Vec<types::Tag>>",
    create = "{ TimedCache::with_lifespan(3600) }",
    result = true,
    convert = "{ String::default() }"
)]
pub async fn tags(r: &mut Repository) -> anyhow::Result<Vec<types::Tag>> {
    r.tags().await
}

#[cached(
    type = "TimedSizedCache<String, Vec<types::MediaInfo>>",
    create = "{ TimedSizedCache::with_size_and_lifespan(50, 3600) }",
    result = true,
    convert = r#"{ format!("{t}") }"#
)]
pub async fn media_info_by_tag_text(
    r: &mut Repository,
    t: &str,
) -> anyhow::Result<Vec<types::MediaInfo>> {
    r.media_info_by_tag_text(t).await
}

#[cached(
    type = "SizedCache<String, Vec<u8>>",
    create = "{ SizedCache::with_size(100) }",
    result = true,
    convert = r#"{ format!("{n}") }"#
)]
pub async fn media_data_by_name(r: &mut Repository, n: &str) -> anyhow::Result<Vec<u8>> {
    r.media_data_by_name(n).await
}

#[cached(
    type = "TimedCache<String, Vec<types::MediaInfo>>",
    create = "{ TimedCache::with_lifespan(3600) }",
    result = true,
    convert = r#"{ format!("{:?}", t) }"#
)]
pub async fn media_info_by_feature_type(
    r: &mut Repository,
    t: types::MediaFeatureType,
) -> anyhow::Result<Vec<types::MediaInfo>> {
    r.media_info_by_feature_type(t).await
}
