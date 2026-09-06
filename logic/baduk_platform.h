#ifndef BADUK_PLATFORM_H
#define BADUK_PLATFORM_H

#ifdef ARDUINO
#include <Arduino.h>
#else
#include <stdint.h>
#include <string.h>

#ifndef PROGMEM
#define PROGMEM
#endif

#ifndef memcpy_P
#define memcpy_P(dest, src, size) memcpy((dest), (src), (size))
#endif

#ifndef pgm_read_word
#define pgm_read_word(addr) (*(const uint16_t*)(addr))
#endif

#ifndef pgm_read_byte
#define pgm_read_byte(addr) (*(const uint8_t*)(addr))
#endif
#endif

#endif
