#!/bin/bash -eu
#
# 立ち上げた node2 or node3 に ssh で接続します。
#  node2: ./connect.sh 2
#  node3: ./connect.sh 3
#
port=2222
if [ $# -eq 1 ];then
  if [ "$1" -eq 2 ];then
    port=2222
  elif [ "$1" -eq 3 ]; then
    port=2223
  fi
fi

dir=$(dirname "$0")
cd "$dir"

# Copy id_rsa (mode: r--)
if [ ! -f ./id_rsa ]; then
  cp ../client/id_rsa .
  chmod 600 ./id_rsa
fi

ssh \
  -p $port \
  -i ./id_rsa \
  -o StrictHostKeyChecking=no \
  -o UserKnownHostsFile=/dev/null \
  -o LogLevel=ERROR \
  root@localhost
