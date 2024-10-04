#!/bin/bash
#
# podman-compose down と(おおよそ)同じ処理
#

podman stop -t 0 docker-compose_node2_1
podman stop -t 0 docker-compose_node1_1

podman rm docker-compose_node2_1
podman rm docker-compose_node1_1
podman pod rm pod_docker-compose
