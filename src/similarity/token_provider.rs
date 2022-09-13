use teloxide::prelude::*;
use teloxide::types::MessageEntityKind;

fn is_special(ch: char) -> bool {
    (' '..='/').contains(&ch) || (':'..='@').contains(&ch) || ('\\'..='`').contains(&ch)
}

pub trait TokenProvider {
    fn provide_owned(&self) -> Vec<String>;
}

pub struct MessageTokenProvider {
    message: Message,
}

impl MessageTokenProvider {
    pub fn new(message: Message) -> MessageTokenProvider {
        MessageTokenProvider { message }
    }
}

impl TokenProvider for MessageTokenProvider {
    fn provide_owned(&self) -> Vec<String> {
        let urls = self
            .message
            .parse_entities()
            .or_else(|| self.message.parse_caption_entities())
            .unwrap_or_default()
            .iter()
            .filter(|x| x.kind() == &MessageEntityKind::Url)
            .map(|x| x.text())
            .collect::<Vec<_>>();

        let text = self
            .message
            .text()
            .or_else(|| self.message.caption())
            .unwrap_or_default()
            .to_string();

        let text = urls.iter().fold(text, |t, u| t.replace(u, ""));

        text.split(is_special)
            .map(str::to_lowercase)
            .filter(|s| !s.is_empty())
            .collect()
    }
}
