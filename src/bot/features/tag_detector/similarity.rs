use fancy_regex::Regex;
use levenshtein::levenshtein;
use mockall::predicate::*;
use ordered_float::OrderedFloat;
use ordslice::Ext;
use percentage::PercentageDecimal;
use rand::Rng;
use std::cmp::max;
use std::collections::BTreeMap;
use tracing_unwrap::ResultExt;

use super::tag_provider::{Tag, TagProvider};
use super::token_provider::TokenProvider;

pub fn recognize_tag_in_tokens(
    token_provider: &impl TokenProvider,
    tag_provider: &impl TagProvider,
    similarity_threshold: &PercentageDecimal,
) -> Option<String> {
    let tokens = token_provider
        .provide()
        .iter()
        .map(|x| x.to_lowercase())
        .collect::<Vec<_>>();
    let tokens = tokens.iter().map(|x| x.as_str()).collect::<Vec<_>>();

    let mut tags = tag_provider.tags().iter().collect::<Vec<_>>();
    tags.sort_by(|a, b| a.is_regexp.cmp(&b.is_regexp));

    let (ordinary_tags, regexp_tags) =
        tags.split_at(tags.lower_bound_by(|x| x.is_regexp.cmp(&true)));

    let matched_regexps = process_regexp_tags(regexp_tags, token_provider.source(), &tokens);
    if !matched_regexps.is_empty() {
        let mr = &matched_regexps[rand::thread_rng().gen::<usize>() % matched_regexps.len()];
        log::debug!("Found matched regexp: '{}'", mr);
        return Some(mr.to_string());
    }

    let tags_to_scores = process_ordinary_tags(
        ordinary_tags,
        token_provider.source(),
        &tokens,
        similarity_threshold,
    );

    match tags_to_scores.iter().next() {
        Some(kv) => {
            let (score, tags) = kv.to_owned();

            let tag = &tags[rand::thread_rng().gen::<usize>() % tags.len()];
            log::debug!("Found a similarity: {{ tag '{}', score {} }}", tag, score);
            Some(tag.to_string())
        }
        None => None,
    }
}

fn process_ordinary_tags<'a>(
    tags: &[&'a Tag],
    source_text: &str,
    tokens: &[&str],
    similarity_threshold: &PercentageDecimal,
) -> BTreeMap<OrderedFloat<f64>, Vec<&'a str>> {
    let mut token_tags = Vec::new();
    let mut source_text_tags = Vec::new();
    for t in tags {
        if t.for_whole_text {
            source_text_tags.push(t.text.as_str());
        } else {
            token_tags.push(t.text.as_str());
        }
    }

    let matches = extract_matched_tags(&[source_text], &source_text_tags, similarity_threshold);
    if !matches.is_empty() {
        return matches;
    }

    extract_matched_tags(tokens, &token_tags, similarity_threshold)
}

fn extract_matched_tags<'a>(
    tokens: &[&str],
    tags: &[&'a str],
    similarity_threshold: &PercentageDecimal,
) -> BTreeMap<OrderedFloat<f64>, Vec<&'a str>> {
    let mut tags_to_scores: BTreeMap<OrderedFloat<f64>, Vec<&'a str>> = BTreeMap::new();
    for tag in tags {
        if let Some(OrderedFloat(score)) = get_min_score(tag, tokens) {
            if score <= similarity_threshold.value() {
                tags_to_scores
                    .entry(OrderedFloat::from(score))
                    .or_insert(Vec::new())
                    .push(tag);
            }
        }
    }

    tags_to_scores
}

fn score(x: &str, y: &str) -> f64 {
    levenshtein(x, y) as f64 / max(x.chars().count(), y.chars().count()) as f64
}

fn process_regexp_tags<'a>(tags: &[&'a Tag], source_text: &str, tokens: &[&str]) -> Vec<&'a str> {
    let mut token_tags = Vec::new();
    let mut source_text_tags = Vec::new();
    for t in tags {
        if t.for_whole_text {
            source_text_tags.push(t.text.as_str());
        } else {
            token_tags.push(t.text.as_str());
        }
    }

    let matches = extract_matched_regexps(&[source_text], &source_text_tags);
    if !matches.is_empty() {
        return matches;
    }

    extract_matched_regexps(tokens, &token_tags)
}

