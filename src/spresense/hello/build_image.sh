#!/bin/bash -eu
dir=$(dirname "$0")
cd "$dir"
source ./spresense_info

cd container
if [ ! -f "$firmware" ]; then
  echo "[ERROR] $(pwd)/$firmware NOT found."
  if command -v jq > /dev/null 2>&1; then
    echo "[INFO] Download URL:"
    curl -s https://raw.githubusercontent.com/sonydevworld/spresense/refs/tags/v$version/firmware/spresense/version.json | jq -r '.DownloadURL'
  else
    echo "[INFO] See URL: https://github.com/sonydevworld/spresense/blob/v$version/firmware/spresense/version.json"
  fi
  exit 1
fi

podman build --build-arg VERSION=$version --build-arg FIRMWARE=$firmware -t sony-spresense:$version .

echo "[DONE] sony-spresense:$version"
