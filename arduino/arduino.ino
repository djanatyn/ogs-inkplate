/*
   Show OGS game records on Inkplate 4 TEMPERE e-ink display.
*/

#include "Inkplate.h"
#include "config.h"
#include "DisplayManager.h"
#include "games_data.h"
#include "../logic/baduk_engine.h"
#include <mbedtls/base64.h>

Inkplate display(INKPLATE_1BIT);
BadukState badukState;
unsigned long lastMoveUpdate = 0;
bool gameActive = true;
uint16_t gameOrder[GAME_COUNT];
uint16_t gameOrderIndex = 0;

DisplayManager displayManager(display);

void shuffle_games() {
    uint16_t i;

    for (i = 0; i < GAME_COUNT; i++) {
        gameOrder[i] = i;
    }

    if (!RANDOM_GAME_ORDER) {
        return;
    }

    for (i = GAME_COUNT - 1; i > 0; i--) {
        uint16_t j = random(i + 1);
        uint16_t tmp = gameOrder[i];

        gameOrder[i] = gameOrder[j];
        gameOrder[j] = tmp;
    }
}

void load_ordered_game(uint16_t orderIndex) {
    uint16_t gameIndex = gameOrder[orderIndex];

    baduk_load_game(&badukState, GAMES, GAME_COUNT, gameIndex);
}

void write_base64(const uint8_t* data, size_t dataLen) {
    const size_t inputChunkSize = 45;
    uint8_t encoded[65];
    size_t offset;

    for (offset = 0; offset < dataLen; offset += inputChunkSize) {
        size_t chunkLen = dataLen - offset;
        size_t encodedLen = 0;
        int result;

        if (chunkLen > inputChunkSize) {
            chunkLen = inputChunkSize;
        }

        result = mbedtls_base64_encode(
            encoded,
            sizeof(encoded),
            &encodedLen,
            data + offset,
            chunkLen
        );

        if (result != 0) {
            Serial.println("BASE64_ERROR");
            return;
        }

        encoded[encodedLen] = '\0';
        Serial.println((char*)encoded);
    }
}

void dump_screenshot() {
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

    write_base64(display._partial, size);

    Serial.println("DATA_END");
    Serial.println("SCREENSHOT_END");
    Serial.flush();
}

void check_serial_commands() {
    int ch;

    if (!Serial.available()) {
        return;
    }

    ch = Serial.read();

    if (ch == 's' || ch == 'S') {
        dump_screenshot();
    }
}

void setup() {
    Serial.begin(115200);
    display.begin();
    display.setTextColor(BLACK);

    // initialize frontlight
    display.frontlight.setState(true);
    display.frontlight.setBrightness(INITIAL_FRONTLIGHT);
    Serial.print("frontlight enabled: ");
    Serial.println(INITIAL_FRONTLIGHT);

    Serial.println("starting baduk game viewer...");

    // initialize state, load first game
    randomSeed(esp_random());
    shuffle_games();
    gameOrderIndex = 0;
    load_ordered_game(gameOrderIndex);
    lastMoveUpdate = millis();

    Serial.print("total games: ");
    Serial.println(GAME_COUNT);

    // initial board
    displayManager.drawAll(&badukState, GAME_COUNT);
    display.display();
}

void loop() {
    unsigned long currentMillis = millis();

    check_serial_commands();

    if (gameActive && currentMillis - lastMoveUpdate >= MOVE_INTERVAL) {
        if (!baduk_play_next_move(&badukState, GAMES)) {
            Serial.print("invalid move in game ");
            Serial.print(badukState.game_index);
            Serial.print(" at move ");
            Serial.println(badukState.move_index + 1);
            gameActive = false;
        }

        // is game over?
        if (badukState.move_index >= badukState.move_count) {
            Serial.println("game finished, moving to next game...");
            delay(5000);  // Pause before next game

            gameOrderIndex++;
            if (gameOrderIndex >= GAME_COUNT) {
                if (LOOP_GAMES) {
                    shuffle_games();
                    gameOrderIndex = 0;
                } else {
                    gameActive = false;
                }
            }

            if (gameActive) {
                load_ordered_game(gameOrderIndex);
            }
        }

        // redraw board with new move
        displayManager.drawAll(&badukState, GAME_COUNT);

        lastMoveUpdate = currentMillis;
    }
    delay(100);
}
