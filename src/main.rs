#![warn(clippy::all)]
// #![warn(clippy::nursery)]
// #![warn(clippy::pedantic)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use env_logger::Env;
use lib::{config, db, error, event, msg, tags};
use log::{debug, error, info, warn};
use std::{cmp, collections::VecDeque, sync::Arc, time};
use tokio::sync::{mpsc, Mutex};
use tokio_postgres::NoTls;
use tungstenite::{connect, Message};

mod lib {
    pub mod config;
    pub mod db;
    pub mod error;
    pub mod event;
    pub mod msg;
    pub mod tags;
}

const MIN_BATCH_SIZE: usize = 10;
const MAX_BATCH_SIZE: usize = 50;

async fn connect_and_listen(
    pool: Pool<PostgresConnectionManager<NoTls>>,
    channels: Vec<String>,
    thread_id: u32,
) {
    let batch = Arc::new(Mutex::new(Vec::new()));
    let mut batch_size = 10;
    let mut buffer = VecDeque::new();
    let mut reconnect_count = 0;
    let mut reconnect_time = 30;
    let config = match config::Config::load() {
        Ok(data) => data,
        Err(e) => {
            error!("Thread #{thread_id}: {e}");
            return;
        }
    };
    let (tx, mut rx) = mpsc::channel(10);
    let pool_clone = pool.clone();

    tokio::spawn(async move {
        while let Some(batch_ready) = rx.recv().await {
            let pool_clone = pool_clone.clone();
            tokio::spawn(async move {
                match db::insert_data(pool_clone, batch_ready).await {
                    Ok(()) => {}
                    Err(e) => warn!("{e}"),
                }
            });
        }
    });

    loop {
        match connect(&config.server) {
            Ok((mut socket, _response)) => {
                info!("Thread #{thread_id}: Connected to websocket server successfully");

                // Reset counters
                reconnect_count = 0;
                reconnect_time = 30;

                socket
                    .send(Message::Text(
                        "CAP REQ :twitch.tv/membership twitch.tv/tags twitch.tv/commands".into(),
                    ))
                    .unwrap();
                socket.send(Message::Text(format!("PASS {}", config.oauth))).unwrap();
                socket.send(Message::Text(format!("NICK {}", config.nickname))).unwrap();

                info!("Thread #{thread_id}: {channels:?}");

                // The rate limit to join channels is 20 per 10 seconds per account
                // https://dev.twitch.tv/docs/irc/#rate-limits
                for channel in &channels {
                    match socket.send(Message::Text(format!("JOIN {channel}"))) {
                        Ok(()) => debug!("Thread #{thread_id}: Joined {channel} successfully"),
                        Err(e) => {
                            warn!("Thread #{thread_id}: Error joining {channel}: {e}");
                        }
                    }
                    tokio::time::sleep(time::Duration::from_secs_f32(0.6)).await;
                }

                loop {
                    let buffer_count = buffer.len();

                    debug!("Thread #{thread_id}: Batch Size: {batch_size} Buffer Count: {buffer_count}");

                    if let Ok(data) = socket.read() {
                        let data = data.into_text().unwrap();

                        if data == "PING :tmi.twitch.tv\r\n" {
                            socket.send(Message::Text("PONG :tmi.twitch.tv".into())).unwrap();
                        }

                        let msg = msg::Msg::parse_message(&data);
                        let tags = tags::Tag::parse_tags(&data);
                        let event = event::Event::new(msg, tags);

                        if !event.msg.content.is_empty() {
                            let mut batch = batch.lock().await;

                            batch.push(event);

                            if batch.len() >= batch_size {
                                let batch_ready = batch.split_off(0);

                                drop(batch);

                                if tx.try_send(batch_ready.clone()).is_err() {
                                    buffer.push_back(batch_ready);
                                    batch_size = cmp::min(batch_size + 10, MAX_BATCH_SIZE);
                                } else {
                                    batch_size = cmp::max(batch_size - 10, MIN_BATCH_SIZE);
                                }
                            }
                        }

                        while let Some(batch_ready) = buffer.pop_front() {
                            if tx.try_send(batch_ready.clone()).is_err() {
                                buffer.push_front(batch_ready);
                                break;
                            }
                        }
                    } else {
                        warn!("Thread #{thread_id}: Disconnected from websocket server");
                        break;
                    }
                }
            }
            Err(e) => {
                warn!("Thread #{thread_id}: Error connecting to websocket server: {e}. Retrying in {reconnect_time} seconds...");

                tokio::time::sleep(time::Duration::from_secs(reconnect_time)).await;

                reconnect_count += 1;
                reconnect_time += 30;

                if reconnect_count == 3 {
                    warn!("Thread #{thread_id}: Failed to connect to websocket server in {reconnect_count} attempts");
                    break;
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), error::Error> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let config = config::Config::load()?;
    let pool = db::create_pool(&config).await?;

    db::create_table(&pool).await?;

    let channels: Vec<String> = config
        .channels
        .iter()
        .map(|x| {
            let mut channel = x.to_lowercase();
            if !channel.starts_with('#') {
                channel.insert(0, '#');
            }
            channel
        })
        .collect();
    let channel_count = channels.len();
    let mut thread_id = 0;
    let thread_count = {
        if channel_count < 50 {
            1
        } else {
            let count = channel_count as f64 / 50.0;
            count.ceil() as usize
        }
    };

    match channel_count {
        0 => info!("Bot is now joining 0 channels...\n"),
        1 => info!("Bot is now joining 1 channel...\n"),
        _ => info!("Bot is now joining {channel_count} channels...\n"),
    };

    if thread_count == 1 {
        connect_and_listen(pool, channels, thread_id).await;
    } else {
        let chunk_size = {
            if channel_count % 2 == 0 {
                channel_count / thread_count
            } else {
                (channel_count + 1) / thread_count
            }
        };
        let thread_channels: Vec<Vec<String>> =
            channels.chunks(chunk_size).map(std::borrow::ToOwned::to_owned).collect();
        let mut threads = Vec::new();

        #[allow(clippy::needless_range_loop)]
        for i in 0..thread_count {
            let pool_clone = pool.clone();
            let thread_channel_list: Vec<String> =
                thread_channels[i].iter().map(std::borrow::ToOwned::to_owned).collect();

            thread_id = u32::try_from(i).unwrap_or(0);

            let thread = tokio::spawn(async move {
                connect_and_listen(pool_clone, thread_channel_list, thread_id).await;
            });

            threads.push(thread);

            // No need to sleep on last thread
            if i != thread_count - 1 {
                // Let previous thread join all channels before starting the next one
                tokio::time::sleep(time::Duration::from_secs(30)).await;
            }
        }

        for thread in threads {
            match thread.await {
                Ok(()) => {}
                Err(e) => {
                    warn!("{e}");
                }
            }
        }
    }

    Ok(())
}
