#include "screenshot.h"

#include <mbedtls/base64.h>

static void screenshot_write_base64(const uint8_t* data, size_t data_len) {
  const size_t input_chunk_size = 45;
  uint8_t encoded[65];
  size_t offset;

  for (offset = 0; offset < data_len; offset += input_chunk_size) {
    size_t chunk_len = data_len - offset;
    size_t encoded_len = 0;
    int result;

    if (chunk_len > input_chunk_size) {
      chunk_len = input_chunk_size;
    }

    result = mbedtls_base64_encode(encoded, sizeof(encoded), &encoded_len,
                                   data + offset, chunk_len);

    if (result != 0) {
      Serial.println("BASE64_ERROR");
      return;
    }

    encoded[encoded_len] = '\0';
    Serial.println((char*)encoded);
  }
}

void screenshot_dump(Inkplate* display) {
  size_t size = E_INK_WIDTH * E_INK_HEIGHT / 8;

  Serial.println("SCREENSHOT_BEGIN");
  Serial.print("WIDTH ");
  Serial.println(E_INK_WIDTH);
  Serial.print("HEIGHT ");
  Serial.println(E_INK_HEIGHT);
  Serial.println("FORMAT INKPLATE_1BIT_LSB_FIRST_BLACK_1");
  Serial.print("BYTES ");
  Serial.println(size);
  Serial.println("DATA_BEGIN");

  screenshot_write_base64(display->_partial, size);

  Serial.println("DATA_END");
  Serial.println("SCREENSHOT_END");
  Serial.flush();
}

void screenshot_check_serial(Inkplate* display) {
  int ch;

  if (!Serial.available()) {
    return;
  }

  ch = Serial.read();

  if (ch == 's' || ch == 'S') {
    screenshot_dump(display);
  }
}
