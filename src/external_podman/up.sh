#!/bin/bash -eu

dst=192.168.1.3
from=192.168.100.3

subnet=192.168.21.0/24
container=192.168.21.5

sudo iptables -t nat -A OUTPUT -p tcp -d $from -j DNAT --to-destination $dst
sudo iptables -t nat -A OUTPUT -p udp -d $from -j DNAT --to-destination $dst

podman network create testnet --subnet $subnet --internal
podman run --rm --name client -it --net testnet:ip=$container --net podman ubuntu-ssh-client bash
