#!/bin/bash

set -e

function print_message() {
  echo "==============================================================="
  echo "$1"
  echo "==============================================================="
}

function install_rust() {
  print_message "Installing Rust..."

  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

  # Ensure rustc and cargo are available within the script
  source $HOME/.cargo/env

  # Add to shell profile for future sessions
  echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> $HOME/.bashrc

  # Export variables for subshells
  export PATH="$HOME/.cargo/bin:$PATH"

  print_message "Rust installed..."

  rustc --version
  cargo --version
}

function build_project() {
  git clone https://github.com/smehlhoff/twitch-log-bot-ws

  cd twitch-log-bot-ws

  cargo build --release
}

function main() {
  install_rust
  build_project

  print_message "Run 'source $HOME/.cargo/env' to update current shell."
}

main
