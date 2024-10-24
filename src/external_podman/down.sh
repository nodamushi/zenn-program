#!/bin/bash

dst=192.168.1.3
from=192.168.100.3

podman kill client
podman network rm testnet
sudo iptables -t nat -D OUTPUT -p tcp -d $from -j DNAT --to-destination $dst
sudo iptables -t nat -D OUTPUT -p udp -d $from -j DNAT --to-destination $dst
