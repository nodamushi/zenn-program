#!/bin/bash -eu
set --
export SDKDIR=/spresense/sdk
source /spresense/sdk/tools/build-env.sh

cd /work
rm -f /work/dist/nuttx.spk /spresense/sdk/nuttx.spk
spr-make -j
mkdir -p /work/dist
cp /spresense/sdk/nuttx.spk /work/dist/nuttx.spk