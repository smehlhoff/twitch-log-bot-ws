use crate::lib::event;

use postgres::{Client, NoTls};
use std::thread;

use crate::lib::{config, error};

fn connect() -> Result<postgres::Client, error::Error> {
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
            badge_info VARCHAR,
            badges VARCHAR,
            bits VARCHAR,
            client_nonce VARCHAR,
            color VARCHAR,
            display_name VARCHAR,
            emote_only VARCHAR,
            emotes VARCHAR,
            first_msg INTEGER,
            flags VARCHAR,
            is_mod INTEGER,
            reply_parent_display_name VARCHAR,
            reply_parent_msg_body VARCHAR,
            reply_parent_msg_id VARCHAR,
            reply_parent_user_id VARCHAR,
            reply_parent_user_login VARCHAR,
            returning_chatter INTEGER,
            room_id VARCHAR,
            subscriber INTEGER,
            tags_raw VARCHAR,
            tmi_sent_ts VARCHAR,
            turbo INTEGER,
            user_id VARCHAR,
            user_type VARCHAR,
            vip VARCHAR,
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
            transaction.execute(
                "INSERT INTO logs (
                    username,
                    command,
                    channel,
                    content,
                    badge_info,
                    badges,
                    bits,
                    client_nonce,
                    color,
                    display_name,
                    emote_only,
                    emotes,
                    first_msg,
                    flags,
                    is_mod,
                    reply_parent_display_name,
                    reply_parent_msg_body,
                    reply_parent_msg_id,
                    reply_parent_user_id,
                    reply_parent_user_login,
                    returning_chatter,
                    room_id,
                    subscriber,
                    tags_raw,
                    tmi_sent_ts,
                    turbo,
                    user_id,
                    user_type,
                    vip,
                    timestamp
                  ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30
                );",
                &[
                    &event.msg.username,
                    &event.msg.command,
                    &event.msg.channel,
                    &event.msg.content,
                    &event.tags.badge_info,
                    &event.tags.badges,
                    &event.tags.bits,
                    &event.tags.client_nonce,
                    &event.tags.color,
                    &event.tags.display_name,
                    &event.tags.emote_only,
                    &event.tags.emotes,
                    &event.tags.first_msg,
                    &event.tags.flags,
                    &event.tags.is_mod,
                    &event.tags.reply_parent_display_name,
                    &event.tags.reply_parent_msg_body,
                    &event.tags.reply_parent_msg_id,
                    &event.tags.reply_parent_user_id,
                    &event.tags.reply_parent_user_login,
                    &event.tags.returning_chatter,
                    &event.tags.room_id,
                    &event.tags.subscriber,
                    &event.tags.tags_raw,
                    &event.tags.tmi_sent_ts,
                    &event.tags.turbo,
                    &event.tags.user_id,
                    &event.tags.user_type,
                    &event.tags.vip,
                    &event.msg.timestamp,
                ],
            )?;
        }

        transaction.commit()?;
        db.close()?;

        Ok(())
    });
}
