use mockall::automock;
use teloxide::prelude::*;
use teloxide::types::MessageEntityKind;

fn is_separator(ch: char) -> bool {
    matches!(ch, ' '..='/' | ':'..='@' | '\\'..='`')
}

#[automock]
pub trait TokenProvider {
    fn provide(&self) -> Vec<String>;
    fn source(&self) -> &str;
}

pub struct MessageTokenProvider {
    message: Message,
}

impl MessageTokenProvider {
    pub fn new(message: Message) -> Self {
        MessageTokenProvider { message }
    }
}

impl TokenProvider for MessageTokenProvider {
    fn provide(&self) -> Vec<String> {
        let entities = self
            .message
            .parse_entities()
            .or_else(|| self.message.parse_caption_entities())
            .unwrap_or_default();

        let url_iter = entities
            .iter()
            .filter(|x| x.kind() == &MessageEntityKind::Url)
            .map(|x| x.text());

        split_text(self.source(), url_iter)
    }

    fn source(&self) -> &str {
        self.message
            .text()
            .or_else(|| self.message.caption())
            .unwrap_or_default()
    }
}

fn split_text<'a>(
    mut source: &str,
    mut excluded_ordered_substrings: impl Iterator<Item = &'a str>,
) -> Vec<String> {
    let mut text_chunks = Vec::new();

    while !source.is_empty() {
        if let Some(url) = excluded_ordered_substrings.next() {
            let idx = source.find(url);
            if idx.is_none() {
                log::warn!("Url was not found in text/caption");
                continue;
            }

            let idx = idx.unwrap();
            text_chunks.push(&source[..idx]);
            source = &source[idx + url.len()..];
        } else {
            text_chunks.push(source);
            break;
        }
    }

    text_chunks
        .iter()
        .flat_map(|x| x.split(is_separator))
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::split_text;

    #[test]
    fn test_text_few_exclusion() {
        let actual = split_text(
            "some random text with exc1 custom exc2 exclusions",
            vec!["exc1", "exc2"].iter().copied(),
        );

        let expected = vec!["some", "random", "text", "with", "custom", "exclusions"];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_text_few_exclusion_wrong_order() {
        let actual = split_text(
            "some random text with exc1 custom exc2 exclusions",
            vec!["exc2", "exc1"].iter().copied(),
        );

        let expected = vec![
            "some",
            "random",
            "text",
            "with",
            "exc1", // this one will be omitted cause exc2 was being looked for
            "custom",
            "exclusions",
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_text_no_exclusion() {
        let actual = split_text("some random text", std::iter::empty());

        let expected = vec!["some", "random", "text"];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_no_text_no_exclusion() {
        let actual = split_text("", std::iter::empty());

        let expected = Vec::<&str>::new();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_no_text_few_exclusions() {
        let actual = split_text("", vec!["exc2", "exc1"].iter().copied());

        let expected = Vec::<&str>::new();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_text_exclusion_at_the_end() {
        let actual = split_text("some random text exc2", vec!["exc2"].iter().copied());

        let expected = vec!["some", "random", "text"];
        assert_eq!(actual, expected);
    }
}
