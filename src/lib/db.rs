use crate::lib::event;

use postgres::{Client, NoTls};
use std::thread;

use crate::lib::{config, error};

pub fn connect() -> Result<postgres::Client, error::Error> {
    let config = config::Config::load()?;
    let db = Client::connect(&config.postgres, NoTls)?;

    Ok(db)
}

pub fn create_tables() -> Result<(), error::Error> {
    let mut db = connect()?;

    db.execute(
        "CREATE TABLE IF NOT EXISTS logs (
            id SERIAL PRIMARY KEY,
            username VARCHAR,
            command VARCHAR,
            channel VARCHAR,
            content VARCHAR,
            tags VARCHAR,
            timestamp TIMESTAMP WITH TIME ZONE
        );",
        &[],
    )?;

    Ok(())
}

pub fn insert_logs(events: Vec<event::Event>) {
    thread::spawn(move || -> Result<(), error::Error> {
        let mut db = connect()?;
        let mut transaction = db.transaction()?;

        for event in events {
            transaction.execute("INSERT INTO logs (username, command, channel, content, tags, timestamp) VALUES ($1, $2, $3, $4, $5, $6)", &[&event.msg.username, &event.msg.command, &event.msg.channel, &event.msg.content, &event.tags.tags_raw, &event.msg.timestamp])?;
        }

        transaction.commit()?;
        db.close()?;

        Ok(())
    });
}
