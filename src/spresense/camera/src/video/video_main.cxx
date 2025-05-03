#include <atomic>
#include <nuttx/config.h>
#include <pthread.h>
#include <stdio.h>
#include <sys/time.h>
#include <sys/types.h>

#include "../include/camera.hpp"
#include "../include/queue.hpp"
#include "../include/usb.hpp"
#include "nuttx/semaphore.h"
#include "nuttx/video/video_controls.h"
#include <nuttx/semaphore.h>

namespace {

/**
 * カメラスレッドの引数
 */
struct CameraArg {
  usb::USBSerial &usb;
  std::atomic_bool run;
  std::atomic_bool videoCapture;
  std::atomic_bool working;
  cam::VideoSize videoSize;
  cam::VideoFPS fps;
  uint8_t bufferSize;
  const char *okResponse;
  const char *errResponse;
  enum v4l2_auto_n_preset_white_balance balance;
  sem_t sem;

  CameraArg(usb::USBSerial &u) noexcept
      : usb(u), run(true), videoCapture(true), working(false), videoSize(),
        fps(), bufferSize(), okResponse(), errResponse(),
        balance(V4L2_WHITE_BALANCE_FLUORESCENT), sem() {
    nxsem_init(&sem, 0, 0);
  }
  ~CameraArg() { nxsem_destroy(&sem); }
  void wait() { nxsem_wait(&sem); }
  void wake() { nxsem_post(&sem); }
};

/**
 * カメラスレッド
 */
void *cameraThread(void *arg) {
  printf("Camera Thread Start\n");
  auto &a = *(CameraArg *)arg;
  while (a.run) {
    a.wait();
    if (!a.run)
      return nullptr;
    a.working = true;
    printf("Task start\n");

    cam::Camera camera(a.videoSize, a.fps, a.bufferSize);
    if (!camera.ok()) {
      printf("Fail to create camera instance.\n");
      a.usb.writeAll(a.errResponse);
      a.usb.flush();
      a.working = false;
      continue;
    }

    if (!camera.setWhiteBalance(a.balance)) {
      printf("Fail to set wihte balance.\n");
      a.usb.writeAll(a.errResponse);
      a.usb.flush();
      a.working = false;
      continue;
    }

    if (!camera.startCapture()) {
      printf("Fail to start capture.\n");
      a.usb.writeAll(a.errResponse);
      a.usb.flush();
      a.working = false;
      continue;
    }

    a.usb.writeAll(a.okResponse);
    a.usb.flush();
    printf("Video Start!\n");

    struct timeval start;
    gettimeofday(&start, NULL);

    const bool stillImage = a.fps == cam::StillImage;
    uint32_t frame = stillImage ? 0xFFFFFFFF : 0;
    uint32_t lastMarker = 0xFA01FB00;

    uint8_t fps = 0;
    switch (a.fps) {
    case cam::FPS120:
      fps = 120;
      break;
    case cam::FPS60:
      fps = 60;
      break;
    case cam::FPS30:
      fps = 30;
      break;
    case cam::FPS15:
      fps = 15;
      break;
    case cam::FPS7_5:
      fps = 7;
      break;
    case cam::FPS6:
      fps = 6;
      break;
    case cam::FPS5:
      fps = 5;
      break;
    case cam::StillImage:
      fps = 0;
      break;
    }

    while (a.videoCapture) {
      auto buffer = camera.dequeue();
      if (buffer) {
        auto buf = buffer->buffer();
        auto len = buffer->getLength();
        uint32_t total_len = len + 4;
        a.usb.writeAll("jpg:", 4);
        a.usb.writeAll(&fps, 1);
        a.usb.writeAll(&frame, 4);
        a.usb.writeAll(&total_len, 4);
        a.usb.writeAll(buf, len);
        a.usb.writeAll(&lastMarker, 4);
        a.usb.flush();
        camera.enqueue(buffer);
        frame++;
        if (stillImage)
          break;
      }
    }

    if (!a.videoCapture) {
      a.usb.writeAll("rsp:VSP.");
      a.usb.flush();
    }

    if (frame != 0) {
      struct timeval end;
      gettimeofday(&end, NULL);
      float elapsed = (end.tv_sec - start.tv_sec) +
                      (end.tv_usec - start.tv_usec) / 1000000.0f;
      float f = frame / elapsed;
      printf("Video Stop\n");
      printf("Frames: %lu\n", frame);
      printf("Time  : %.3f [s]\n", elapsed);
      printf("FPS   : %.2f\n", f);
    }
    a.working = false;
  }
  a.working = false;
  return nullptr;
}

void mainLoop(usb::USBSerial &usb) {

  CameraArg arg(usb);
  pthread_t cameraP = -1;
  struct sched_param sparam = {};
  sparam.sched_priority = 110;
  pthread_attr_t attr;
  pthread_attr_init(&attr);
  pthread_attr_setschedparam(&attr, &sparam);
  pthread_create(&cameraP, &attr, cameraThread, (void *)&arg);

  bool run = true;
  while (run) {

    int i = usb.read();
    if (i < 0) {
      printf("USB Serial Error.\n");
      run = false;
      break;
    }

    switch ((char)i) {
    case '0': // Stop
      arg.videoCapture = false;
      break;

    case '1': // KILL
      printf("KILL.\n");
      arg.videoCapture = false;
      arg.run = false;
      arg.wake();
      pthread_join(cameraP, nullptr);
      cameraP = -1;

      usb.writeAll("rsp:KIL.");
      usb.flush();
      run = false;
      break;

    case 'a': // Start
      if (!arg.working) {
        arg.videoSize = cam::QVGA;
        arg.fps = cam::FPS120;
        arg.bufferSize = 3;
        arg.okResponse = "rsp:VST.";
        arg.errResponse = "rsp:VST!";
        arg.working = true;
        arg.videoCapture = true;
        arg.wake();
      }
      break;

    case 'b': // Start
      if (!arg.working) {
        arg.videoSize = cam::VGA;
        arg.fps = cam::FPS60;
        arg.bufferSize = 3;
        arg.okResponse = "rsp:VST.";
        arg.errResponse = "rsp:VST!";
        arg.working = true;
        arg.videoCapture = true;
        arg.wake();
      }
      break;

    case 'c': // Start
      if (!arg.working) {
        arg.videoSize = cam::HD;
        arg.fps = cam::FPS30;
        arg.bufferSize = 3;
        arg.okResponse = "rsp:VST.";
        arg.errResponse = "rsp:VST!";
        arg.working = true;
        arg.videoCapture = true;
        arg.wake();
      }
      break;

    case 's': // StillImage
      if (!arg.working) {
        arg.videoSize = cam::FullHD;
        arg.fps = cam::StillImage;
        arg.bufferSize = 1;
        arg.okResponse = "rsp:SST.";
        arg.errResponse = "rsp:SST!";
        arg.working = true;
        arg.videoCapture = true;
        arg.wake();
      }
      break;
      // white balance
      case 'A': arg.balance = V4L2_WHITE_BALANCE_MANUAL; break;
      case 'B': arg.balance = V4L2_WHITE_BALANCE_AUTO; break;
      case 'C': arg.balance = V4L2_WHITE_BALANCE_INCANDESCENT; break;
      case 'D': arg.balance = V4L2_WHITE_BALANCE_FLUORESCENT; break;
      case 'E': arg.balance = V4L2_WHITE_BALANCE_FLUORESCENT_H; break;
      case 'F': arg.balance = V4L2_WHITE_BALANCE_HORIZON; break;
      case 'G': arg.balance = V4L2_WHITE_BALANCE_DAYLIGHT; break;
      case 'H': arg.balance = V4L2_WHITE_BALANCE_FLASH; break;
      case 'I': arg.balance = V4L2_WHITE_BALANCE_CLOUDY; break;
      case 'J': arg.balance = V4L2_WHITE_BALANCE_SHADE; break;
    }
  }

  if (cameraP != -1) {
    arg.videoCapture = false;
    arg.run = false;
    arg.wake();
    pthread_join(cameraP, nullptr);
    cameraP = -1;
  }
}

} // namespace

extern "C" int main(int argc, FAR char *argv[]) {
  cam::Camera::init();
  printf("Start Program\n");
  usb::USBSerial usb(9800);
  printf("Start\n");
  mainLoop(usb);
  usb.writeAll("rsp:EXT.");
  usb.flush();
  printf("Exit\n");
  return 0;
}
