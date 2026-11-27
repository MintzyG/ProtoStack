# ProtoStack: Architecture and Technical Design

Doc ID: PS-ARCH-002
Status: Current (synced with ADR-001 and ADR-002)

## 1. The four-pillar architecture

We started from your three-part idea (`ProtoUI`, `ProtoIO`, `ProtoBrain`) and ran into one structural issue: `ProtoIO` was trying to be both the shared communication protocol (which has to compile for the web browser and every chip) and the chip-specific wiring drivers (which only make sense on a physical microcontroller).

Splitting those into two parts gives us four pillars with no overlap:

- **ProtoUI (`web/`):** The browser configurator (`/`) and builder workbench (`/builder`) written in TypeScript + Vite + React with neutral `--ps-*` CSS tokens, backed by our compiled Rust/WASM core (`crates/proto-wasm`). (Earlier drafts considered two Leptos micro-frontends; changed in `ADR-001` for better WebHID support and faster UI work.)
- **ProtoSpeak (`crates/proto-speak`):** The shared binary protocol for USB configuration, wired/wireless intra-suit sync, and Proto-to-Proto social messages. Pure `no_std` Rust with no hardware dependencies, so it compiles the same for firmware and the browser.
- **ProtoIO (`crates/proto-io` + `crates/proto-bsp-*`):** Physical hardware traits and board-specific drivers (USB WebHID, paw buttons, flash storage, HUB75 panels in v1.0; boop sensors, cooling fans, WS2812B strips, and radios in v1.1+).
- **ProtoBrain (`crates/proto-brain`):** The core engine that runs expressions, animations, button triggers, and power limits. Because it is pure `no_std` Rust, it compiles to WASM too, so the browser simulator and PC visor mirror run the exact same Rust engine as the Teensy 4.1.

### Modular attachments via `ProtoParts`
Rather than hardcoding every possible accessory (tail, ears, sensors, props) into the brain and UI, every piece of hardware is modeled as a `ProtoPart` descriptor defined in `proto-speak` and tracked in a small fixed-size `PartRegistry` inside `proto-brain`:
- Each `ProtoPart` declares its name, kind (`Visor`, `ButtonBank`, `BoopSensor`, `Fan`, `LightZone`, or `Custom`), the events it emits, the actions it accepts, and optional UI controls (sliders, toggles, color pickers).
- **In v1.0:** Two parts are active (`Visor` and `ButtonBank(0..8)`), keeping RAM and wiring simple.
- **In v1.1:** Helmet add-ons (`BoopSensor` and `CoolingFan`) plug into the same `ProtoPart` registry.
- **Future hardware plugin trail (v1.2+):** Because parts describe themselves (similar to USB HID descriptors or Matter device clusters, with no unsafe code upload), a maker can build any custom suit part or accessory (tails, wings, chest panels, body LEDs, or handheld props), connect it over wired `ProtoSpeak` (v1.2) or Bluetooth/ESP-NOW (v1.3), and have it appear in `ProtoBrain` and `ProtoUI` without editing the helmet firmware or the website.

## 2. Planned workspace layout

```text
protostack/
  Cargo.toml
  crates/
    proto-speak/          # [no_std + wasm] Packet framing, opcodes, profile schema, mesh packets
    proto-brain/          # [no_std + wasm] Frame compositor, trigger rules, safety governor
    proto-io/             # [no_std] Hardware traits: VisorDisplay, ButtonBank, FlashStore (+ v1.1 SensorBus, FanPwm)
    proto-wasm/           # [wasm-bindgen] Thin WASM bridge exposing ProtoBrain + ProtoSpeak to web/
    proto-bsp-teensy41/   # [ARM Cortex-M7] Teensy 4.1 drivers: imxrt-hal, imxrt-usbd WebHID, GPIO, Flash, FlexIO HUB75
    proto-bsp-esp32s3/    # [Future v1.1+] ESP32-S3 drivers: esp-hal, native USB HID, ESP-NOW / BLE
    proto-bsp-linux/      # [Future v2.0+] Raspberry Pi / OrangePi backend
  web/                    # TypeScript + Vite + React ProtoUI app (`/` Config + `/builder` Workbench)
    src/
      tokens.css          # Brandless --ps-* CSS tokens
      transport/          # ProtoSpeak transport interface (Loopback Simulator + Browser WebHID + HalfKay flasher)
      routes/             # Distinct Config (/) and Builder (/builder) layouts
  boards/
    teensy41-breadboard.toml
    teensy41-smartled-v5.toml
    teensy41-coela-delta.toml
```

## 3. Key technical decisions

### Driverless USB over WebHID (and browser flashing)
To match Wootility's plug-and-play feel, the firmware will expose a custom USB HID interface (Vendor Usage Page `0xFF60`, Usage `0x61`, matching the proven VIA pattern) using `imxrt-usbd` and `usb-device` in pure Rust.

Why WebHID instead of WebUSB or WebSerial:
- Windows, macOS, and ChromeOS load an inbox HID driver automatically. Users never need Zadig or custom drivers.
- WebSerial forces users to guess between `COM3` and `COM7`. WebHID lets the browser show the device name directly (for example, "ProtoStack Visor").
- WebHID works in Chrome, Edge, Brave, Opera, and Arc. For Firefox or Safari users, the site still works in Simulator Mode and tells them cleanly that USB connection requires a Chromium browser.

