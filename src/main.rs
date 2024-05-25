#![warn(clippy::all)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;

mod lib {
    pub mod config;
    pub mod db;
    pub mod error;
    pub mod event;
    pub mod msg;
    pub mod tags;
}

use lib::{config::Config, db, event::Event, msg::Msg, tags::Tag};
use std::sync::{Arc, Mutex};
use std::{thread, time};
use tungstenite::{connect, Message};

fn connect_and_listen(channels: Vec<String>, buffer_size: usize) {
    let v = Arc::new(Mutex::new(Vec::new()));
    let config = Config::load().expect("Error reading config file");
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
        let mut v = v.lock().expect("Error acquiring channel mutex");

        if data == "PING :tmi.twitch.tv" {
            socket.send(Message::Text("PONG :tmi.twitch.tv".into())).unwrap();
        }

        let tags = Tag::parse_tags(&data);
        let msg = Msg::parse_message(&data);
        let event = Event::new(msg, tags);

        if event.msg.content.len() > 1 {
            v.push(event);

            if v.len() >= buffer_size {
                db::insert_logs(v.to_owned());
                v.clear();
            }
        }
    }
    // socket.close(None);
}

fn main() {
    db::create_tables().expect("Error creating postgres table");

    let config = Config::load().expect("Error reading config file");
    let channels = config.channels;
    let channel_count = channels.len();

    match channel_count {
        0 => println!("Bot is logging 0 channels..."),
        1 => println!("Bot is logging 1 channel..."),
        _ => println!("Bot is logging {channel_count} channels..."),
    };

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
        connect_and_listen(channels, buffer_size);
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

        for i in 0..thread_count {
            let thread_channel_list: Vec<String> =
                thread_channels[i].iter().map(std::borrow::ToOwned::to_owned).collect();

            buffer_size = thread_channel_list.len() * 4;

            let thread =
                thread::spawn(move || connect_and_listen(thread_channel_list, buffer_size));

            threads.push(thread);
        }

        for thread in threads {
            thread.join().expect("Thread panicked");
            // Let thread join all channels before starting next one
            thread::sleep(time::Duration::from_secs(30));
        }
    }
}
