// Nead device/camera
#ifndef CAM_CAMERA_HXX__
#define CAM_CAMERA_HXX__

#include "nuttx/video/video_controls.h"
#include <cstdlib>
#include <memory>
#include <nuttx/config.h>
#include <nuttx/video/video.h>
#include <sys/ioctl.h>
#include <utility>

#include "./raii.hpp"

namespace cam {

/**
 * @brief 動画のFPS. 画像サイズ依存がある
 * @see
 * https://developer.sony.com/spresense/development-guides/sdk_developer_guide_ja?#_image%E3%82%B5%E3%82%A4%E3%82%BA%E3%81%A8frame_rate%E3%81%AE%E5%88%B6%E7%B4%84)
 */
enum VideoFPS {
  /** FPS 120. QQVGA Only */
  FPS120,
  /** FPS 60 */
  FPS60,
  /** FPS 30 */
  FPS30,
  /** FPS 15 */
  FPS15,
  /** FPS 7.5 */
  FPS7_5,
  /** FPS 6 */
  FPS6,
  /** FPS 5 */
  FPS5,
  /** 静止画 */
  StillImage
};

/** @brief 利用可能なビデオのサイズ */
enum VideoSize {
  /** 160 x 120 */
  QQVGA,
  /** 320 x 240 */
  QVGA,
  /** 640 x 480 */
  VGA,
  /** 1280 x 720 */
  HD,
  /**  ISX012(HDR版ではないカメラ) のみ. 1920x1080 */
  FullHD,
  /**  ISX012(HDR版ではないカメラ) のみ. 2M Pixel(1632x1244) */
  P2M,
  /**  ISX012(HDR版ではないカメラ) の最大値. 5M Pixel(2592x1944). */
  P5M
};

/** @brief 画像の幅と高さ */
struct Size {
  uint16_t width;
  uint16_t height;

  Size() noexcept : width(0), height(0) {}
  Size(uint16_t w, uint16_t h) noexcept : width(w), height(h) {}
  Size(VideoSize s) noexcept : width(), height() {
    switch (s) {
    case QQVGA:
      width = VIDEO_HSIZE_QVGA / 2;
      height = VIDEO_VSIZE_QVGA / 2;
      break;
    case QVGA:
      width = VIDEO_HSIZE_QVGA;
      height = VIDEO_VSIZE_QVGA;
      break;
    case VGA:
      width = VIDEO_HSIZE_VGA;
      height = VIDEO_VSIZE_VGA;
      break;
    case FullHD:
      width = 1920;
      height = 1080;
      break;
    case P2M:
      width = 1632;
      height = 1244;
      break;
    case P5M:
      width = 2592;
      height = 1944;
      break;
    default:
      width = VIDEO_HSIZE_HD;
      height = VIDEO_VSIZE_HD;
      break;
    }
  }
};

/** @brief ビデオバッファー */
struct VideoBuffer {
  VideoBuffer() noexcept : mem_(), max_length_(0), data_length_(0), id_(0) {}
  VideoBuffer(uint16_t idx, uint32_t length) noexcept
      : mem_(32, length), max_length_(length), data_length_(0), id_(idx) {}

  inline bool ok() const noexcept { return mem_.ok(); }

  /** @brief データポインタを返す */
  inline void *buffer() const noexcept { return (void *)mem_; }

  /** @brief データの長さ(byte) を設定する */
  inline void setLength(uint32_t length) noexcept { data_length_ = length; }

  /** @brief データの長さ(byte) を返す */
  inline uint32_t getLength() const noexcept { return data_length_; }

  /** @brief バッファのサイズそのもの */
  inline uint32_t getMaxBufferLength() const noexcept { return max_length_; }

  /** @brief VideoBuffersにおけるインデックスを返す */
  inline uint16_t getIndex() const noexcept { return id_; }

private:
  raii::AlignedMem mem_;
  uint32_t max_length_;
  uint32_t data_length_;
  uint16_t id_;
};

// see:
// https://developer.sony.com/spresense/development-guides/sdk_developer_guide_ja#_%E6%A6%82%E8%A6%81_4
inline constexpr enum v4l2_buf_type get_v4l2_buf_type(bool movie) {
  return movie ? V4L2_BUF_TYPE_VIDEO_CAPTURE : V4L2_BUF_TYPE_STILL_CAPTURE;
}

/** @brief VideoBuffer の配列 */
struct VideoBuffers {

