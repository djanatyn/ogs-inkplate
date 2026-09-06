#!/usr/bin/env bash
set -xeuo pipefail

BOARD_URL="https://github.com/SolderedElectronics/Inkplate-Board-Definitions-for-Arduino-IDE/raw/refs/heads/main/package_Inkplate_Boards_index.json"
INKPLATE_CORE="soldered-inkplate-boards:esp32@3.0.0"

if ! command -v arduino-cli >/dev/null 2>&1; then
  echo "arduino-cli is required but was not found on PATH."
  echo "Install it from https://arduino.github.io/arduino-cli/latest/installation/"
  exit 1
fi

arduino-cli core update-index --additional-urls "$BOARD_URL"

arduino-cli core install \
  --additional-urls "$BOARD_URL" \
  "$INKPLATE_CORE"

arduino-cli lib install InkplateLibrary@11.1.2

echo "Setup complete."
echo "Compile with: arduino-cli compile --fqbn soldered-inkplate-boards:esp32:Inkplate4TEMPERA:UploadSpeed=115200 arduino"
