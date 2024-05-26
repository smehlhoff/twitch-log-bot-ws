#![warn(clippy::all)]
// #![warn(clippy::nursery)]
// #![warn(clippy::pedantic)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use lib::{config, db, error, event, msg, tags};
use std::sync::Arc;
use std::{thread, time};
use tokio::sync::Mutex;
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

async fn connect_and_listen(
    pool: Pool<PostgresConnectionManager<NoTls>>,
    channels: Vec<String>,
    batch_size: usize,
) {
    let batch = Arc::new(Mutex::new(Vec::new()));
    let config = config::Config::load().expect("Error reading config file");
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

    loop {
        let data = socket.read().expect("Error reading websocket message");
        let data = data.into_text().expect("Error converting websocket message to string");
        let mut batch = batch.lock().await;

        if data == "PING :tmi.twitch.tv\r\n" {
            socket.send(Message::Text("PONG :tmi.twitch.tv".into())).unwrap();
        }

        let tags = tags::Tag::parse_tags(&data);
        let msg = msg::Msg::parse_message(&data);
        let event = event::Event::new(msg, tags);

        if event.msg.content.len() > 1 {
            batch.push(event);

            if batch.len() >= batch_size {
                let pool_clone = pool.clone();
                let batch_clone = batch.clone();
                tokio::spawn(async move {
                    match db::insert_data(pool_clone, batch_clone).await {
                        Ok(_) => {}
                        Err(e) => eprint!("{e}"),
                    };
                });
                batch.clear();
            }
        }
    }
    // socket.close(None);
}

#[tokio::main]
async fn main() -> Result<(), error::Error> {
    let config = config::Config::load()?;
    let pool = db::create_pool(&config).await?;
    let channels = config.channels;
    let channel_count = channels.len();

    db::create_table(&pool).await?;

    let thread_count = {
        if channel_count < 50 {
            1
        } else {
            let count = channel_count as f64 / 50.0;
            count.ceil() as usize
        }
    };
    let mut batch_size = channel_count * 4;

    match channel_count {
        0 => println!("Bot is now joining 0 channels...\n"),
        1 => println!("Bot is now joining 1 channel...\n"),
        _ => println!("Bot is now joining {channel_count} channels...\n"),
    };

    if thread_count == 1 {
        println!("Thread #0: {channels:?}");
        connect_and_listen(pool, channels, batch_size).await;
    } else {
        let chunk_size = {
            if channel_count % 2 == 0 {
                channel_count / thread_count
            } else {
                (channel_count + 1) / thread_count
            }
        };
        let thread_channels: Vec<Vec<String>> =
            channels.chunks(chunk_size).map(|x| x.to_vec()).collect();
        let mut threads = Vec::new();
        let mut count = 0;

        #[warn(clippy::needless_range_loop)]
        for i in 0..thread_count {
            let pool_clone = pool.clone();
            let thread_channel_list: Vec<String> =
                thread_channels[i].iter().map(std::borrow::ToOwned::to_owned).collect();

            batch_size = thread_channel_list.len() * 4;

            println!("Thread #{i}: {thread_channel_list:?}");

            count += thread_channel_list.len();

            let thread = tokio::spawn(async move {
                connect_and_listen(pool_clone, thread_channel_list, batch_size).await;
            });

            threads.push(thread);

            if count != channel_count {
                // Let previous thread join all channels before starting the next one
                thread::sleep(time::Duration::from_secs(25));
            }
        }

        for thread in threads {
            thread.await.expect("Thread panicked");
        }
    }

    Ok(())
}
