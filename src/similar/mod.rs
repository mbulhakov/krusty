use levenshtein::levenshtein;
use ordered_float::OrderedFloat;
use serde_json;
use serde_json::Value;
use std::cmp::max;
use std::collections::BTreeMap;

fn is_special(ch: char) -> bool {
    (' '..='/').contains(&ch) || (':'..='@').contains(&ch) || ('\\'..='`').contains(&ch)
}

fn split_on_tokens(message: &str) -> Vec<String> {
    message
        .split(is_special)
        .map(|s| s.to_string().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect::<Vec<String>>()
}

fn get_max_score(tag: &str, words: &[String]) -> Option<OrderedFloat<f64>> {
    words
        .iter()
        .map(|x| x.as_str())
        .map(|x| levenshtein(x, tag) as f64 / max(x.len(), tag.len()) as f64)
        .map(OrderedFloat::from)
        .min()
        .to_owned()
}

struct Entry {
    id: String,
    tag: String,
}

pub fn find_similar(message: &str) -> Option<String> {
    let base: &'static str = r#"
{
  "come on lets go": ["go", "го", "поехали"],
  "Fisting is 300 $": ["фистинг", "300", "баксы", "бакинские"],
  "FUCK YOU": ["нахуй", "впизду", "на хуй", "fuck you"],
  "fuck you...": ["нахуй", "впизду", "на хуй", "fuck you"],
  "Fucking slaves get your ass back here": ["слейвс", "slaves", "♂", "асс", "эс", "ass"],
  "Iam cumming": ["кам", "камминг", "кончаю", "♂"],
  "Id be right happy to": ["радостью", "залюбкы", "залюбки", "happy"],
  "It gets bigger when i pull on it": ["хуй", "тяну", "большой"],
  "Its so fucking deep": ["глубоко", "факинг", "fucking", "♂", "дип", "deep"],
  "Lets suck some dick": ["ура", "поздравляю", "поздравления", "наконец-то", "наконецто"],
  "Oh shit iam sorry": ["сори", "сорян", "извини", "извините", "sorry"],
  "Our daddy told us not to be ashamed": ["стыдно", "стыдоба", "неудобно"],
  "Sometimes i rip the skin:": ["♂"],
  "Sorry for what": ["сори", "сорян", "извини", "извините", "sorry"]
}"#;

    let tokens = split_on_tokens(message);

    let parsed: Value = serde_json::from_str(base).unwrap();
    let obj = parsed.as_object().unwrap().to_owned();

    let mut names_to_scores = BTreeMap::new();
    for (k, v) in obj.iter() {
        let mut scores = Vec::new();
        for tag in v.as_array().unwrap().iter().map(|x| x.to_string()) {
            if let Some(OrderedFloat(score)) = get_max_score(&tag, &tokens) {
                if score <= 0.26f64 {
                    scores.push((OrderedFloat::from(score), k, tag));
                }
            }
        }
        if let Some((OrderedFloat(f), id, tag)) = scores.iter().max().map(|x| x.to_owned()) {
            names_to_scores.insert(
                OrderedFloat::from(f),
                Entry {
                    id: id.clone(),
                    tag,
                },
            );
        }
    }

    match names_to_scores.iter().next() {
        Some(entry) => {
            let (score, Entry { id, tag }) = entry.to_owned();
            log::debug!(
                "Find a similarity, '{}', tag '{}' with score {}, tag: '{}'",
                id,
                tag,
                score,
                message
            );
            Some(id.to_owned())
        }
        None => None,
    }
}
