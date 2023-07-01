use regex::Regex;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Tag {
    pub badge_info: String,
    pub badges: String,
    pub bits: String,
    pub client_nonce: String,
    pub color: String,
    pub display_name: String,
    pub emote_only: String,
    pub emotes: String,
    pub first_msg: i32,
    pub flags: String,
    pub id: String,
    pub is_mod: i32,
    pub reply_parent_display_name: String,
    pub reply_parent_msg_body: String,
    pub reply_parent_msg_id: String,
    pub reply_parent_user_id: String,
    pub reply_parent_user_login: String,
    pub returning_chatter: i32,
    pub room_id: String,
    pub subscriber: i32,
    pub tags_raw: String,
    pub tmi_sent_ts: String,
    pub turbo: i32,
    pub user_id: String,
    pub user_type: String,
    pub vip: String,
}

impl Tag {
    pub const fn new() -> Self {
        Self {
            badge_info: String::new(),
            badges: String::new(),
            bits: String::new(),
            client_nonce: String::new(),
            color: String::new(),
            display_name: String::new(),
            emote_only: String::new(),
            emotes: String::new(),
            first_msg: 0,
            flags: String::new(),
            id: String::new(),
            is_mod: 0,
            reply_parent_display_name: String::new(),
            reply_parent_msg_body: String::new(),
            reply_parent_msg_id: String::new(),
            reply_parent_user_id: String::new(),
            reply_parent_user_login: String::new(),
            returning_chatter: 0,
            room_id: String::new(),
            subscriber: 0,
            tags_raw: String::new(),
            tmi_sent_ts: String::new(),
            turbo: 0,
            user_id: String::new(),
            user_type: String::new(),
            vip: String::new(),
        }
    }

    pub fn capture_tags(data: &str) -> String {
        lazy_static! {
            static ref RE: Regex = {
                let pattern = r"@(?P<tags>(.+)) :\w*!\w*@\w*.tmi.twitch.tv .+";

                Regex::new(pattern).unwrap()
            };
        }

        RE.captures(data).map_or_else(|| String::new(), |msg| msg["tags"].to_string())
    }

    // https://dev.twitch.tv/docs/irc/tags
    pub fn parse_tags(data: &str) -> Self {
        let raw_tags = Self::capture_tags(data);
        let raw_tags: Vec<&str> = raw_tags.split(';').collect();
        let mut tags = HashMap::new();

        if raw_tags.len() > 1 {
            for tag in raw_tags {
                let sep = tag.find('=').unwrap();
                let tag_name = &tag[0..sep];
                let tag_value = &tag[sep + 1..];

                tags.insert(tag_name.to_string(), tag_value.to_string().replace(r"\s", " "));
            }

            let tags_raw = serde_json::to_string(&tags).unwrap();

            Self {
                badge_info: tags
                    .get("badge-info")
                    .map_or(String::new(), std::string::ToString::to_string),
                badges: tags.get("badges").map_or(String::new(), std::string::ToString::to_string),
                bits: tags.get("bits").map_or(String::new(), std::string::ToString::to_string),
                client_nonce: tags
                    .get("client-nonce")
                    .map_or(String::new(), std::string::ToString::to_string),
                color: tags.get("color").map_or(String::new(), std::string::ToString::to_string),
                display_name: tags
                    .get("display-name")
                    .map_or(String::new(), std::string::ToString::to_string),
                emote_only: tags
                    .get("emote-only")
                    .map_or(String::new(), std::string::ToString::to_string),
                emotes: tags.get("emotes").map_or(String::new(), std::string::ToString::to_string),
                first_msg: tags.get("first-msg").map_or(0, |x| x.parse::<i32>().unwrap_or(0)),
                flags: tags.get("flags").map_or(String::new(), std::string::ToString::to_string),
                id: tags.get("id").map_or(String::new(), std::string::ToString::to_string),
                is_mod: tags.get("mod").map_or(0, |x| x.parse::<i32>().unwrap_or(0)),
                reply_parent_display_name: tags
                    .get("reply-parent-display-name")
                    .map_or(String::new(), std::string::ToString::to_string),
                reply_parent_msg_body: tags
                    .get("reply-parent-msg-body")
                    .map_or(String::new(), std::string::ToString::to_string),
                reply_parent_msg_id: tags
                    .get("reply-parent-msg-id")
                    .map_or(String::new(), std::string::ToString::to_string),
                reply_parent_user_id: tags
                    .get("reply-parent-user-id")
                    .map_or(String::new(), std::string::ToString::to_string),
                reply_parent_user_login: tags
                    .get("reply-parent-user-login")
                    .map_or(String::new(), std::string::ToString::to_string),
                returning_chatter: tags
                    .get("returning-chatter")
                    .map_or(0, |x| x.parse::<i32>().unwrap_or(0)),
                room_id: tags
                    .get("room-id")
                    .map_or(String::new(), std::string::ToString::to_string),
                subscriber: tags.get("subscriber").map_or(0, |x| x.parse::<i32>().unwrap_or(0)),
                tags_raw,
                tmi_sent_ts: tags
                    .get("tmi-sent-ts")
                    .map_or(String::new(), std::string::ToString::to_string),
                turbo: tags.get("turbo").map_or(0, |x| x.parse::<i32>().unwrap_or(0)),
                user_id: tags
                    .get("user-id")
                    .map_or(String::new(), std::string::ToString::to_string),
                user_type: tags
                    .get("user-type")
                    .map_or(String::new(), std::string::ToString::to_string),
                vip: tags.get("vip").map_or(String::new(), std::string::ToString::to_string),
            }
        } else {
            Self::new()
        }
    }
}