  VideoBuffers() noexcept : size_(0), bufs_(nullptr) {}

  VideoBuffers(int fd, bool is_movie, uint8_t bufferSize,
               uint32_t eachBufferLength) noexcept
      : size_(0), bufs_(nullptr) {

    // Get buffer memory
    auto bufs = std::make_unique<VideoBuffer[]>(bufferSize);
    if (!bufs) {
      return; // TODO error
    }
    for (uint16_t i = 0; i < bufferSize; i++) {
      bufs[i] = VideoBuffer(i, eachBufferLength);
      if (!bufs[i].ok()) {
        return; // TODO error
      }
    }

    // Set driver
    auto buf_type = get_v4l2_buf_type(is_movie);
    struct v4l2_requestbuffers req = {};
    req.type = buf_type;
    req.memory = V4L2_MEMORY_USERPTR;
    req.count = bufferSize;
    req.mode = V4L2_BUF_MODE_RING;
    if (::ioctl(fd, VIDIOC_REQBUFS, (unsigned long)&req) < 0) {
      return; // TODO error
    }

    for (uint16_t i = 0; i < bufferSize; i++) {
      struct v4l2_buffer buf = {};
      buf.type = buf_type;
      buf.memory = V4L2_MEMORY_USERPTR;
      buf.index = i;
      buf.m.userptr = (unsigned long)bufs[i].buffer();
      buf.length = eachBufferLength;

      if (::ioctl(fd, VIDIOC_QBUF, (unsigned long)&buf)) {
        return; // TODO error
      }
    }

    // OK
    size_ = bufferSize;
    bufs_ = std::move(bufs);
  }

  inline bool ok() const noexcept { return (bool)bufs_; }

  inline VideoBuffer *at(uint16_t i) const noexcept {
    if (ok() && i < size_)
      return &bufs_[i];
    return nullptr;
  }

private:
  uint8_t size_;
  std::unique_ptr<VideoBuffer[]> bufs_;
};

/**
 * @brief JPEG カメラオブジェクト
 */
struct Camera {
  /**
   * @param videoSize 画像サイズ
   * @param fps 動画の FPS.
   * @param bufferSize バッファ数
   */
  Camera(VideoSize videoSize, VideoFPS fps, uint8_t bufferSize) noexcept
      : Camera(Size(videoSize), fps, bufferSize) {}

  /**
   * @param videoSize 画像サイズ
   * @param fps 動画の FPS.
   * @param bufferSize バッファ数
   */
  Camera(Size videoSize, VideoFPS fps, uint8_t bufferSize) noexcept
      : fd_(), bufs_(), is_movie_(fps != StillImage), size_(videoSize),
        started_(false) {

    // see
    // https://developer.sony.com/spresense/development-guides/sdk_developer_guide_ja#_%E6%A6%82%E8%A6%81_4
    ::video_initialize(VIDEO_DEV_PATH);
    raii::FileDescripter fd(VIDEO_DEV_PATH, 0);
    if (!fd.ok()) {
      return;
    }

    // Set pixel data format and image resolution
    auto buf_type = get_v4l2_buf_type(is_movie_);
    struct v4l2_format fmt = {};
    fmt.type = buf_type;
    fmt.fmt.pix.width = videoSize.width;
    fmt.fmt.pix.height = videoSize.height;
    fmt.fmt.pix.field = V4L2_FIELD_ANY;
    fmt.fmt.pix.pixelformat = V4L2_PIX_FMT_JPEG;
    if (::ioctl((int)fd, VIDIOC_S_FMT, (unsigned long)&fmt) < 0) {
      return; // TODO error
    }

    if (is_movie_) {
      struct v4l2_streamparm parm = {};
      parm.type = V4L2_BUF_TYPE_VIDEO_CAPTURE;
      uint32_t num = 1;
      uint32_t den = 1;
      switch (fps) {
      case FPS120:
        den = 120;
        break;
      case FPS60:
        den = 60;
        break;
      case FPS30:
        den = 30;
        break;
      case FPS15:
        den = 15;
        break;
      case FPS7_5:
        num = 2;
        den = 15;
        break;
      case FPS6:
        den = 6;
        break;
      default:
        den = 5;
        break;
      }
      parm.parm.capture.timeperframe.numerator = num;
      parm.parm.capture.timeperframe.denominator = den;
      if (ioctl(fd, VIDIOC_S_PARM, &parm) < 0) {
        return; // TODO error
      }
    }

    uint32_t length =
        (uint32_t)videoSize.width * videoSize.height * sizeof(uint16_t) / 7;
    VideoBuffers bufs((int)fd, is_movie_, bufferSize, length);
    if (!bufs.ok()) {
      return; // TODO error
    }

    fd_ = std::move(fd);
    bufs_ = std::move(bufs);
  }

