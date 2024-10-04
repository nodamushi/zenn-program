#!/bin/bash -eu
#
# podman-compose up と(おおよそ)同じ処理
#
echo "[INFO] Create Pod"
podman pod create --name=pod_docker-compose2 --infra=false --share=

echo "[INFO] Create Net"
if ! podman network exists docker-compose2_testnet; then
  podman network create --driver bridge --subnet 192.168.11.0/24 docker-compose2_testnet
fi

if ! podman network exists docker-compose2_hostnet; then
  podman network create --driver bridge docker-compose2_hostnet
fi

echo "[INFO] Create Container"
podman create \
 --name=docker-compose2_node1_1 \
 --pod=pod_docker-compose2 \
 --network=docker-compose2_testnet --ip=192.168.11.100 \
 --network-alias=node1 \
 docker-compose2_node1 sh -c "sleep 1; ping -c 1 node3;  ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o LogLevel=ERROR root@node2 ifconfig; ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o LogLevel=ERROR root@node3 ifconfig"

podman create \
  --name=docker-compose2_node2_1 \
  --pod=pod_docker-compose2 \
  --network=docker-compose2_testnet:ip=192.168.11.101 \
  --network=docker-compose2_hostnet \
  --network-alias=node2 \
  -p 2222:22 \
  docker-compose2_node2

podman create \
  --name=docker-compose2_node3_1 \
  --pod=pod_docker-compose2 \
  --network=docker-compose2_testnet:ip=192.168.11.102 \
  --network=docker-compose2_hostnet \
  --network-alias=node3 \
  -p 2223:22 \
  docker-compose2_node3

echo "[INFO] Start Container"
podman start docker-compose2_node2_1
podman start docker-compose2_node3_1
podman start -a docker-compose2_node1_1
