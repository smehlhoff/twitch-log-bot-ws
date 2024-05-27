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
use log::{info, warn};
use std::{cmp, collections::VecDeque, sync::Arc, thread, time};
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
const MAX_BATCH_SIZE: usize = 100;

async fn connect_and_listen(
    pool: Pool<PostgresConnectionManager<NoTls>>,
    channels: Vec<String>,
    thread_id: u32,
) {
    let batch = Arc::new(Mutex::new(Vec::new()));
    let mut batch_size = 10;
    let mut buffer = VecDeque::new();
    let config = config::Config::load().expect("Error reading config file");
    let (tx, mut rx) = mpsc::channel(10);
    let (mut socket, _response) =
        connect(config.server).expect("Error connecting to websocket server");

    socket
        .send(Message::Text(
            "CAP REQ :twitch.tv/membership twitch.tv/tags twitch.tv/commands".into(),
        ))
        .unwrap();
    socket.send(Message::Text(format!("PASS {}", config.oauth))).unwrap();
    socket.send(Message::Text(format!("NICK {}", config.nickname))).unwrap();

    // The rate limit to join channels is 20 per 10 seconds per account
    // https://dev.twitch.tv/docs/irc/#rate-limits
    for channel in channels {
        if channel.contains('#') {
            socket.send(Message::Text(format!("JOIN {channel}"))).unwrap();
        } else {
            socket.send(Message::Text(format!("JOIN #{channel}"))).unwrap();
        }
        thread::sleep(time::Duration::from_secs_f32(0.6));
    }

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
        let buffer_count = buffer.len();

        info!("Thread #{thread_id} - Batch Size: {batch_size} Buffer Count: {buffer_count}");

        let data = socket.read().expect("Error reading websocket message");
        let data = data.into_text().expect("Error converting websocket message to string");
        let mut batch = batch.lock().await;

        if data == "PING :tmi.twitch.tv\r\n" {
            socket.send(Message::Text("PONG :tmi.twitch.tv".into())).unwrap();
        }

        let tags = tags::Tag::parse_tags(&data);
        let msg = msg::Msg::parse_message(&data);
        let event = event::Event::new(msg, tags);

        if !event.msg.content.is_empty() {
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
    }
    // socket.close(None);
}

#[tokio::main]
async fn main() -> Result<(), error::Error> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let config = config::Config::load()?;
    let pool = db::create_pool(&config).await?;
    let channels = config.channels;
    let channel_count = channels.len();
    let mut thread_id = 0;

    db::create_table(&pool).await?;

    let thread_count = {
        if channel_count < 50 {
            1
        } else {
            let count = channel_count as f64 / 50.0;
            count.ceil() as usize
        }
    };

    match channel_count {
        0 => println!("Bot is now joining 0 channels...\n"),
        1 => println!("Bot is now joining 1 channel...\n"),
        _ => println!("Bot is now joining {channel_count} channels...\n"),
    };

    if thread_count == 1 {
        println!("Thread #{thread_id}: {channels:?}");

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
        let mut count = 0;

        #[allow(clippy::needless_range_loop)]
        for i in 0..thread_count {
            let pool_clone = pool.clone();
            let thread_channel_list: Vec<String> =
                thread_channels[i].iter().map(std::borrow::ToOwned::to_owned).collect();

            thread_id = u32::try_from(i).unwrap_or(0);

            println!("Thread #{thread_id}: {thread_channel_list:?}\n");

            count += thread_channel_list.len();

            let thread = tokio::spawn(async move {
                connect_and_listen(pool_clone, thread_channel_list, thread_id).await;
            });

            threads.push(thread);

            if count != channel_count {
                // Let previous thread join all channels before starting the next one
                thread::sleep(time::Duration::from_secs(25));
            }
        }

        for thread in threads {
            match thread.await {
                Ok(()) => {}
                Err(e) => {
                    warn!("Thread panicked: {e}");
                }
            }
        }
    }

    Ok(())
}
