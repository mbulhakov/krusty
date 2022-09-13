pub mod token_provider;

use levenshtein::levenshtein;
use ordered_float::OrderedFloat;
use rand::Rng;
use regex::Regex;
use serde_json;
use serde_json::Value;
use std::cmp::max;
use std::collections::BTreeMap;

use self::token_provider::TokenProvider;

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

fn maybe_special_case(w: &str) -> Option<&str> {
    if Regex::new(r"^[mм]{2,}$").unwrap().is_match(w) {
        return Some("Mmmmh");
    } else if w.contains('♂') || Regex::new(r"^[аоao]{2,}$").unwrap().is_match(w) {
        let orgasms = vec![
            "Orgasm 1",
            "Orgasm 2",
            "Orgasm 3",
            "Orgasm 4",
            "Orgasm 5",
            "Orgasm 6",
            "RIP EARS ORGASM",
        ];
        return Some(orgasms[rand::thread_rng().gen::<usize>() % orgasms.len()]);
    }

    None
}

fn extract_special_cases(words: &[String]) -> Vec<&str> {
    words
        .iter()
        .map(String::as_str)
        .filter_map(maybe_special_case)
        .collect()
}

struct GachiEntry {
    id: String,
    tag: String,
}

pub fn find_similar<T: TokenProvider>(provider: &T) -> Option<String> {
    let base: &'static str = r#"
{
  "come on lets go": ["go", "го", "поехали", "погнали"],
  "Fisting is 300 $": ["фистинг", "300", "баксы", "бакинские"],
  "FUCK YOU": ["нахуй", "впизду", "fuck you"],
  "fuck you...": ["нахуй", "впизду", "fuck you"],
  "Fucking slaves get your ass back here": ["слейвс", "slaves", "асс", "эс", "ass"],
  "Iam cumming": ["кам", "камминг", "кончаю", "cum", "cumming", "coming", "come"],
  "Swallow my cum": ["кам", "камминг", "кончаю", "cum", "cumming", "coming", "come"],
  "Id be right happy to": ["радостью", "залюбкы", "залюбки", "happy"],
  "It gets bigger when i pull on it": ["хуй", "тяну", "большой"],
  "Its so fucking deep": ["глубоко", "факинг", "fucking", "дип", "deep"],
  "Lets suck some dick": ["ура", "поздравляю", "поздравления", "наконец-то", "наконецто"],
  "Oh shit iam sorry": ["сори", "сорян", "извини", "извините", "sorry"],
  "Our daddy told us not to be ashamed": ["стыдно", "стыдоба", "неудобно", "стыдиться"],
  "Sometimes i rip the skin": ["хуй", "тяну"],
  "Sorry for what": ["сори", "сорян", "извини", "извините", "sorry"]
}"#;

    let tokens = provider.provide_owned();

    let special_cases = extract_special_cases(&tokens);
    if !special_cases.is_empty() {
        let sc = &special_cases[rand::thread_rng().gen::<usize>() % special_cases.len()];
        log::debug!("Found a special case, '{}'", sc);
        return Some(sc.to_string());
    }

    let parsed: Value = serde_json::from_str(base).unwrap();
    let json = parsed.as_object().unwrap().to_owned();

    let mut names_to_scores = BTreeMap::new();

    for (k, v) in json.iter() {
        let mut scores = Vec::new();
        for tag in v.as_array().unwrap().iter().map(|x| x.as_str().unwrap()) {
            if let Some(OrderedFloat(score)) = get_max_score(tag, &tokens) {
                if score <= 0.26f64 {
                    scores.push((OrderedFloat::from(score), k, tag));
                }
            }
        }
        if let Some((OrderedFloat(f), id, tag)) = scores.iter().max().map(|x| x.to_owned()) {
            names_to_scores
                .entry(OrderedFloat::from(f))
                .or_insert(Vec::new())
                .push(GachiEntry {
                    id: id.clone(),
                    tag: tag.to_string(),
                });
        }
    }

    match names_to_scores.iter().next() {
        Some(kv) => {
            let (score, entries) = kv.to_owned();

            let entry = &entries[rand::thread_rng().gen::<usize>() % entries.len()];
            log::debug!(
                "Found a similarity: {{id: '{}', tag '{}', score {}}}",
                entry.id,
                entry.tag,
                score
            );
            Some(entry.id.to_owned())
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
        assert_eq!(score("мультикам", "кам"), 6f64 / 9f64);
        assert_eq!(score("сука", "ура"), 2f64 / 4f64);
        assert_eq!(score("hhhhрррр", "hр"), 6f64 / 8f64);
    }

    #[test]
    fn test_maybe_special_case() {
        assert_eq!(maybe_special_case("ecommerce"), None);
        assert_eq!(maybe_special_case("екомммммерц"), None);
        assert_eq!(maybe_special_case("haaaalloooo"), None);

        assert_ne!(maybe_special_case("мmmм"), None);
        assert_ne!(maybe_special_case("оаoa"), None);
        assert_ne!(maybe_special_case("оооааа"), None);
        assert_ne!(maybe_special_case("string♂"), None);
    }
}