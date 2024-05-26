use std::fs;

use super::error;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub debug: bool,
    pub channels: Vec<String>,
    pub nickname: String,
    pub oauth: String,
    pub postgres_db: String,
    pub postgres_host: String,
    pub postgres_password: String,
    pub postgres_user: String,
    pub server: String,
}

impl Config {
    pub fn load() -> Result<Self, error::Error> {
        let file = fs::OpenOptions::new().read(true).open("config.json")?;
        let json: Self = serde_json::from_reader(file)?;

        Ok(json)
    }
}