fn extract_matched_regexps<'a>(tokens: &[&str], patterns: &[&'a str]) -> Vec<&'a str> {
    patterns
        .iter()
        .filter(|r| {
            let regexp = Regex::new(r);
            match regexp {
                Ok(regexp) => tokens.iter().any(|w| {
                    regexp.is_match(w).unwrap_or_log()
                        || regexp.is_match(&w.to_lowercase()).unwrap_or_log()
                }),
                Err(e) => {
                    log::warn!("Failed to compile regex '{}', skipping. Cause: {}", r, e);
                    false
                }
            }
        })
        .copied()
        .collect()
}

fn get_min_score(tag: &str, tokens: &[&str]) -> Option<OrderedFloat<f64>> {
    tokens
        .iter()
        .map(|x| score(x, tag))
        .map(OrderedFloat::from)
        .min()
}

//
// TESTS
//

#[cfg(test)]
mod tests {
    use percentage::Percentage;

    use crate::bot::features::tag_detector::{
        similarity::{recognize_tag_in_tokens, score},
        tag_provider::{MockTagProvider, Tag},
        token_provider::MockTokenProvider,
    };

    fn token_tag(text: &str) -> Tag {
        Tag {
            text: text.to_string(),
            is_regexp: false,
            for_whole_text: false,
        }
    }

    fn text_tag(text: &str) -> Tag {
        Tag {
            text: text.to_string(),
            is_regexp: false,
            for_whole_text: true,
        }
    }

    fn regexp_token_tag(text: &str) -> Tag {
        Tag {
            text: text.to_string(),
            is_regexp: true,
            for_whole_text: false,
        }
    }

    fn regexp_text_tag(text: &str) -> Tag {
        Tag {
            text: text.to_string(),
            is_regexp: true,
            for_whole_text: true,
        }
    }

    #[test]
    fn test_unicode_scoring() {
        // expected results as: number of modifications / max len of string
        assert_eq!(score("как", "кам"), 1f64 / 3f64);
        assert_eq!(score("мультикак", "как"), 6f64 / 9f64);
        assert_eq!(score("сума", "ура"), 2f64 / 4f64);
        assert_eq!(score("hhhhрррр", "hр"), 6f64 / 8f64);
    }

    #[test]
    fn test_valid_unicode_regexp_on_tokens() {
        let mut tag_provider = MockTagProvider::new();
        tag_provider.expect_tags().return_const(vec![
            regexp_token_tag("^wrong$"),
            regexp_token_tag("^правильный$"),
        ]);

        let mut token_provider = MockTokenProvider::new();

        let source = "This is the правильный token";
        token_provider
            .expect_source()
            .return_const(source.to_string());
        token_provider
            .expect_provide()
            .return_once(move || source.split(' ').map(str::to_string).collect::<Vec<_>>());

        let actual = recognize_tag_in_tokens(
            &token_provider,
            &tag_provider,
            &Percentage::from_decimal(0.25f64),
        );
        assert_eq!(actual, Some("^правильный$".to_string()));
    }

    #[test]
    fn test_invalid_regexp_on_tokens() {
        let mut tag_provider = MockTagProvider::new();
        tag_provider.expect_tags().return_const(vec![
            regexp_token_tag("^[is+$"),
            regexp_token_tag("^right$"),
        ]);

        let source = "This is the right token";
        let mut token_provider = MockTokenProvider::new();
        token_provider
            .expect_source()
            .return_const(source.to_string());
        token_provider
            .expect_provide()
            .return_once(|| source.split(' ').map(str::to_string).collect::<Vec<_>>());

        let actual = recognize_tag_in_tokens(
            &token_provider,
            &tag_provider,
            &Percentage::from_decimal(0.25f64),
        );
        assert_eq!(actual, Some("^right$".to_string()));
    }

    #[test]
    fn test_valid_unicode_regexp_on_tokens_case_insensitive() {
        let mut tag_provider = MockTagProvider::new();
        tag_provider.expect_tags().return_const(vec![
            regexp_token_tag("^wrong$"),
            regexp_token_tag("^правильный$"),
        ]);

        let source = "THIS IS THE ПРАВИЛЬНЫЙ TOKEN";
        let mut token_provider = MockTokenProvider::new();
        token_provider
            .expect_source()
            .return_const(source.to_string());
        token_provider
            .expect_provide()
            .return_once(|| source.split(' ').map(str::to_string).collect::<Vec<_>>());

        let actual = recognize_tag_in_tokens(
            &token_provider,
            &tag_provider,
            &Percentage::from_decimal(0.25f64),
        );
        assert_eq!(actual, Some("^правильный$".to_string()));
    }