One big finding from auditing Coela's ProtoTracer source code (`research/01-prototracer-teardown.md`): **Teensy 4.0 and 4.1 HalfKay bootloaders can also be flashed directly from Chrome/Edge over WebHID** (`VID 0x16C0`, `PID 0x0478/0479`, 1088-byte HID reports). If we add a `BOOTLOADER_JUMP` command (the same trick VIA uses, triggering Teensy's `bkpt #251`), owners can update Teensy 4.1 firmware straight from the Cloudflare website without opening their helmet to press the physical program button.

### Display pipeline on Teensy 4.1
- `ProtoBrain` renders into a logical framebuffer (such as 128x32 for two chained 64x32 HUB75 panels) plus any auxiliary LED zones (cheeks, ears).
- On Teensy 4.1, `proto-bsp-teensy41` streams the front buffer to HUB75 using the i.MX RT1062 FlexIO, FlexPWM, and eDMA peripherals while `ProtoBrain` prepares the next frame in the back buffer.
- Hardware quirks we found in SmartMatrix/ProtoTracer (`research/01-prototracer-teardown.md`) - specifically FM6126A latch timing (100 ns), 18.462 MHz clock divider (26), and RGB vs RBG color channel swaps - will be runtime settings in the hardware manifest rather than compile-time C++ macros.

### Safety without studio lockouts
Since builders should never be able to lock an owner out of their own helmet:
- **Preview in RAM first:** Edits from the website update RAM immediately so you can see changes live without touching flash. If something looks wrong, unplugging or resetting restores the saved state.
- **Factory Snapshot:** A builder or owner can save a known-good snapshot in flash. Holding a hidden button at boot (or clicking Restore Factory in the web app) reverts cleanly to that snapshot.
- **Current and thermal guardrails:** `ProtoBrain` estimates total LED current draw against the configured power bank limit (for example 5V / 3A) and gently scales down peak brightness to prevent brownout reboots. Fan curves warn before letting you drop cooling to 0%, but never lock you out.

## 4. Current v1.0 scope and future roadmap

We scoped v1.0 around a desk-first workflow (see `ADR-002` for milestones v0.1 through v1.0). Helmet-shell sensors and fans were discussed in early discovery and moved to v1.1 so Phase 1 hardware stays USB-only and inexpensive:

### v1.0 Scope
- **Hardware (two phases):**
  - **Phase 1 (v0.2 - v0.4):** USB-powered Teensy 4.1 + 0-8 breadboard paw buttons speaking real USB WebHID while `ProtoUI` renders the live `128x32` visor on your computer monitor.
  - **Phase 2 (v0.5 - v1.0):** Physical dual `64x32` HUB75 P3 (1/16 scan) LED panels driven by FlexIO + FlexPWM + eDMA (runtime `RGB/RBG/BGR` order, FM6126A latch timing, `PANEL_IDENTIFY`, supporting `2x SN74HCT245N` breadboard wiring as well as SmartLED V5 / Coela presets).
- **Inputs:** 0 to 8 wired paw/chin buttons, each supporting independent Tap and Hold actions (boop sensor, multi-button Combos, and Double-Tap move to v1.1).
- **Power safety:** Software LED current-budget brightness limiter computed in `ProtoBrain` (no extra hardware required).
- **Profiles:** Single active profile in v1.0 (multi-profile slots move to v1.1).
- **Web UI (`web/`, single TypeScript + Vite + React app wrapping Rust/WASM, no desktop app):**
  - **`/` Config route (Owners):** Wootility-style layout with WebHID connect, live WASM visor simulator / PC mirror, starter faces + PNG/GIF animation import (`64x32` / `128x32`, no pixel painter), live color tint and brightness, N-button Tap + Hold mapper, inline renaming (helmet, faces, buttons), current-budget limiter, RAM preview vs atomic A/B flash save, and Restore Factory Snapshot.
  - **`/builder` Builder route (Makers & Studios):** Separate Hardware Workbench layout (interactive Teensy 4.1 PCB pinout canvas + hardware inspector) with WebHID HalfKay firmware flasher, board/pinout presets, click-to-assign custom pins, Panel 1/2 orientation test (`PANEL_IDENTIFY`), fault log viewer, and Save Factory Snapshot.
- **Protocol:** `ProtoSpeak` Domain 0 (USB WebHID) active; 12-byte header reserves `Domain` and `NodeId` fields for future wired/wireless phases.

### Future Phases
- **v1.1 (Helmet sensors, comfort, and more boards):** Boop sensor (APDS-9960 I2C with 1.5 cm hysteresis, plus digital pin mode), 5V PWM cooling fan control, I2C `SENSOR_SCAN`, ESP32-S3 board support, WS2812B cheek/ear LEDs, MAX7219 panels, multi-profile slots, multi-button Combos and Double-Tap gestures, and optional mic/jaw lip-sync.
- **v1.2 (Wired modular suit):** Enable `ProtoSpeak` Domain 1 over a physical wire (UART / CAN / RS-485) so wired tails, ears, chestplates, or sensor pods self-register and appear in `ProtoUI`.
- **v1.3 (Wireless modular suit and autonomy):** Run the same Domain 1 messages over wireless PHYs (ESP-NOW, BLE, nRF24/LoRa) with encrypted `SuitPSKey` pairing and a 3-second standalone fallback when the head is out of range.
- **v2.0 (Social mesh and ecosystem):** Opt-in Proto-to-Proto social beacons, procedural vector faces, optional in-browser pixel editor, and Linux SBCs (Raspberry Pi / OrangePi).
