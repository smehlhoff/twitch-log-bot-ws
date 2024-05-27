use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use log::{error, info, warn};
use tokio::time::Duration;
use tokio_postgres::NoTls;

use super::{config, error, event};

pub async fn create_pool(
    secrets: &config::Config,
) -> Result<Pool<PostgresConnectionManager<NoTls>>, error::Error> {
    let mut config = tokio_postgres::Config::new();

    config
        .host(&secrets.postgres_host)
        .user(&secrets.postgres_user)
        .password(&secrets.postgres_password)
        .dbname(&secrets.postgres_db);

    let manager = PostgresConnectionManager::new(config, NoTls);

    match Pool::builder()
        .max_size(15)
        .connection_timeout(Duration::from_secs(30))
        .build(manager)
        .await
    {
        Ok(pool) => {
            info!("Postgres connection pool created successfully");
            Ok(pool)
        }
        Err(e) => {
            error!("Error creating Postgres connection pool: {e}");
            Err(error::Error::Postgres(e))
        }
    }
}

pub async fn create_table(
    pool: &Pool<PostgresConnectionManager<NoTls>>,
) -> Result<(), error::Error> {
    match pool.get().await {
        Ok(conn) => {
            match conn
                .execute(
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
                )
                .await
            {
                Ok(_) => {
                    info!("Postgres table created successfully or already exists.");
                    Ok(())
                }
                Err(e) => {
                    error!("Error creating Postgres table: {e}");
                    Err(error::Error::Postgres(e))
                }
            }
        }
        Err(e) => {
            error!("Error retrieving connection from pool: {e}");
            Err(error::Error::bb8(e))
        }
    }
}

pub async fn insert_data(
    pool: Pool<PostgresConnectionManager<NoTls>>,
    events: Vec<event::Event>,
) -> Result<(), error::Error> {
    match pool.get().await {
        Ok(mut conn) => {
            let transaction = match conn.transaction().await {
                Ok(transaction) => {
                    info!("Retrieved Postgres transaction successfully");
                    transaction
                }
                Err(e) => {
                    warn!("Error retrieving Postgres transaction: {e}");
                    return Err(error::Error::Postgres(e));
                }
            };
            let statement = match transaction.prepare("INSERT INTO logs (
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
            );").await {
                Ok(statement) => {
                    info!("Postgres statement prepared successfully");
                    statement
                },
                Err(e) => {
                    warn!("Error preparing Postgres statement: {e}");
                    return Err(error::Error::Postgres(e));
                }
            };

            for event in events {
                let result = tokio::time::timeout(
                    Duration::from_secs(10),
                    transaction.execute(
                        &statement,
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
                    ),
                )
                .await;

                match result {
                    Ok(Ok(_)) => info!("Data inserted successfully"),
                    Ok(Err(e)) => {
                        warn!("Error inserting data: {e}");
                        return Err(error::Error::Postgres(e));
                    }
                    Err(e) => {
                        warn!("Timeout occurred while inserting data: {e}");
                    }
                };
            }

            match transaction.commit().await {
                Ok(()) => {
                    info!("Postgres transaction committed successfully");
                    Ok(())
                }
                Err(e) => {
                    error!("Error committing Postgres transaction: {e}");
                    Err(error::Error::Postgres(e))
                }
            }
        }
        Err(e) => {
            error!("Error retrieving connection from pool: {e}");
            Err(error::Error::bb8(e))
        }
    }
}
