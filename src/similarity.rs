use levenshtein::levenshtein;
use ordered_float::OrderedFloat;
use rand::Rng;
use regex::Regex;
use std::cmp::max;
use std::collections::BTreeMap;
use std::env;

use crate::tag_provider::TagProvider;
use crate::token_provider::TokenProvider;

fn score(x: &str, y: &str) -> f64 {
    levenshtein(x, y) as f64 / max(x.chars().count(), y.chars().count()) as f64
}

fn get_max_score(tag: &str, words: &[String]) -> Option<OrderedFloat<f64>> {
    words
        .iter()
        .map(String::as_str)
        .map(|x| score(tag, x))
        .map(OrderedFloat::from)
        .min()
        .to_owned()
}

fn extract_matched_regexps<'a>(words: &[String], patterns: &'a [String]) -> Vec<&'a str> {
    patterns
        .iter()
        .filter(|r| {
            words.iter().any(|w| {
                Regex::new(r).unwrap().is_match(w)
                    || Regex::new(r).unwrap().is_match(&w.to_lowercase())
            })
        })
        .map(String::as_str)
        .collect()
}

pub fn recognize_tag_in_tokens<T1: TokenProvider, T2: TagProvider>(
    token_provider: &T1,
    tag_provider: &T2,
) -> Option<String> {
    let tokens = token_provider.provide();
    let regexp_tags = tag_provider.regexp_tags();

    let matched_regexps = extract_matched_regexps(&tokens, &regexp_tags);
    if !matched_regexps.is_empty() {
        let mr = &matched_regexps[rand::thread_rng().gen::<usize>() % matched_regexps.len()];
        log::debug!("Found matched regexp: '{}'", mr);
        return Some(mr.to_string());
    }

    let threshold =
        env::var("MAX_ACCEPTED_SCORE_SIMILARITY").map_or_else(|_| 0.25f64, |x| x.parse().unwrap());

    let ordinary_tags = tag_provider.ordinary_tags();
    let mut tags_to_scores = BTreeMap::new();
    for tag in &ordinary_tags {
        if let Some(OrderedFloat(score)) = get_max_score(tag, &tokens) {
            if score <= threshold {
                tags_to_scores
                    .entry(OrderedFloat::from(score))
                    .or_insert(Vec::new())
                    .push(tag);
            }
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unicode_scoring() {
        // expected results as: number of modifications / max len of string
        assert_eq!(score("как", "кам"), 1f64 / 3f64);
        assert_eq!(score("мультикак", "как"), 6f64 / 9f64);
        assert_eq!(score("сума", "ура"), 2f64 / 4f64);
        assert_eq!(score("hhhhрррр", "hр"), 6f64 / 8f64);
    }
}
