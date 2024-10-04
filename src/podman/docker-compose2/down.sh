#!/bin/bash
#
# podman-compose down と(おおよそ)同じ処理
#

podman stop -t 0 docker-compose2_node3_1
podman stop -t 0 docker-compose2_node2_1
podman stop -t 0 docker-compose2_node1_1

podman rm docker-compose2_node3_1
podman rm docker-compose2_node2_1
podman rm docker-compose2_node1_1

podman pod rm pod_docker-compose2
