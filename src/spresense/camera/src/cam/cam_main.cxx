#include <cstdint>
#include <cstdio>
#include <nuttx/config.h>
#include <stdio.h>
#include <string.h>

#include "../include/base64.hpp"
#include "../include/camera.hpp"
#include "../include/usb.hpp"

extern "C" int main(int argc, FAR char *argv[]) {
  printf("Start cam program\n");

  // デバッグ用のシリアル(115200)から出すとクッソ遅いので、
  // 拡張ボードの USB から出力できるようにしています (cam usb として起動)
  //
  // nsh> sercon
  // nsh> cam usb
  //
  bool useExernalUSB = false;
  usb::USBSerial u;
  if (argc != 1) {
    if (strcmp("usb", argv[1]) == 0) {
      useExernalUSB = true;
    }
  }

  if (useExernalUSB) {
    printf("Use External USB Serial port.\n");
    printf("-- Baudrate: 30000000\n");
    u = usb::USBSerial(30000000);
    if (!u.ok()) {
      printf("-- Fail to init USB\n");
      return 1;
    }
    printf("-- init OK\n");
  }

  // ----- カメラの初期化 -----------------------------------
  cam::Camera::init();
  printf("Init camera\n");
  cam::Camera camera(cam::HD, cam::StillImage, 2);
  if (!camera.ok()) {
    printf("-- Fail to init camera\n");
    return 1;
  }
  if (!camera.setWhiteBalance(V4L2_WHITE_BALANCE_FLUORESCENT)) {
    printf("-- Fail to init white balance\n");
    return 1;
  }
  printf("-- init OK\n");

  // ----  キャプチャ開始 -------------------------------------
  printf("Start Capture\n");
  if (!camera.startCapture()) {
    printf("Fail to start camera\n");
    return 1;
  }

  auto ret = camera.dequeue();
  if (!ret) {
    printf("Fail to dequeue\n");
    return 1;
  }
  printf("End Capture\n");

  // ---- 出力(Base64) -----------------------------------------
  auto buf = ret->buffer();
  auto length = ret->getLength();
  printf("Output Base64: \n\n");

  if (useExernalUSB) {
    // 早い
    uint32_t buflen = 4 * ((length + 2) / 3) + 1;
    char *buffer = new char[buflen];
    auto len = base64::convertBase64(buf, length, buffer, buflen) - 1;

    uint32_t from = 0;
    while (from < len) {
      auto size = u.write(buffer + from, len - from);
      from += size;
    }
    delete[] buffer;
    u.write("\n\n", 2);
    printf("Success! Output Base64\n\n");
  } else {
    // 遅い
    base64::printBase64(buf, length);
    printf("\n\nSuccess! Output Base64\n\n");
  }

  printf("Done !\n");
  return 0;
}
