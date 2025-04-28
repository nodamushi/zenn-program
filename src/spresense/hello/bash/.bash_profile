# /bash/.bash_profile に格納
# ~/.bash_profile から読み込まれる

# 糞(~/.spresense_env) を排出
if [ ! -f ~/.spresense_env ];then
  cat <<EOF >~/.spresense_env
SPRESENSE_HOME=/work
SPRESENSE_SDK=/spresense
EOF
fi

set --
export SDKDIR=/spresense/sdk
source /spresense/sdk/tools/build-env.sh

if [ -f /work/.bash_profile ];then
  source /work/.bash_profile
fi
