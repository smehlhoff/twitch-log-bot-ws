# twitch-log-bot-ws

This bot logs twitch.tv channel messages via websockets.

By default, channel messages are stored in a `postgres` database.

## Installation

Rename `config-example.json` to `config.json` and edit fields.

    $ sudo apt update -y
    $ sudo apt install build-essential libssl-dev pkg-config
    $ curl https://sh.rustup.rs -sSf | sh
    $ source $HOME/.cargo/env
    $ git clone https://github.com/smehlhoff/twitch-log-bot-ws
    $ cd twitch-log-bot-ws
    $ cargo build --release
    $ nohup ./target/release/twitch-log-bot-ws &

## Docker

Use docker compose to run `dev` or `prod` environments.

    $ docker compose -f dev.yml up --detach --build
    $ docker compose -f prod.yml up --detach --build

For prod, read `/docker/deploy.md` file, as additional steps are required.

## Contributing

Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

## License

[MIT](https://github.com/smehlhoff/twitch-log-bot-ws/blob/master/LICENSE)
