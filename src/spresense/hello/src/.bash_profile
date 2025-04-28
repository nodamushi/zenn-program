# .bash_profile から読み込まれます

function init-work() {
  spr-create-approot /work
}

# build手順
function build() {
  spr-config default
  spr-make
  mkdir -p /work/dist
  cp /spresense/sdk/nuttx.spk /work/dist/nuttx.spk
}

# Boot-loader を書き込むショートカット
alias write-bootloader="/spresense/sdk/tools/flash.sh -l /spresense/firmware/spresense -c $TARGET_USB"

# アプリを書き込むショートカット
function write-app() {
  if [ -z "$1" ];then
    bin=/work/dist/nuttx.spk
  else
    bin=$1
  fi
  /spresense/sdk/tools/flash.sh -c $TARGET_USB $bin
}
