#!/bin/sh
set -e
 
sudo -S setcap cap_net_raw,cap_net_admin=eip $1
$1
