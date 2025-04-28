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
    usb=$(dmesg | grep "cp21.*attached" | grep -o "ttyUSB[0-9]*")
    if [ -z "$usb" ]; then
      echo "[ERROR] USB not found!"
      exit 1
    fi
    devusb="/dev/$usb"
  fi

  if [ ! -c "$devusb" ]; then
    echo "[ERROR] $devusb not found or not a character device"
    exit 1
  fi
  # read/write check of /dev/ttyUSBx
  if [ ! -r "$devusb" ] || [ ! -w "$devusb" ]; then
    echo "[WARN] No read/write permission on $devusb"
    echo "[INFO] Attempting to set permissions with sudo..."
    sudo chown $USER:$USER "$devusb"

    if [ ! -r "$devusb" ] || [ ! -w "$devusb" ]; then
      echo "[ERROR] Failed to set permissions on $devusb"
      exit 1
    else
      echo "[INFO] Successfully set permissions on $devusb"
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

# Container execution
if [ "$daemon_mode" = true ]; then
  container_name="dev-spresense"
  echo "[INFO] run '$container_name' daemon container"

  podman run -d \
   -name=dev-spresense \
   "--mount=type=bind,src=$dir/bash,dst=/bash,ro=true" \
   "--mount=type=bind,src=$dir/src,dst=$srcmnt" \
   $usb_options \
   -w $srcmnt \
   $img \
   tail -f /dev/null
elif [ $build_only = true ]; then

  podman run --rm \
   "--mount=type=bind,src=$dir/bash,dst=/bash,ro=true" \
   "--mount=type=bind,src=$dir/src,dst=$srcmnt" \
   $usb_options \
   -w $srcmnt \
   $img \
   bash --login -c build

else
  podman  run --rm -it \
   "--mount=type=bind,src=$dir/bash,dst=/bash,ro=true" \
   "--mount=type=bind,src=$dir/src,dst=$srcmnt" \
   $usb_options \
   -w $srcmnt \
   $img \
   bash
fi

