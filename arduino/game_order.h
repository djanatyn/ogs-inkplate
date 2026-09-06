#ifndef GAME_ORDER_H
#define GAME_ORDER_H

#include "config.h"
#include "games_data.h"
#include <Arduino.h>

struct GameOrder {
    uint16_t games[GAME_COUNT];
    uint16_t index;
};

void game_order_init(GameOrder* order);
uint16_t game_order_current(GameOrder* order);
uint8_t game_order_advance(GameOrder* order);

#endif
