#ifndef CAM_BASE64_HPP__
#define CAM_BASE64_HPP__
#include <cstdint>
#include <cstdio>

namespace base64 {
static constexpr char base64_chars[] =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

void printBase64(const void *buffer, uint32_t length) noexcept {
  const uint8_t *data = static_cast<const uint8_t *>(buffer);
  uint32_t i = 0;

  while (i + 2 < length) {
    uint32_t val = (data[i] << 16) | (data[i + 1] << 8) | data[i + 2];
    std::putchar(base64_chars[(val >> 18) & 0x3F]);
    std::putchar(base64_chars[(val >> 12) & 0x3F]);
    std::putchar(base64_chars[(val >> 6) & 0x3F]);
    std::putchar(base64_chars[val & 0x3F]);
    i += 3;
  }

  if (i < length) {
    uint32_t val = data[i] << 16;
    if (i + 1 < length) {
      val |= data[i + 1] << 8;
    }

    std::putchar(base64_chars[(val >> 18) & 0x3F]);
    std::putchar(base64_chars[(val >> 12) & 0x3F]);
    if (i + 1 < length) {
      std::putchar(base64_chars[(val >> 6) & 0x3F]);
    } else {
      std::putchar('=');
    }
    std::putchar('=');
  }
}

uint32_t convertBase64(const void *input, uint32_t inputLength, char *buffer,
                   uint32_t bufferLength) noexcept {
  const uint8_t *data = static_cast<const uint8_t *>(input);

  uint32_t requiredLength = 4 * ((inputLength + 2) / 3);
  if (bufferLength < requiredLength + 1) {
    return 0;
  }

  uint32_t i = 0, j = 0;
  while (i < inputLength) {
    uint32_t octet_a = i < inputLength ? data[i++] : 0;
    uint32_t octet_b = i < inputLength ? data[i++] : 0;
    uint32_t octet_c = i < inputLength ? data[i++] : 0;

    uint32_t triple = (octet_a << 16) | (octet_b << 8) | octet_c;

    buffer[j++] = base64_chars[(triple >> 18) & 0x3F];
    buffer[j++] = base64_chars[(triple >> 12) & 0x3F];
    buffer[j++] =
        (i > inputLength + 1) ? '=' : base64_chars[(triple >> 6) & 0x3F];
    buffer[j++] = (i > inputLength) ? '=' : base64_chars[triple & 0x3F];
  }
  buffer[j] = '\0';
  return j + 1;
}

} // namespace base64

#endif
