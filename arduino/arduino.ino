/*
   Show OGS game records on Inkplate 4 TEMPERA e-ink display.
*/

#include "Inkplate.h"
#include "config.h"
#include "display.h"
#include "game_order.h"
#include "games_data.h"
#include "screenshot.h"
#include "../logic/baduk_engine.h"

Inkplate display(INKPLATE_1BIT);
BadukState baduk_state;
DisplayState display_state;
GameOrder game_order;
unsigned long last_move_update = 0;
bool game_active = true;

void load_current_game() {
    baduk_load_game(&baduk_state, GAMES, GAME_COUNT, game_order_current(&game_order));
}

void setup() {
    Serial.begin(115200);
    display.begin();
    display_init(&display_state);
    display.setTextColor(BLACK);

    // initialize frontlight
    display.frontlight.setState(true);
    display.frontlight.setBrightness(INITIAL_FRONTLIGHT);
    Serial.print("frontlight enabled: ");
    Serial.println(INITIAL_FRONTLIGHT);

    Serial.println("starting baduk game viewer...");

    // initialize state, load first game
    game_order_init(&game_order);
    load_current_game();
    last_move_update = millis();

    Serial.print("total games: ");
    Serial.println(GAME_COUNT);

    // initial board
    display_draw_all(&display, &display_state, &baduk_state, GAME_COUNT);
}

void loop() {
    unsigned long currentMillis = millis();

    screenshot_check_serial(&display);

    if (game_active && currentMillis - last_move_update >= MOVE_INTERVAL) {
        if (!baduk_play_next_move(&baduk_state, GAMES)) {
            Serial.print("invalid move in game ");
            Serial.print(baduk_state.game_index);
            Serial.print(" at move ");
            Serial.println(baduk_state.move_index + 1);
            game_active = false;
        }

        // is game over?
        if (baduk_state.move_index >= baduk_state.move_count) {
            Serial.println("game finished, moving to next game...");
            delay(5000);  // Pause before next game

            if (game_order_advance(&game_order)) {
                load_current_game();
            } else {
                game_active = false;
            }
        }

        // redraw board with new move
        display_draw_all(&display, &display_state, &baduk_state, GAME_COUNT);

        last_move_update = currentMillis;
    }
    delay(100);
}
