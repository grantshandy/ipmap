#!/bin/sh

pkexec setcap cap_net_raw,cap_net_admin=eip $1
exec $1
