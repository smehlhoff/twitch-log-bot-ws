#![warn(clippy::all)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]

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
    buffer_size: usize,
) {
    let v = Arc::new(Mutex::new(Vec::new()));
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
        let mut v = v.lock().await;

        if data == "PING :tmi.twitch.tv\r\n" {
            socket.send(Message::Text("PONG :tmi.twitch.tv".into())).unwrap();
        }

        let tags = tags::Tag::parse_tags(&data);
        let msg = msg::Msg::parse_message(&data);
        let event = event::Event::new(msg, tags);

        if event.msg.content.len() > 1 {
            v.push(event);

            if v.len() >= buffer_size {
                let _ = db::insert_data(&pool, v.to_owned()).await;
                v.clear();
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
    let mut buffer_size = channel_count * 4;

    if thread_count == 1 {
        connect_and_listen(pool, channels, buffer_size).await;
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

        println!("Bot is now joining channels... This may take some time depending on the number of channels configured.\n");

        for i in 0..thread_count {
            let pool_clone = pool.clone();
            let thread_channel_list: Vec<String> =
                thread_channels[i].iter().map(std::borrow::ToOwned::to_owned).collect();

            buffer_size = thread_channel_list.len() * 4;

            println!("Joined: {thread_channel_list:?}");

            count += thread_channel_list.len();

            let thread = tokio::spawn(async move {
                connect_and_listen(pool_clone, thread_channel_list, buffer_size).await;
            });

            threads.push(thread);

            if count != channel_count {
                // Let previous thread join all channels before starting the next one
                thread::sleep(time::Duration::from_secs(25));
            }
        }

        match channel_count {
            0 => println!("\nBot is logging 0 channels..."),
            1 => println!("\nBot is logging 1 channel..."),
            _ => println!("\nBot is logging {channel_count} channels..."),
        };

        for thread in threads {
            thread.await.expect("Thread panicked");
        }
    }

    Ok(())
}