    #[test]
    fn test_valid_unicode_ordinary_tag_on_tokens_case() {
        let mut tag_provider = MockTagProvider::new();
        tag_provider
            .expect_tags()
            .return_const(vec![token_tag("праильнй"), token_tag("si")]);

        let source = "THIS IS THE ПРАВИЛЬНЫЙ TOKEN";
        let mut token_provider = MockTokenProvider::new();

        token_provider
            .expect_source()
            .return_const(source.to_string());
        token_provider
            .expect_provide()
            .return_once(|| source.split(' ').map(str::to_string).collect::<Vec<_>>());

        let actual = recognize_tag_in_tokens(
            &token_provider,
            &tag_provider,
            &Percentage::from_decimal(0.25f64),
        );
        assert_eq!(actual, Some("праильнй".to_string()));
    }

    #[test]
    fn test_no_tags_on_tokens_case() {
        let mut tag_provider = MockTagProvider::new();
        tag_provider.expect_tags().return_const(Vec::new());

        let mut token_provider = MockTokenProvider::new();
        let source = "THIS IS THE ПРАВИЛЬНЫЙ TOKEN";
        token_provider
            .expect_source()
            .return_const(source.to_string());
        token_provider
            .expect_provide()
            .return_once(|| source.split(' ').map(str::to_string).collect::<Vec<_>>());

        let actual = recognize_tag_in_tokens(
            &token_provider,
            &tag_provider,
            &Percentage::from_decimal(0.25f64),
        );
        assert_eq!(actual, None);
    }

    #[test]

    fn test_both_regexp_and_ordinary_tags_on_tokens_case() {
        let mut tag_provider = MockTagProvider::new();
        tag_provider.expect_tags().return_const(vec![
            regexp_token_tag("^token$"),
            token_tag("this"),
            regexp_token_tag("^wrong$"),
            token_tag("is"),
        ]);

        let mut token_provider = MockTokenProvider::new();
        let source = "this is the right token";
        token_provider
            .expect_source()
            .return_const(source.to_string());
        token_provider
            .expect_provide()
            .return_once(|| source.split(' ').map(str::to_string).collect::<Vec<_>>());

        let actual = recognize_tag_in_tokens(
            &token_provider,
            &tag_provider,
            &Percentage::from_decimal(0.25f64),
        );
        assert_eq!(actual, Some("^token$".to_string()));
    }

    #[test]
    fn text_regexp_token_and_text_tag() {
        let mut tag_provider = MockTagProvider::new();
        tag_provider.expect_tags().return_const(vec![
            regexp_token_tag(".*"),
            regexp_text_tag(r#"^(?=(?:[^\p{Ll}]*[\p{Lu}]){2})[^\p{Ll}]+$"#), // match string if all letters, including utf-8 ones, are uppercase
        ]);

        let mut token_provider = MockTokenProvider::new();
        let source = r#"АХАХАWERTE@$%!#$ТПОНЇЪ !!!!!4565655БЯЯЬTER!!!@$%%$##$"#;
        token_provider
            .expect_source()
            .return_const(source.to_string());
        token_provider
            .expect_provide()
            .return_once(|| source.split(' ').map(str::to_string).collect::<Vec<_>>());

        let actual = recognize_tag_in_tokens(
            &token_provider,
            &tag_provider,
            &Percentage::from_decimal(0.25f64),
        );
        assert_eq!(
            actual,
            Some(r#"^(?=(?:[^\p{Ll}]*[\p{Lu}]){2})[^\p{Ll}]+$"#.to_string())
        );
    }

    #[test]
    fn text_ordinary_token_and_text_tag() {
        let mut tag_provider = MockTagProvider::new();
        tag_provider.expect_tags().return_const(vec![
            token_tag("token"),
            text_tag("this is the right tokee"),
        ]);

        let mut token_provider = MockTokenProvider::new();
        let source = "this is the right token";
        token_provider
            .expect_source()
            .return_const(source.to_string());
        token_provider
            .expect_provide()
            .return_once(|| source.split(' ').map(str::to_string).collect::<Vec<_>>());

        let actual = recognize_tag_in_tokens(
            &token_provider,
            &tag_provider,
            &Percentage::from_decimal(0.25f64),
        );
        assert_eq!(actual, Some("this is the right tokee".to_string()));
    }
}
