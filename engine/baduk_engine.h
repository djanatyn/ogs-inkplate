#ifndef BADUK_ENGINE_H
#define BADUK_ENGINE_H

#include "baduk_types.h"

void baduk_reset(BadukState* state);
uint8_t baduk_load_game(BadukState* state, const BadukGameRecord* games,
                        uint16_t game_count, uint16_t game_index);
uint8_t baduk_play_next_move(BadukState* state, const BadukGameRecord* games);
uint8_t baduk_play_move(BadukState* state, uint16_t encoded_move,
                        uint8_t color);

#endif
