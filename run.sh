#!/bin/bash

trap 'kill $BGPID; exit' INT
cargo run &
BGPID=$! # Kill cargo subprocess on ctrl+c
sleep 1
echo "Performing manual subscriptions..."
python3 python/manual_subscriptions.py

# Keep alive until ctrl+c
read -r -d '' _ </dev/tty