  ~Camera() noexcept {
    if (ok()) {
      stopCapture();
    }
  }

  inline bool ok() const noexcept { return fd_.ok(); }

  /** @brief ホワイトバランスの設定をする */
  bool setWhiteBalance(enum v4l2_auto_n_preset_white_balance balance) noexcept {
    struct v4l2_ext_control ctl_param = {};
    ctl_param.id = V4L2_CID_AUTO_N_PRESET_WHITE_BALANCE;
    ctl_param.value = balance;

    struct v4l2_ext_controls param = {};
    param.ctrl_class = V4L2_CTRL_CLASS_CAMERA;
    param.count = 1;
    param.controls = &ctl_param;

    if (::ioctl((int)fd_, VIDIOC_S_EXT_CTRLS, (unsigned long)&param)) {
      return false;
    }

    return true;
  }

  /** @brief キャプチャを開始する */
  bool startCapture() noexcept {
    if (!ok())
      return false;
    if (started_)
      return true;

    enum v4l2_buf_type type = V4L2_BUF_TYPE_VIDEO_CAPTURE;
    if (::ioctl(fd_, VIDIOC_STREAMON, (unsigned long)&type) < 0) {
      return false;
    }
    started_ = true;
    if (!is_movie_) {
      if (::ioctl(fd_, VIDIOC_TAKEPICT_START, 0)) {
        return false;
      }
    }
    return true;
  }

  /** @brief キャプチャを停止する */
  bool stopCapture() noexcept {
    if (!started_)
      return true;

    if (!is_movie_)
      if (::ioctl(fd_, VIDIOC_TAKEPICT_STOP, 0))
        return false;

    enum v4l2_buf_type type = V4L2_BUF_TYPE_VIDEO_CAPTURE;
    if (::ioctl(fd_, VIDIOC_STREAMOFF, (unsigned long)&type))
      return false;
    started_ = false;
    return true;
  }

  /**
   * @brief 画像を取得する。
   *
   * @return const VideoBuffer* . 動画の場合は `enqueue`
   * して再び使えるようにすること
   */
  const VideoBuffer *dequeue() const noexcept {
    if (!ok())
      return nullptr;
    v4l2_buffer_t buf = {};
    buf.type = get_v4l2_buf_type(is_movie_);
    buf.memory = V4L2_MEMORY_USERPTR;
    if (::ioctl((int)fd_, VIDIOC_DQBUF, (unsigned long)&buf))
      return nullptr;

    auto ret = bufs_.at(buf.index);
    if (ret)
      ret->setLength(buf.bytesused);

    return ret;
  }

  /** @brief 再度バッファを使えるようにする */
  bool enqueue(const VideoBuffer *b) const noexcept {
    if (!ok() || !b || b != bufs_.at(b->getIndex()))
      return false;

    struct v4l2_buffer buf = {};
    buf.type = get_v4l2_buf_type(is_movie_);
    buf.memory = V4L2_MEMORY_USERPTR;
    buf.index = b->getIndex();
    buf.m.userptr = (unsigned long)b->buffer();
    buf.length = b->getMaxBufferLength();

    return ::ioctl((int)fd_, VIDIOC_QBUF, (unsigned long)&buf) == 0;
  }

private:
  raii::FileDescripter fd_;
  VideoBuffers bufs_;
  bool is_movie_;
  Size size_;
  bool started_;

  static constexpr const char *VIDEO_DEV_PATH = "/dev/video0";
};

} // namespace cam
#endif
