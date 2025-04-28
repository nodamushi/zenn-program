# ~/.spresense_env
set --
export SDKDIR=/spresense/sdk
source /spresense/sdk/tools/build-env.sh

if [ -f /bash/.bash_profile ];then
  source /bash/.bash_profile
fi

if [ -f /work/.bash_profile ];then
  source /work/.bash_profile
fi
