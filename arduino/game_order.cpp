#include "game_order.h"

static void game_order_shuffle(GameOrder *order) {
  uint16_t i;

  for (i = 0; i < GAME_COUNT; i++) {
    order->games[i] = i;
  }

  if (!RANDOM_GAME_ORDER) {
    return;
  }

  for (i = GAME_COUNT - 1; i > 0; i--) {
    uint16_t j = random(i + 1);
    uint16_t tmp = order->games[i];

    order->games[i] = order->games[j];
    order->games[j] = tmp;
  }
}

void game_order_init(GameOrder *order) {
  randomSeed(esp_random());
  order->index = 0;
  game_order_shuffle(order);
}

uint16_t game_order_current(GameOrder *order) {
  return order->games[order->index];
}

uint8_t game_order_advance(GameOrder *order) {
  order->index++;

  if (order->index < GAME_COUNT) {
    return 1;
  }

  if (!LOOP_GAMES) {
    return 0;
  }

  order->index = 0;
  game_order_shuffle(order);

  return 1;
}
