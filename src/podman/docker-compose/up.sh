#!/bin/bash -eu
#
# podman-compose up と(おおよそ)同じ処理
#
echo "[INFO] Create Pod"
podman pod create --name=pod_docker-compose --infra=false --share=

echo "[INFO] Create Net"
if ! podman network exists docker-compose_testnet; then
  podman network create --driver bridge --subnet 192.168.10.0/24 docker-compose_testnet
fi

echo "[INFO] Create Container"
podman create \
  --name=docker-compose_node1_1 \
  --pod=pod_docker-compose \
  --network=docker-compose_testnet \
  --ip=192.168.10.100 \
  --network-alias=node1 \
  docker-compose_node1 sh -c "sleep 1; ping -c 1 node2;  ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o LogLevel=ERROR root@node2 ls .ssh"

podman create \
  --name=docker-compose_node2_1 \
  --pod=pod_docker-compose \
  --network=docker-compose_testnet \
  --ip=192.168.10.101 \
  --network-alias=node2 \
  docker-compose_node2

echo "[INFO] Start Container"
podman start docker-compose_node2_1
podman start -a docker-compose_node1_1
