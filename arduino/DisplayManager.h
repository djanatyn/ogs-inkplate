#ifndef DISPLAY_MANAGER_H
#define DISPLAY_MANAGER_H

#include "config.h"
#include "games_metadata.h"
#include "../logic/baduk_engine.h"
#include <Arduino.h>
#include <Inkplate.h>

class DisplayManager {
private:
    Inkplate& display;
    uint16_t partialUpdateCount;

public:
    DisplayManager(Inkplate& disp);

    void drawAll(BadukState* state, uint16_t totalGames);
    void drawBoard();
    void drawStones(BadukState* state);
    void drawGameInfo(BadukState* state, uint16_t totalGames);
    void drawGridLines();
    void drawHoshiPoints();
    int pixelX(int boardCol);
    int pixelY(int boardRow);
};

#endif
