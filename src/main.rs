#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;

mod lib;

use lib::{config::Config, db, event::Event, msg::Msg, tags::Tag};
use std::sync::{Arc, Mutex};
use std::{thread, time};
use tungstenite::{connect, Message};

fn connect_and_listen(shard_channels: Vec<String>) {
    let v = Arc::new(Mutex::new(Vec::new()));
    let config = Config::load().expect("Error reading config file");
    let (mut socket, _response) =
        connect(config.server).expect("Error connecting to websocket server");

    socket
        .write_message(Message::Text(
            "CAP REQ :twitch.tv/membership twitch.tv/tags twitch.tv/commands".into(),
        ))
        .unwrap();
    socket.write_message(Message::Text(format!("PASS {}", config.oauth))).unwrap();
    socket.write_message(Message::Text(format!("NICK {}", config.nickname))).unwrap();

    // The rate limit to join channels is 50 every 15 seconds.
    for channel in shard_channels {
        if !channel.contains("#") {
            socket.write_message(Message::Text(format!("JOIN #{}", channel))).unwrap();
        } else {
            socket.write_message(Message::Text(format!("JOIN {}", channel))).unwrap();
        }
        thread::sleep(time::Duration::from_millis(320));
    }

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

fn main() {
    db::create_tables().expect("Error creating postgres table");

    let config = Config::load().expect("Error reading config file");
    let channels = config.channels;
    let channel_count = channels.len();

    match channel_count {
        0 => println!("Bot is logging 0 channels..."),
        1 => println!("Bot is logging 1 channel..."),
        _ => println!("Bot is logging {} channels...", channel_count),
    };

    let thread_count = {
        if channel_count < 50 {
            1
        } else {
            let count = channel_count as f64 / 50 as f64;
            count.ceil() as usize
        }
    };

    if thread_count == 1 {
        connect_and_listen(channels);
    } else {
        let chunk_size = channel_count / thread_count;
        let thread_channels: Vec<Vec<_>> =
            channels.chunks(chunk_size).map(|c| c.to_vec()).collect();
        let mut threads = Vec::new();

        for i in 0..=thread_count {
            let thread_channel_list: Vec<String> =
                thread_channels[i].iter().map(|x| x.to_owned()).collect();
            let thread = thread::spawn(move || connect_and_listen(thread_channel_list));

            threads.push(thread);
        }

        for thread in threads {
            thread.join().expect("Thread panicked");
        }
    }
}
