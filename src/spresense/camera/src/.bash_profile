# .bash_profile から読み込まれます
function init-work() {
  spr-create-approot /work
}

# build手順
function build() {
  bash /work/build.sh
}

function rebuild() {
  bash /work/rebuild.sh
}

function clean() {
  spr-make distclean
}

# Boot-loader を書き込むショートカット
function write-bootloader() {
  /spresense/sdk/tools/flash.sh -l /spresense/firmware/spresense -b 500000 -c $TARGET_USB
}

# アプリを書き込むショートカット
function write-app() {
  if [ -z "$1" ];then
    bin=/work/dist/nuttx.spk
  else
    bin=$1
  fi
  /spresense/sdk/tools/flash.sh -c $TARGET_USB -w /work/init.rc
  /spresense/sdk/tools/flash.sh -c $TARGET_USB -b 500000 $bin
}

