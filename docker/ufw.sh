#!/bin/bash

set -e

PORTS=("5432" "9187" "9090" "3000" "3001" "2222" "5678" "80" "443")

allow_port() {
  local port=$1
  sudo ufw allow $port
}

sudo ufw enable

for port in "${PORTS[@]}"; do
  allow_port $port
done

sudo ufw reload

sudo ufw status
