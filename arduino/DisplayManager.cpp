#include "DisplayManager.h"

DisplayManager::DisplayManager(Inkplate& disp) : display(disp), partialUpdateCount(0) {
}

int DisplayManager::pixelX(int boardCol) {
    return BOARD_OFFSET_X + (boardCol * GRID_SPACING);
}

int DisplayManager::pixelY(int boardRow) {
    return BOARD_OFFSET_Y + (boardRow * GRID_SPACING);
}

void DisplayManager::drawAll(BadukState* state, uint16_t totalGames) {
    display.clearDisplay();
    drawBoard();
    drawStones(state);
    drawGameInfo(state, totalGames);

    partialUpdateCount++;
    if (partialUpdateCount >= FULL_REFRESH_EVERY) {
        display.display();
        partialUpdateCount = 0;
    } else {
        display.partialUpdate();
    }
}

void DisplayManager::drawBoard() {
    display.setTextSize(1);
    display.setTextColor(BLACK);

    // Draw grid lines
    drawGridLines();

    // Draw hoshi (star) points
    drawHoshiPoints();

    // Draw board edge
    display.drawRect(pixelX(0), pixelY(0), GRID_SPACING * (BADUK_BOARD_SIZE - 1),
                     GRID_SPACING * (BADUK_BOARD_SIZE - 1), BLACK);
}

void DisplayManager::drawGridLines() {
    // Draw vertical lines
    for (int col = 0; col < BADUK_BOARD_SIZE; col++) {
        int x = pixelX(col);
        display.drawLine(x, pixelY(0), x, pixelY(BADUK_BOARD_SIZE - 1), BLACK);
    }

    // Draw horizontal lines
    for (int row = 0; row < BADUK_BOARD_SIZE; row++) {
        int y = pixelY(row);
        display.drawLine(pixelX(0), y, pixelX(BADUK_BOARD_SIZE - 1), y, BLACK);
    }
}

void DisplayManager::drawHoshiPoints() {
    // Hoshi (star) points on a 19x19 board
    int hoshis[9][2] = {
        {3, 3}, {3, 9}, {3, 15}, {9, 3}, {9, 9}, {9, 15}, {15, 3}, {15, 9}, {15, 15}
    };

    for (int i = 0; i < 9; i++) {
        int x = pixelX(hoshis[i][0]);
        int y = pixelY(hoshis[i][1]);
        display.fillCircle(x, y, 2, BLACK);
    }
}

void DisplayManager::drawStones(BadukState* state) {
    for (int row = 0; row < BADUK_BOARD_SIZE; row++) {
        for (int col = 0; col < BADUK_BOARD_SIZE; col++) {
            uint8_t cell = state->board[row][col];

            if (cell == BADUK_BLACK) {
                // Black stone - filled circle
                display.fillCircle(pixelX(col), pixelY(row), STONE_RADIUS, BLACK);
            } else if (cell == BADUK_WHITE) {
                // White stone - outlined circle
                display.drawCircle(pixelX(col), pixelY(row), STONE_RADIUS, BLACK);
                // Fill with white by drawing a slightly smaller filled white circle
                display.fillCircle(pixelX(col), pixelY(row), STONE_RADIUS - 1, WHITE);
            }

            // Mark the last stone with a small circle (inverted color for visibility)
            if (row == state->last_row && col == state->last_col && state->move_index > 0) {
                int x = pixelX(col);
                int y = pixelY(row);
                int markerRadius = 3;
                // Invert color: WHITE marker on black stones, BLACK marker on white stones
                int markerColor = (cell == BADUK_BLACK) ? WHITE : BLACK;
                display.fillCircle(x, y, markerRadius, markerColor);
            }
        }
    }
}

void DisplayManager::drawGameInfo(BadukState* state, uint16_t totalGames) {
    display.setTextSize(2);
    display.setTextColor(BLACK);

    // Read metadata from PROGMEM
    GameMetadata metadata;
    memcpy_P(&metadata, &GAMES_METADATA[state->game_index], sizeof(GameMetadata));

    // Line 1: Game number and date
    display.setCursor(100, 10);
    display.print("Game ");
    display.print(state->game_index + 1);
    display.print("/");
    display.print(totalGames);
    display.print(" - ");
    char dateBuffer[11];
    strcpy_P(dateBuffer, metadata.date);
    display.println(dateBuffer);

    // Line 2: Players - combined into one string
    display.setCursor(100, 35);
    char blackBuffer[32];
    strcpy_P(blackBuffer, metadata.black_player);
    char blackRankBuffer[8];
    strcpy_P(blackRankBuffer, metadata.black_rank);
    char whiteBuffer[32];
    strcpy_P(whiteBuffer, metadata.white_player);
    char whiteRankBuffer[8];
    strcpy_P(whiteRankBuffer, metadata.white_rank);

    display.print("B:");
    display.print(blackBuffer);
    display.print(" (");
    display.print(blackRankBuffer);
    display.print(") vs W:");
    display.print(whiteBuffer);
    display.print(" (");
    display.print(whiteRankBuffer);
    display.print(")");

    // Line 3: Move counter
    display.setCursor(100, 60);
    display.print("Move ");
    display.print(state->move_index);
    display.print("/");
    display.println(state->move_count);
}
