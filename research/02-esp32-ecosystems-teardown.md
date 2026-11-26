# Source Teardown: ESP32 Protogen Ecosystems (ProtoESP and LumiFur)

Repos audited:
- `research/sources/ProtogenHelmet-ESP32` (`https://github.com/NCPlyn/ProtogenHelmet-ESP32`, commit `15419f5`)
- `research/sources/LumiFur_Controller` (`https://github.com/stef1949/LumiFur_Controller`, commit `6693e35`)

## 1. ProtoESP findings

ProtoESP runs on ESP32-S3 with MAX7219 or WS2812B matrices, ear/cheek LEDs, an internal SSD1306 OLED, an INA219 voltage/current sensor, a PWM fan, a local Wi-Fi AP web dashboard + frame animator, and a separate XIAO ESP32-C3/S3 BLE paw remote.

### Real setup problems documented in `troublefaq.md`
- **PlatformIO friction (lines 5-26):** Spaces in Windows folder paths break builds; setup takes 7 steps in VS Code; users have to run two separate uploads (`Upload` and `Upload Filesystem`). Forgetting the second upload leaves the web UI broken with `"file not found"`.
- **Silent USB failure (line 51):** Single-USB ESP32-S3 boards fail unless `DARDUINO_USB_CDC_ON_BOOT=1` is uncommented manually.
- **Wi-Fi AP annoyance (`ProtoESP-Remote/readme.md`, lines 44-45):** Connecting to `192.168.4.1` requires turning off mobile data and dropping internet access on your phone or laptop.
- **Electrical glitches (lines 61-71):** Thin power wire (under 22 AWG), poor shared grounds, voltage sag, and flaky MAX7219 clones cause glitched or stuck-on LED panels.

**What this means for ProtoStack:** Hosting `ProtoUI` on Cloudflare over WebHID/WebBLE avoids both PlatformIO and Wi-Fi AP dropouts. Adding an active current limiter and INA219 voltage-sag logging directly addresses real hardware failures makers hit every day.

### Useful real-world config fields (`ProtoESP-Controller/data/config.json`, lines 1-33)
- Separate brightness controls for visor (`bVisor`), ears (`bEar`), and internal HUD OLED (`bOled`). Internal OLEDs need to run much dimmer so they do not blind the wearer.
- 3-axis head pose vectors (`neutralX/Y/Z`, `tiltX/Y/Z`, `upX/Y/Z`, `tiltTol`) because every maker mounts the IMU at a slightly different angle. ProtoStack should offer a one-click "Capture Current Pose" button (`CALIBRATE_POSE`) instead of raw X/Y/Z numbers.
- Speech noise-floor tuning (`spMin`, `spMax`, `spTrig`) for loud conventions.

### Real wireless paw remote numbers (`ProtoESP-Remote/readme.md`, lines 8-21)
- Runs on a Seeed XIAO ESP32-C3 or S3 over BLE. Break-apart PCB pads let makers wire buttons into paws.
- Measured draw: **110 mA active** and **15.5 uA in deep sleep** (about 4.5 hours active on a 500 mAh LiPo).
- Holding the BOOT button for 10 seconds factory-resets the remote. This backs up our own physical button hold to trigger Restore Factory Snapshot.

## 2. LumiFur findings

LumiFur targets the Adafruit MatrixPortal ESP32-S3 with dual 64x32 HUB75 panels, NimBLE BLE control, and a GitHub Pages Web Bluetooth OTA updater (`docs/firmware-updater/index.html`).

Good patterns from `README.md` (lines 27-97) worth keeping in mind for later phases:
- **Ambient light auto-brightness (lines 29, 32):** Reuses the APDS9960 boop sensor's light reading to brighten the visor outdoors and dim it in dark rooms with zero extra hardware.
- **Sleep dimming on inactivity (line 32):** Uses the accelerometer to dim the visor when the helmet is sitting on a table.
- **Mouth brightness pop (line 76):** Keeps idle face pixels dimmed while letting active speech pixels scale brighter when talking.
- **Compact video playback (`LFV1`, lines 34, 70):** Converts GIFs/videos into lightweight flash-streamable 64x32 frames.

Even with these nice features, remapping pins or adding custom faces in LumiFur still requires editing `src/hardware/deviceConfig.h` and compiling in PlatformIO (lines 39-40, 68).
