#!/bin/bash

cd $(dirname "$0")
cd ssh-client
podman build -t ubuntu-ssh-client . -f ContainerFile
