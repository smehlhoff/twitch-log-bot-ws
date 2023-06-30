use crate::lib::{msg::Msg, tags::Tag};

#[derive(Debug, Clone)]
pub struct Event {
    pub msg: Msg,
    pub tags: Tag,
}

impl Event {
    pub const fn new(msg: Msg, tags: Tag) -> Self {
        Self { msg, tags }
    }
}
