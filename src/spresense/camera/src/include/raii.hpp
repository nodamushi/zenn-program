#ifndef CAM_RAII_HPP__
#define CAM_RAII_HPP__

#include <fcntl.h>
#include <stdio.h>
#include <utility>

namespace raii {
/**
 * @brief FileDescripter の RAII クラス
 */
struct FileDescripter {
  FileDescripter() noexcept : fd(-1) {}
  FileDescripter(const char *file, int flag) noexcept
      : fd(::open(file, flag)) {}
  ~FileDescripter() noexcept {
    if (fd >= 0) {
      ::close(fd);
    }
  }

  FileDescripter(FileDescripter &&x) noexcept : fd(std::exchange(x.fd, -1)) {}
  FileDescripter &operator=(FileDescripter &&x) noexcept {
    if (&x == this)
      return *this;
    if (fd >= 0)
      ::close(fd);
    fd = std::exchange(x.fd, -1);
    return *this;
  }

  inline operator int() const noexcept { return fd; }
  inline bool ok() const noexcept { return fd >= 0; }

private:
  int fd;
  FileDescripter(const FileDescripter &) = delete;
  FileDescripter &operator=(const FileDescripter &) = delete;
};

/**
 * @brief memalign の RAII クラス
 */
struct AlignedMem {
  AlignedMem() noexcept : ptr(nullptr) {}
  AlignedMem(size_t alignment, size_t length) noexcept
      : ptr(memalign(alignment, length)) {}
  AlignedMem(AlignedMem &&x) noexcept : ptr(std::exchange(x.ptr, nullptr)) {}
  AlignedMem &operator=(AlignedMem &&x) noexcept {
    if (&x == this)
      return *this;
    if (ptr)
      free(ptr);
    ptr = std::exchange(x.ptr, nullptr);
    return *this;
  }
  ~AlignedMem() noexcept {
    if (ptr)
      free(ptr);
  }
  inline void *get() const noexcept { return ptr; }
  inline operator void *() const noexcept { return ptr; }
  inline bool ok() const noexcept { return ptr; }

private:
  void *ptr;
  AlignedMem(const AlignedMem &) = delete;
  AlignedMem &operator=(const AlignedMem &) = delete;
};

} // namespace raii

#endif