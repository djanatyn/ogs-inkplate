#ifndef BADUK_TYPES_H
#define BADUK_TYPES_H

#include <stdint.h>

#define BADUK_BOARD_SIZE 19
#define BADUK_EMPTY 0
#define BADUK_BLACK 1
#define BADUK_WHITE 2
#define BADUK_NO_LAST_MOVE 255
#define BADUK_PASS_MOVE 0xFFFF

struct BadukGameRecord {
  const uint16_t* moves;
  uint16_t move_count;
};

struct BadukState {
  uint8_t board[BADUK_BOARD_SIZE][BADUK_BOARD_SIZE];
  uint16_t game_index;
  uint16_t move_index;
  uint16_t move_count;
  uint8_t last_row;
  uint8_t last_col;
};

#endif
