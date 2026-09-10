#ifndef BADUK_DISPLAY_H
#define BADUK_DISPLAY_H

#include <Arduino.h>
#include <Inkplate.h>

#include "../engine/baduk_engine.h"
#include "config.h"
#include "games_metadata.h"

struct DisplayState {
  uint16_t partial_update_count;
};

void display_init(DisplayState* display_state);
void display_draw_all(Inkplate* display, DisplayState* display_state,
                      BadukState* baduk_state, uint16_t total_games);

#endif
