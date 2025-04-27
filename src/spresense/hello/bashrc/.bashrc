# /bashrc/.bashrc に格納
# ~/.bashrc から読み込まれる

PS1='\[\e[1;$(($? == 0 ? 32 : 31))m\]\w >\[\e[0m\] '

# Boot-loader を書き込むショートカット
alias write-bootloader="/spresense/sdk/tools/flash.sh -l /spresense/firmware/spresense -c $TARGET_USB"

alias serial="screen $TARGET_USB 115200"

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


function init-work() {
  spr-create-approot /work
}

function write-app() {
  if [ -z "$1" ];then
    bin=/work/dist/nuttx.spk
  else
    bin=$1
  fi
  /spresense/sdk/tools/flash.sh -c $TARGET_USB $bin
}

function build() {
  spr-make
  mkdir -p /work/dist
  cp /spresense/sdk/nuttx.spk /work/dist/nuttx.spk
}
