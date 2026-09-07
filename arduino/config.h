#ifndef CONFIG_H
#define CONFIG_H

#include "../engine/baduk_types.h"

// Display Configuration
#define MOVE_INTERVAL 8000               // 8 seconds per move (in milliseconds)
#define FULL_REFRESH_EVERY 20            // Full refresh every 20 partial updates

// Board Layout (600x600 display)
// Actual board is GRID_SPACING * (BADUK_BOARD_SIZE - 1) = 26 * 18 = 468px
#define GRID_SPACING 26                  // Space between grid lines (px)
#define ACTUAL_BOARD_SIZE (GRID_SPACING * (BADUK_BOARD_SIZE - 1))  // 468px
#define HEADER_HEIGHT 40                 // Space for game/move text at top
#define BOARD_OFFSET_X ((600 - ACTUAL_BOARD_SIZE) / 2)  // Center horizontally = 66px
#define BOARD_OFFSET_Y ((600 - HEADER_HEIGHT - ACTUAL_BOARD_SIZE) / 2 + HEADER_HEIGHT)  // Center vertically with header margin
#define BOARD_WIDTH ACTUAL_BOARD_SIZE    // Will be 468px
#define BOARD_HEIGHT ACTUAL_BOARD_SIZE   // Will be 468px
#define STONE_RADIUS 11                  // Stone radius (slightly smaller than spacing)

// Game Playback
#define RANDOM_GAME_ORDER true           // Shuffle games or play in order
#define LOOP_GAMES true                  // Restart from beginning after all games played

// Frontlight Configuration
// The Inkplate 4 TEMPERA has a frontlight that can be adjusted from 0-63
// 0 = minimum brightness (off)
// 63 = maximum brightness (very bright)
// Recommended range: 40-50 for comfortable viewing
#define INITIAL_FRONTLIGHT 5            // Initial frontlight level (0-63)

#endif
