#!/bin/bash
dir=$(dirname "$0")
cd "$dir"
dir=$(pwd)
source ./spresense_info
img=sony-spresense:$version
srcmnt=/work
################################
# Arguments
################################
daemon_mode=false
devusb=""
build_only=false
deploy=false
deploy_app=false

while [[ $# -gt 0 ]]; do
  case $1 in
    -d)
      daemon_mode=true
      shift
      ;;
    -u|--usb)
      if [[ -n "$2" ]]; then
        devusb="/dev/$2"
        shift 2
      else
        echo "[ERROR] USB device name not found"
        exit 1
      fi
      ;;
    -b|--build-only)
      build_only=true
      shift
      ;;
    --deploy)
      deploy=true
      build_only=false
      shift
      ;;
    --deploy-app)
      deploy_app=true
      build_only=false
      shift
      ;;
    *)
      echo "[ERROR] Unknown option $1"
      echo "usage: $0 [-d] [-u|--usb USB-Device-Name] [-b|--build-only]"
      exit 1
      ;;
  esac
done

# USB device detection and verification (only if not build_only)
if [ "$build_only" = false ]; then
  if [ -z "$devusb" ]; then
    usb=$(dmesg | grep "cp21.*attached" | grep -o "ttyUSB[0-9]*" | tail -n 1)
    if [ -z "$usb" ]; then
      echo "[ERROR] USB not found!"
      exit 1
    fi
    devusb="/dev/$usb"
  fi

  # read/write check of /dev/ttyUSBx
  owner=$(stat -c '%U' "$devusb")
  if [ "$owner" != "$USER" ]; then
    echo "[WARN] Current owner of $devusb is '$owner'"
    echo "[INFO] Attempting to change owner to $USER with sudo..."
    sudo chown "$USER:$USER" "$devusb"

    new_owner=$(stat -c '%U' "$devusb")
    if [ "$new_owner" != "$USER" ]; then
      echo "[ERROR] Failed to change owner of $devusb to $USER"
      exit 1
    else
      echo "[INFO] Successfully changed owner of $devusb to $USER"
    fi
  fi

  echo "[INFO] USB Device: $devusb"

  # USB device related options
  usb_options="--device=${devusb} -e TARGET_USB=$devusb "
else
  daemon_mode=false
  echo "[INFO] Build-only mode (no USB device required)"
  usb_options=""
fi

bash=
if [ -d "$dir/bash" ];then
  bash="--mount=type=bind,src=$dir/bash,dst=/bash,ro=true"
fi

# Container execution
if [ "$daemon_mode" = true ]; then
  container_name="dev-spresense"
  echo "[INFO] run '$container_name' daemon container"

  podman run -d \
   -name=dev-spresense \
    $bash \
   "--mount=type=bind,src=$dir/src,dst=$srcmnt" \
   $usb_options \
   -w $srcmnt \
   $img \
   tail -f /dev/null
elif [ $deploy_app = true ]; then

  container_name="dev-spresense"
  echo "[INFO] run '$container_name' daemon container"

  podman run --rm \
    $bash \
   "--mount=type=bind,src=$dir/src,dst=$srcmnt" \
   $usb_options \
   -w $srcmnt \
   $img \
      bash --login -c "build && write-app"

elif [ $deploy = true ]; then
  container_name="dev-spresense"
  echo "[INFO] run '$container_name' daemon container"

  podman run --rm \
    $bash \
   "--mount=type=bind,src=$dir/src,dst=$srcmnt" \
   $usb_options \
   -w $srcmnt \
   $img \
      bash --login -c "build && write-bootloader && write-app"

elif [ $build_only = true ]; then

  podman run --rm \
   $bash \
   "--mount=type=bind,src=$dir/src,dst=$srcmnt" \
   $usb_options \
   -w $srcmnt \
   $img \
   bash --login -c build

else
  podman  run --rm -it \
   $bash \
   "--mount=type=bind,src=$dir/src,dst=$srcmnt" \
   $usb_options \
   -w $srcmnt \
   $img \
   bash
fi

