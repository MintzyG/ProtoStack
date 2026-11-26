# Source Teardown: Coela's ProtoTracer

Repo: `research/sources/ProtoTracer` (`https://github.com/coelacant1/ProtoTracer`)
Commit: `15e8c3d`
Stack: C++ / Arduino / PlatformIO targeting Teensy 4.0 and 4.1

## 1. How ProtoTracer is built

ProtoTracer renders real-time 3D `.OBJ` and `.FBX` blendshape faces (like `NukudeFlat`) onto dual HUB75 panels or WS35 LED arrays using Pixelmatix's `SmartMatrix` library. It proves the Teensy 4.1 has plenty of horsepower for smooth Protogen visors.

The downside is configuration friction. Everything is decided at compile time:
- `platformio.ini` (lines 30-131) maintains 10 separate build targets (`teensy40hub75`, `teensy41hub75`, `teensy41ws35`, `teensy41verifyhardware`, etc.).
- `UserConfiguration.h` (lines 19-62) uses C++ `#define` toggles for panel color order (`HUB75_RBG`), menu hardware (`NEOTRELLISMENU`), and button input (`MORSEBUTTON`).
- `ProtogenHUB75Project.h` (lines 17-169) hardcodes every face name, color, morph weight, boop override, and button mapping inside C++ methods and `switch` statements.

For a non-technical owner, changing one color or boop face requires installing VS Code, PlatformIO, editing C++, and reflashing.

## 2. Key finding: Teensy 4.x can be flashed straight from the browser via WebHID

Inspecting `.docs/prototracer-firmware-uploader.js` proved something important: Teensy 4.0 and 4.1 HalfKay bootloaders work directly over WebHID in Chrome/Edge with no drivers or Teensyduino installed.

Verified details from `prototracer-firmware-uploader.js`:
- **Device filters (lines 153-158):** Teensy 4.0 uses `VID 0x16C0` (5824) / `PID 0x0478` (1144); Teensy 4.1 uses `VID 0x16C0` / `PID 0x0479` (1145).
- **HEX parsing (lines 177-343):** Subtracts the Teensy 4.x flash base offset `0x60000000` and packs records into 1024-byte pages padded with `0xFF`.
- **HID write protocol (lines 414-477):** Sends 1088-byte HID Output Reports (`sendReport(0, report)`):
  - Bytes 0-2: 24-bit little-endian flash address.
  - Bytes 3-63: zero padding.
  - Bytes 64-1087: 1024-byte flash page.
  - Skips blank `0xFF` blocks except block 0. Waits 1500 ms after block 0 (flash erase), then 5 ms per block.
  - Sends `[0xFF, 0xFF, 0xFF]` at the end to reboot into the new firmware.

**Impact on ProtoStack:** We can build a pure-Rust/WASM HalfKay flasher right into the Cloudflare website. Paired with a `BOOTLOADER_JUMP` command in `ProtoSpeak`, users can flash and update Teensy 4.1 firmware completely in the browser without opening their helmet to hit the tiny program button.

## 3. Verified HUB75 timing and SmartLED Shield V5 pins

From `lib/ProtoTracer/Controller/MatrixHardware_Teensy4_ShieldV5_RBG.h`:

- **FlexIO pixel clock (line 62):** Divider `26` on the 480 MHz PLL3 gives **18.462 MHz** (more stable than 24 MHz / divider 20 on cheap panels).
- **Latch pulse width (line 49):** `100 ns` (works with both DP5020B and FM6126A panels; stay under 150 ns).
- **Latch-to-clock delay (line 55):** `400 ns` safe DMA margin.
- **Color order swaps (lines 71-76):** Some HUB75 panels swap green and blue (RBG). ProtoStack should make color order (`RGB`, `RBG`, `BGR`) and panel driver (`Standard`, `FM6126A`) runtime settings in the manifest instead of C++ header edits.
- **SmartLED Shield V5 pinout (lines 88-102):** `OE=2`, `LATCH=3`, `CLK=7` (alt `8`), `R0=6`, `G0=9`, `B0=10`, `R1=12`, `G1=11`, `B1=13`, `APA_CLK=4`, `APA_DAT=5`, `APA_EN=22`. We should include this as a built-in board preset in `boards/`.

## 4. Good ideas worth keeping

- **Hardware self-test (`src/main.cpp`, lines 57-62):** ProtoTracer has a build target that scans I2C devices and tests boop, NeoTrellis, and HUD. In ProtoStack, we can expose this live over `ProtoSpeak` (`SENSOR_SCAN` + `PANEL_IDENTIFY`) without flashing a separate test binary.
- **Common community sensors (`platformio.ini`, lines 17-28):** Confirms we should prioritize I2C drivers in `ProtoIO` for APDS9960/APDS9999 (boop), BNO055 (IMU), SSD1306 (internal HUD OLED), and Seesaw/NeoTrellis buttons.
