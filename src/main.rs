#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;

mod lib;

use indicatif::ProgressIterator;
use lib::{config::Config, db, event::Event, msg::Msg, tags::Tag};
use std::sync::{Arc, Mutex};
use std::{thread, time};
use tungstenite::{connect, Message};

fn main() {
    db::create_tables().expect("Error creating postgres table");

    let v = Arc::new(Mutex::new(Vec::new()));
    let config = Config::load().expect("Error reading config file");
    let channel_count = config.channels.len();
    let (mut socket, _response) =
        connect(config.server).expect("Error connecting to websocket server");

    println!("Connected to websocket server...");
    println!("Joining channel(s) now...");

    socket
        .write_message(Message::Text(
            "CAP REQ :twitch.tv/membership twitch.tv/tags twitch.tv/commands".into(),
        ))
        .unwrap();
    socket.write_message(Message::Text(format!("PASS {}", config.oauth))).unwrap();
    socket.write_message(Message::Text(format!("NICK {}", config.nickname))).unwrap();

    // The rate limit to join channels is 50 every 15 seconds.
    for channel in config.channels.iter().progress() {
        socket.write_message(Message::Text(format!("JOIN {}", channel))).unwrap();
        thread::sleep(time::Duration::from_millis(320));
    }

    match channel_count {
        0 => println!("Bot is now logging 0 channels..."),
        1 => println!("Bot is now logging 1 channel..."),
        _ => println!("Bot is now logging {} channels...", channel_count),
    };

    loop {
        let data = socket.read_message().expect("Error reading websocket message");
        let data = data.into_text().expect("Error converting websocket message to string");
        let mut v = v.lock().expect("Error acquiring channel mutex");

        if data == "PING :tmi.twitch.tv" {
            socket.write_message(Message::Text("PONG :tmi.twitch.tv".into())).unwrap();
        }

        let tags = Tag::parse_tags(&data);
        let msg = Msg::parse_message(&data);
        let event = Event::new(msg, tags);

        println!("{:?}", event);

        if event.msg.content.len() > 1 {
            v.push(event);

            if v.len() >= 100 {
                db::insert_logs(v.to_owned());
                v.clear();
            }
        }
    }
    // socket.close(None);
}
