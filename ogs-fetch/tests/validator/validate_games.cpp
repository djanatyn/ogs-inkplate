#include <stdio.h>
#include "baduk_engine.h"
#include "games_data.h"

static void print_board(BadukState* state) {
    uint8_t row;
    uint8_t col;

    for (row = 0; row < BADUK_BOARD_SIZE; row++) {
        for (col = 0; col < BADUK_BOARD_SIZE; col++) {
            printf("%u", state->board[row][col]);
        }
    }
}

int main(void) {
    BadukState state;
    uint16_t game_index;

    for (game_index = 0; game_index < GAME_COUNT; game_index++) {
        if (!baduk_load_game(&state, GAMES, GAME_COUNT, game_index)) {
            printf("load_error %u\n", game_index);
            return 1;
        }

        printf("game %u moves %u\n", game_index, state.move_count);

        while (state.move_index < state.move_count) {
            if (!baduk_play_next_move(&state, GAMES)) {
                printf("move_error %u %u\n", game_index, state.move_index + 1);
                return 1;
            }

            printf("move %u ", state.move_index);
            print_board(&state);
            printf("\n");
        }
    }

    return 0;
}
