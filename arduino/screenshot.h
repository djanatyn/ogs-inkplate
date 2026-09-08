#ifndef SCREENSHOT_H
#define SCREENSHOT_H

#include <Arduino.h>
#include <Inkplate.h>

void screenshot_check_serial(Inkplate *display);
void screenshot_dump(Inkplate *display);

#endif
