use mockall::automock;
use teloxide::prelude::*;
use teloxide::types::MessageEntityKind;

fn is_special(ch: char) -> bool {
    (' '..='/').contains(&ch) || (':'..='@').contains(&ch) || ('\\'..='`').contains(&ch)
}

#[automock]
pub trait TokenProvider {
    fn provide(&self) -> Vec<String>;
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

        let urls = entities
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
            .map(str::to_string)
            .filter(|s| !s.is_empty())
            .collect()
    }
}
