use chrono::prelude::*;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct Msg {
    pub username: String,
    pub command: String,
    pub channel: String,
    pub content: String,
    pub timestamp: chrono::DateTime<Utc>,
}

impl Msg {
    pub fn new() -> Self {
        Self {
            username: String::new(),
            command: String::new(),
            channel: String::new(),
            content: String::new(),
            timestamp: Utc::now(),
        }
    }

    pub fn parse_message(data: &str) -> Self {
        lazy_static! {
            static ref RE: Regex = {
                let pattern = [
                    r":(?P<username>\w*)!\w*@\w*.tmi.twitch.tv ",
                    r"(?P<command>PRIVMSG) ",
                    r"(?P<channel>#\w*) :(?P<content>.+)",
                ]
                .join("");

                Regex::new(&pattern).unwrap()
            };
        }

        RE.captures(data).map_or_else(Self::new, |msg| Self {
            username: msg["username"].to_string(),
            command: msg["command"].to_string(),
            channel: msg["channel"].to_string(),
            content: msg["content"].to_string().replace('\r', ""),
            timestamp: Utc::now(),
        })
    }
}
