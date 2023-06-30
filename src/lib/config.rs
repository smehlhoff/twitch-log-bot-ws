use std::fs;

use crate::lib::error;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub nickname: String,
    pub oauth: String,
    pub server: String,
    pub postgres: String,
    pub admins: Vec<String>,
    pub channels: Vec<String>,
}

impl Config {
    pub fn load() -> Result<Self, error::Error> {
        let file = fs::OpenOptions::new().read(true).open("config.json")?;
        let json: Self = serde_json::from_reader(file)?;

        Ok(json)
    }
}
