#include <nuttx/config.h>
#include <stdio.h>
#include <sys/time.h>
#include <sys/types.h>

#include "../include/camera.hpp"
#include "../include/usb.hpp"
#include "nuttx/video/video_controls.h"

namespace {


void mainLoop(usb::USBSerial &usb) {

  bool videoEnable = false;
  bool stillImageEnable = false;
  bool error = false;

  uint32_t frame = 0;
  uint32_t lastMarker = 0xFA01FB00;

  cam::Camera camera;
  bool run = true;

  struct timeval start;
  enum v4l2_auto_n_preset_white_balance balance =
      V4L2_WHITE_BALANCE_FLUORESCENT;

  while (run) {
    if (!stillImageEnable && (!videoEnable || (frame & 63) == 0) &&
        usb.availableRead()) {
      int i = usb.read();
      if (i < 0) {
        printf("USB Serial Error.\n");
        run = false;
        break;
      }

      switch ((char)i) {
      case '0': // Health Check
        printf("Health Check.\n");
        if (error)
          usb.writeAll("rsp:HCK!");
        else
          usb.writeAll("rsp:HCK.");
        usb.flush();
        break;

      case '1': // KILL
        printf("KILL.\n");
        usb.writeAll("rsp:KIL.");
        usb.flush();
        run = false;
        break;

      case 'a': // Start
        if (!videoEnable) {
          camera = cam::Camera(cam::QVGA, cam::FPS120, 3);
          if (!camera.ok()) {
            printf("Fail to create camera instance.\n");
            usb.writeAll("rsp:VST!");
            usb.flush();
            error = true;
            break;
          }

          if (!camera.setWhiteBalance(balance)) {
            printf("Fail to set wihte balance.\n");
            usb.writeAll("rsp:VST!");
            usb.flush();
            error = true;
            break;
          }

          if (!camera.startCapture()) {
            printf("Fail to start capture.\n");
            usb.writeAll("rsp:VST!");
            usb.flush();
            error = true;
            break;
          }

          usb.writeAll("rsp:VST.");
          usb.flush();
          printf("Video Start\n");
          videoEnable = true;
          frame = 0;
          gettimeofday(&start, NULL);
        } else {
          usb.writeAll("rsp:VST!");
          usb.flush();
        }
        break;

      case 'b': // Stop
        if (videoEnable) {
          videoEnable = false;
          camera = cam::Camera();
          usb.writeAll("rsp:VSP.");
          usb.flush();
          struct timeval end;
          gettimeofday(&end, NULL);
          float elapsed = (end.tv_sec - start.tv_sec) +
                          (end.tv_usec - start.tv_usec) / 1000000.0f;

          float fps = frame / elapsed;
          printf("Video Stop\n");
          printf("Frames: %lu\n", frame);
          printf("Time  : %.3f [s]\n", elapsed);
          printf("FPS   : %.2f\n", fps);
        } else {
          usb.writeAll("rsp:VSP!");
          usb.flush();
        }
        break;

      case 's': // StillImage
        if (!videoEnable) {
          camera = cam::Camera(cam::P2M, cam::StillImage, 1);
          if (!camera.ok()) {
            printf("Fail to create camera instance.\n");
            usb.writeAll("rsp:SST!");
            usb.flush();
            error = true;
            break;
          }

          if (!camera.setWhiteBalance(balance)) {
            printf("Fail to set wihte balance.\n");
            usb.writeAll("rsp:SST!");
            usb.flush();
            error = true;
            break;
          }

          if (!camera.startCapture()) {
            printf("Fail to start capture.\n");
            usb.writeAll("rsp:SST!");
            usb.flush();
            error = true;
            break;
          }

          usb.writeAll("rsp:SST.");
          usb.flush();
          printf("StilImage Start\n");
          stillImageEnable = true;
          frame = 0xFFFFFFFF;
        }
        break;
      }
    } // end usb

    if (videoEnable || stillImageEnable) {
      auto ret = camera.dequeue();
      if (ret) {
        auto buf = ret->buffer();
        auto len = ret->getLength();
        uint32_t total_len = len + 4;
        usb.writeAll("jpg:", 4);
        usb.writeAll(&frame, 4);
        usb.writeAll(&total_len, 4);
        usb.writeAll(buf, len);

        if (stillImageEnable) {
          camera = cam::Camera();
          stillImageEnable = false;
        }else{
          camera.enqueue(ret);
        }
        usb.writeAll(&lastMarker, 4);
        usb.flush();
        frame++;

      }
    }
  }
}

} // namespace

extern "C" int main(int argc, FAR char *argv[]) {
  printf("Start Program\n");
  usb::USBSerial usb(200000000);
  printf("Start\n");
  mainLoop(usb);
  printf("Exit\n");
  return 0;
}
