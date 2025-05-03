#!/bin/bash
start=$1
set -e
set --
export SDKDIR=/spresense/sdk
source /spresense/sdk/tools/build-env.sh

cd /work
rm -f /work/dist/nuttx.spk /spresense/sdk/nuttx.spk

s=""
if [ "$start" = yes ];then
  s=feature/startup_script
fi

spr-config default feature/libcxx device/camera feature/usbcdcacm feature/smp +SMP_NCPUS=2 $s
spr-make -j
mkdir -p /work/dist
cp /spresense/sdk/nuttx.spk /work/dist/nuttx.spk
