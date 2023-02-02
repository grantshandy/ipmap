#!/bin/bash

set -e

echo "Running custom setcap.sh script."

if [[ ! "$TAURI_PLATFORM" ]]; then
  cargo $1
fi

sudo setcap cap_net_raw,cap_net_admin=eip target/debug/ipmap
sudo setcap cap_net_raw,cap_net_admin=eip target/release/ipmap
