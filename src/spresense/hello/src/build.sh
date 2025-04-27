#!/bin/bash -eu

dir="$(dirname $0)"
cd "$dir"

set --
export SDKDIR=/spresense/sdk
source /spresense/sdk/tools/build-env.sh

spr-config default
spr-make -j
mkdir -p /work/dist
cp /spresense/sdk/nuttx.spk /work/dist/nuttx.spk
