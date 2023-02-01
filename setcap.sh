#!/bin/sh
set -e

sudo setcap cap_net_raw,cap_net_admin=eip $1
$1
