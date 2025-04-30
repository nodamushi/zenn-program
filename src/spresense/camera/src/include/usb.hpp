#ifndef CAM_USB_HPP__
#define CAM_USB_HPP__

#include <cerrno>
#include <cstring>
#include <fcntl.h>
#include <nuttx/usb/cdcacm.h>
#include <nuttx/usb/usbdev.h>
#include <nuttx/usb/usbdev_trace.h>
#include <sys/boardctl.h>
#include <sys/ioctl.h>
#include <termios.h>
#include <unistd.h>

#include "./raii.hpp"
#include "nuttx/fs/ioctl.h"

namespace usb {

struct USBSerial {
  USBSerial() noexcept : wfd_(), rfd_() {}
  USBSerial(uint32_t baudrate) noexcept : wfd_(), rfd_() {

    struct boardioc_usbdev_ctrl_s ctrl = {};
    FAR void *handle;
    ctrl.usbdev = BOARDIOC_USBDEV_CDCACM;
    ctrl.action = BOARDIOC_USBDEV_CONNECT;
    ctrl.instance = 0;
    ctrl.handle = &handle;
    boardctl(BOARDIOC_USBDEV_CONTROL, (uintptr_t)&ctrl);
    usbtrace_enable(TRACE_BITSET);

    raii::FileDescripter wfd(DEVNAME, O_WRONLY);
    while (!wfd.ok()) {
      int errcode = errno;
      /* ENOTCONN means that the USB device is not yet connected */
      if (errcode == ENOTCONN) {
        sleep(1);
        wfd = raii::FileDescripter(DEVNAME, O_WRONLY);
      } else {
        return;
      }
    }

    raii::FileDescripter rfd(DEVNAME, O_RDONLY | O_NONBLOCK);
    if (!rfd.ok()) {
      return;
    }
    struct termios tio;
    tcgetattr(rfd, &tio);

    tio.c_cflag |= CREAD;
    tio.c_cflag |= CLOCAL;
    tio.c_cflag &= ~CSIZE;
    tio.c_cflag |= CS8;
    tio.c_cflag &= ~CSTOPB;
    tio.c_cflag &= ~PARENB;

    cfsetspeed(&tio, baudrate);
    tcsetattr(rfd, TCSANOW, &tio);

    wfd_ = std::move(wfd);
    rfd_ = std::move(rfd);
  }

  bool ok() const noexcept { return wfd_.ok(); }
  int availableRead() const noexcept {
    if (!ok())
      return 0;

    int count;
    if (::ioctl(rfd_, FIONREAD, (long unsigned int)&count))
      count = 0;

    return count;
  }

  int read() const noexcept {
    if (!ok())
      return -1;
    char buf[1] = {0};
    ::read(rfd_, buf, 1);
    return (int)buf[0];
  }

  int availableWrite() const noexcept {
    if (!ok())
      return 0;
    int count;
    if (::ioctl(wfd_, FIONSPACE, (long unsigned int)&count))
      count = 0;
    return count;
  }

  void flush() const noexcept { ioctl(wfd_, TCIOFLUSH); }

  size_t write(const void *buffer, size_t size) const noexcept {
    if (!ok())
      return 0;
    return ::write(wfd_, buffer, size);
  }

  void writeAll(const void* buffer, size_t size) const noexcept {
    if (!ok()) return;
    size_t s = 0;
    const uint8_t * buf = (const uint8_t *)buffer;
    while (s < size) {
      auto x =  ::write(wfd_,buf + s, size - s);
      if (x < 0) break;
      s += x;
    }
  }

  void writeAll(const char* str) const noexcept {
    if (!ok()) return;
    auto len = ::strlen(str);
    writeAll(str, len);
  }

private:
  raii::FileDescripter wfd_;
  raii::FileDescripter rfd_;
  static constexpr const char *DEVNAME = "/dev/ttyACM0";
};

} // namespace usb

#endif
