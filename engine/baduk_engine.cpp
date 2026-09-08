#include "baduk_engine.h"
#include "baduk_platform.h"

static uint8_t baduk_other_color(uint8_t color) {
  if (color == BADUK_BLACK) {
    return BADUK_WHITE;
  }

  return BADUK_BLACK;
}

static uint8_t baduk_on_board(int row, int col) {
  return row >= 0 && row < BADUK_BOARD_SIZE && col >= 0 &&
         col < BADUK_BOARD_SIZE;
}

void baduk_reset(BadukState *state) {
  uint8_t row;
  uint8_t col;

  for (row = 0; row < BADUK_BOARD_SIZE; row++) {
    for (col = 0; col < BADUK_BOARD_SIZE; col++) {
      state->board[row][col] = BADUK_EMPTY;
    }
  }

  state->game_index = 0;
  state->move_index = 0;
  state->move_count = 0;
  state->last_row = BADUK_NO_LAST_MOVE;
  state->last_col = BADUK_NO_LAST_MOVE;
}

uint8_t baduk_load_game(BadukState *state, const BadukGameRecord *games,
                        uint16_t game_count, uint16_t game_index) {
  BadukGameRecord game;

  if (game_index >= game_count) {
    return 0;
  }

  memcpy_P(&game, &games[game_index], sizeof(BadukGameRecord));
  baduk_reset(state);

  state->game_index = game_index;
  state->move_count = game.move_count;

  return 1;
}

static uint8_t
baduk_group_has_liberty(BadukState *state, uint8_t start_row, uint8_t start_col,
                        uint8_t visited[BADUK_BOARD_SIZE][BADUK_BOARD_SIZE]) {
  uint8_t color;
  uint8_t stack_row[BADUK_BOARD_SIZE * BADUK_BOARD_SIZE];
  uint8_t stack_col[BADUK_BOARD_SIZE * BADUK_BOARD_SIZE];
  uint16_t stack_size;
  int dirs[4][2] = {
      {-1, 0},
      {1, 0},
      {0, -1},
      {0, 1},
  };

  color = state->board[start_row][start_col];
  stack_size = 0;

  stack_row[stack_size] = start_row;
  stack_col[stack_size] = start_col;
  stack_size++;
  visited[start_row][start_col] = 1;

  while (stack_size > 0) {
    int row;
    int col;
    uint8_t i;

    stack_size--;
    row = stack_row[stack_size];
    col = stack_col[stack_size];

    for (i = 0; i < 4; i++) {
      int next_row = row + dirs[i][0];
      int next_col = col + dirs[i][1];

      if (!baduk_on_board(next_row, next_col)) {
        continue;
      }

      if (state->board[next_row][next_col] == BADUK_EMPTY) {
        return 1;
      }

      if (state->board[next_row][next_col] == color &&
          visited[next_row][next_col] == 0) {
        visited[next_row][next_col] = 1;
        stack_row[stack_size] = next_row;
        stack_col[stack_size] = next_col;
        stack_size++;
      }
    }
  }

  return 0;
}

static void baduk_remove_group(BadukState *state, uint8_t start_row,
                               uint8_t start_col) {
  uint8_t color;
  uint8_t stack_row[BADUK_BOARD_SIZE * BADUK_BOARD_SIZE];
  uint8_t stack_col[BADUK_BOARD_SIZE * BADUK_BOARD_SIZE];
  uint16_t stack_size;
  int dirs[4][2] = {
      {-1, 0},
      {1, 0},
      {0, -1},
      {0, 1},
  };

  color = state->board[start_row][start_col];
  stack_size = 0;

  stack_row[stack_size] = start_row;
  stack_col[stack_size] = start_col;
  stack_size++;
  state->board[start_row][start_col] = BADUK_EMPTY;

  while (stack_size > 0) {
    int row;
    int col;
    uint8_t i;

    stack_size--;
    row = stack_row[stack_size];
    col = stack_col[stack_size];

    for (i = 0; i < 4; i++) {
      int next_row = row + dirs[i][0];
      int next_col = col + dirs[i][1];

      if (!baduk_on_board(next_row, next_col)) {
        continue;
      }

      if (state->board[next_row][next_col] == color) {
        state->board[next_row][next_col] = BADUK_EMPTY;
        stack_row[stack_size] = next_row;
        stack_col[stack_size] = next_col;
        stack_size++;
      }
    }
  }
}

uint8_t baduk_play_move(BadukState *state, uint16_t encoded_move,
                        uint8_t color) {
  uint8_t row;
  uint8_t col;
  uint8_t enemy;
  uint8_t i;
  int dirs[4][2] = {
      {-1, 0},
      {1, 0},
      {0, -1},
      {0, 1},
  };

  if (encoded_move == BADUK_PASS_MOVE) {
    state->last_row = BADUK_NO_LAST_MOVE;
    state->last_col = BADUK_NO_LAST_MOVE;
    return 1;
  }

  col = encoded_move / BADUK_BOARD_SIZE;
  row = encoded_move % BADUK_BOARD_SIZE;

  if (row >= BADUK_BOARD_SIZE || col >= BADUK_BOARD_SIZE) {
    return 0;
  }

  if (state->board[row][col] != BADUK_EMPTY) {
    return 0;
  }

  state->board[row][col] = color;
  enemy = baduk_other_color(color);

  for (i = 0; i < 4; i++) {
    int next_row = row + dirs[i][0];
    int next_col = col + dirs[i][1];

    if (!baduk_on_board(next_row, next_col)) {
      continue;
    }

    if (state->board[next_row][next_col] == enemy) {
      uint8_t visited[BADUK_BOARD_SIZE][BADUK_BOARD_SIZE] = {0};

      if (!baduk_group_has_liberty(state, next_row, next_col, visited)) {
        baduk_remove_group(state, next_row, next_col);
      }
    }
  }

  {
    uint8_t visited[BADUK_BOARD_SIZE][BADUK_BOARD_SIZE] = {0};

    if (!baduk_group_has_liberty(state, row, col, visited)) {
      state->board[row][col] = BADUK_EMPTY;
      return 0;
    }
  }

  state->last_row = row;
  state->last_col = col;

  return 1;
}

uint8_t baduk_play_next_move(BadukState *state, const BadukGameRecord *games) {
  BadukGameRecord game;
  uint16_t encoded_move;
  uint8_t color;

  if (state->move_index >= state->move_count) {
    return 0;
  }

  memcpy_P(&game, &games[state->game_index], sizeof(BadukGameRecord));
  encoded_move = pgm_read_word(&game.moves[state->move_index]);

  if ((state->move_index % 2) == 0) {
    color = BADUK_BLACK;
  } else {
    color = BADUK_WHITE;
  }

  if (!baduk_play_move(state, encoded_move, color)) {
    return 0;
  }

  state->move_index++;

  return 1;
}
