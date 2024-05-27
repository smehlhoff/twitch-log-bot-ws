use std::fs;

use log::{info, warn};

use super::error;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
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
        let file = match fs::OpenOptions::new().read(true).open("config.json") {
            Ok(file) => {
                info!("File opened successfully");
                file
            }
            Err(e) => {
                warn!("Error opening file: {e}");
                return Err(error::Error::Io(e));
            }
        };
        let json: Self = match serde_json::from_reader(file) {
            Ok(json) => {
                info!("JSON file parsed successfully");
                json
            }
            Err(e) => {
                warn!("Error parsing JSON file: {e}");
                return Err(error::Error::Json(e));
            }
        };

        Ok(json)
    }
}
