#include "display.h"

static void display_draw_board(Inkplate *display);
static void display_draw_grid_lines(Inkplate *display);
static void display_draw_hoshi_points(Inkplate *display);
static void display_draw_stones(Inkplate *display, BadukState *baduk_state);
static void display_draw_game_info(Inkplate *display, BadukState *baduk_state,
                                   uint16_t total_games);
static int display_pixel_x(int board_col);
static int display_pixel_y(int board_row);

void display_init(DisplayState *display_state) {
  display_state->partial_update_count = 0;
}

static int display_pixel_x(int board_col) {
  return BOARD_OFFSET_X + (board_col * GRID_SPACING);
}

static int display_pixel_y(int board_row) {
  return BOARD_OFFSET_Y + (board_row * GRID_SPACING);
}

void display_draw_all(Inkplate *display, DisplayState *display_state,
                      BadukState *baduk_state, uint16_t total_games) {
  display->clearDisplay();
  display_draw_board(display);
  display_draw_stones(display, baduk_state);
  display_draw_game_info(display, baduk_state, total_games);

  display_state->partial_update_count++;
  if (display_state->partial_update_count >= FULL_REFRESH_EVERY) {
    display->display();
    display_state->partial_update_count = 0;
  } else {
    display->partialUpdate();
  }
}

static void display_draw_board(Inkplate *display) {
  display->setTextSize(1);
  display->setTextColor(BLACK);

  // Draw grid lines
  display_draw_grid_lines(display);

  // Draw hoshi (star) points
  display_draw_hoshi_points(display);

  // Draw board edge
  display->drawRect(display_pixel_x(0), display_pixel_y(0),
                    GRID_SPACING * (BADUK_BOARD_SIZE - 1),
                    GRID_SPACING * (BADUK_BOARD_SIZE - 1), BLACK);
}

static void display_draw_grid_lines(Inkplate *display) {
  // Draw vertical lines
  for (int col = 0; col < BADUK_BOARD_SIZE; col++) {
    int x = display_pixel_x(col);
    display->drawLine(x, display_pixel_y(0), x,
                      display_pixel_y(BADUK_BOARD_SIZE - 1), BLACK);
  }

  // Draw horizontal lines
  for (int row = 0; row < BADUK_BOARD_SIZE; row++) {
    int y = display_pixel_y(row);
    display->drawLine(display_pixel_x(0), y,
                      display_pixel_x(BADUK_BOARD_SIZE - 1), y, BLACK);
  }
}

static void display_draw_hoshi_points(Inkplate *display) {
  // Hoshi (star) points on a 19x19 board
  int hoshis[9][2] = {{3, 3},  {3, 9},  {3, 15}, {9, 3},  {9, 9},
                      {9, 15}, {15, 3}, {15, 9}, {15, 15}};

  for (int i = 0; i < 9; i++) {
    int x = display_pixel_x(hoshis[i][0]);
    int y = display_pixel_y(hoshis[i][1]);
    display->fillCircle(x, y, 2, BLACK);
  }
}

static void display_draw_stones(Inkplate *display, BadukState *baduk_state) {
  for (int row = 0; row < BADUK_BOARD_SIZE; row++) {
    for (int col = 0; col < BADUK_BOARD_SIZE; col++) {
      uint8_t cell = baduk_state->board[row][col];

      if (cell == BADUK_BLACK) {
        // Black stone - filled circle
        display->fillCircle(display_pixel_x(col), display_pixel_y(row),
                            STONE_RADIUS, BLACK);
      } else if (cell == BADUK_WHITE) {
        // White stone - outlined circle
        display->drawCircle(display_pixel_x(col), display_pixel_y(row),
                            STONE_RADIUS, BLACK);
        // Fill with white by drawing a slightly smaller filled white circle
        display->fillCircle(display_pixel_x(col), display_pixel_y(row),
                            STONE_RADIUS - 1, WHITE);
      }

      // Mark the last stone with a small circle (inverted color for visibility)
      if (row == baduk_state->last_row && col == baduk_state->last_col &&
          baduk_state->move_index > 0) {
        int x = display_pixel_x(col);
        int y = display_pixel_y(row);
        int markerRadius = 3;
        // Invert color: WHITE marker on black stones, BLACK marker on white
        // stones
        int markerColor = (cell == BADUK_BLACK) ? WHITE : BLACK;
        display->fillCircle(x, y, markerRadius, markerColor);
      }
    }
  }
}

static void display_draw_game_info(Inkplate *display, BadukState *baduk_state,
                                   uint16_t total_games) {
  display->setTextSize(2);
  display->setTextColor(BLACK);

  // Read metadata from PROGMEM
  GameMetadata metadata;
  memcpy_P(&metadata, &GAMES_METADATA[baduk_state->game_index],
           sizeof(GameMetadata));

  // Line 1: Game number and date
  display->setCursor(100, 10);
  display->print("Game ");
  display->print(baduk_state->game_index + 1);
  display->print("/");
  display->print(total_games);
  display->print(" - ");
  char dateBuffer[11];
  strcpy_P(dateBuffer, metadata.date);
  display->println(dateBuffer);

  // Line 2: Players - combined into one string
  display->setCursor(100, 35);
  char blackBuffer[32];
  strcpy_P(blackBuffer, metadata.black_player);
  char blackRankBuffer[8];
  strcpy_P(blackRankBuffer, metadata.black_rank);
  char whiteBuffer[32];
  strcpy_P(whiteBuffer, metadata.white_player);
  char whiteRankBuffer[8];
  strcpy_P(whiteRankBuffer, metadata.white_rank);

  display->print("B:");
  display->print(blackBuffer);
  display->print(" (");
  display->print(blackRankBuffer);
  display->print(") vs W:");
  display->print(whiteBuffer);
  display->print(" (");
  display->print(whiteRankBuffer);
  display->print(")");

  // Line 3: Move counter
  display->setCursor(100, 60);
  display->print("Move ");
  display->print(baduk_state->move_index);
  display->print("/");
  display->println(baduk_state->move_count);
}
