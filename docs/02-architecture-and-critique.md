# ProtoStack: Architecture and Technical Design

Doc ID: PS-ARCH-002
Status: Current (living specification — synced with ADR-001 & ADR-002)

## 1. The four-pillar architecture

We started from your three-part idea (`ProtoUI`, `ProtoIO`, `ProtoBrain`) and ran into one clean structural issue: `ProtoIO` was trying to be both the shared communication protocol (which has to compile for the web browser and every chip) and the chip-specific wiring drivers (which only make sense on a physical microcontroller).

Splitting those into two clear parts gives us four pillars with zero overlap:

- **ProtoUI (`web/`):** The browser configurator (`/`) and builder hardware workbench (`/builder`) built with **TypeScript + Vite + React** and brandless `--ps-*` CSS tokens, backed by our compiled Rust/WASM core (`crates/proto-wasm`). *(Originally explored as two separate Leptos micro-frontends; updated per `ADR-001` for native WebHID ergonomics and contributor accessibility.)*
- **ProtoSpeak (`crates/proto-speak`):** The shared binary protocol for USB configuration, wired/wireless intra-suit sync, and Proto-to-Proto social messages. Pure `no_std` Rust with no hardware dependencies, so it compiles identically for firmware and the browser.
- **ProtoIO (`crates/proto-io` + `crates/proto-bsp-*`):** Physical hardware traits and board-specific drivers (USB WebHID, paw buttons, flash storage, HUB75 panels in `v1.0`; boop sensors, cooling fans, WS2812B strips, and radios in `v1.1+`).
- **ProtoBrain (`crates/proto-brain`):** The core engine that runs expressions, animations, button triggers, and power/thermal safety limits. Because it is also pure `no_std` Rust, it compiles to WASM too, meaning the browser simulator and live PC visor mirror run the exact same Rust engine as the Teensy 4.1.

### Modular attachments via `ProtoParts`
Rather than hardcoding every possible accessory (tail, ears, sensors, props) into the brain and UI, every piece of hardware is modeled as a **`ProtoPart`** descriptor defined in `proto-speak` and tracked in a small fixed-size `PartRegistry` inside `proto-brain`:
- Each `ProtoPart` declares its name, kind (`Visor`, `ButtonBank`, `BoopSensor`, `Fan`, `LightZone`, or `Custom`), the **events** it emits, the **actions** it accepts, and optional **UI controls** (sliders, toggles, color pickers).
- **In `v1.0`:** The 2 core tabletop/visor parts are active (`Visor` and `ButtonBank(0..8)`), keeping RAM and hardware complexity minimal.
- **In `v1.1`:** Helmet comfort/sensor parts (`BoopSensor` and `CoolingFan`) bolt onto the same `ProtoPart` registry without rewrites.
- **Future hardware plugin trail (`v1.2+`):** Because parts are self-describing (similar to USB HID descriptors or Matter device clusters, with no unsafe code upload), a maker can build any custom suit part or accessory (tails, wings, chest panels, body LEDs, or handheld props), connect it over wired `ProtoSpeak` (`v1.2`) or Bluetooth/ESP-NOW (`v1.3`), and have it automatically appear in `ProtoBrain` and `ProtoUI` so you can bind events and actions without modifying the helmet firmware or the website.

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

## 4. Locked `v1.0` scope and future roadmap

We deliberately trimmed `v1.0` to a lean tabletop-first vertical scope (see `ADR-002` for milestone slices `v0.1`→`v1.0`), shipping the best zero-driver browser UX and solid Teensy 4.1 firmware first while deferring helmet-shell sensors and fans to `v1.1` *(initial discovery explored boop + fan in `v1.0`; deferred to keep Phase 1 hardware USB-only and affordable)*:

### Locked `v1.0` Scope ("The Wootility Moment")
- **Hardware (staged bench → visor):**
  - **Phase 1 (`v0.2`–`v0.4`):** USB-powered **Teensy 4.1 + 0–8 breadboard paw buttons** speaking real USB WebHID while `ProtoUI` mirrors the live `128x32` visor on the user's PC screen.
  - **Phase 2 (`v0.5`–`v1.0`):** Physical **dual `64x32` HUB75 P3 (1/16 scan) LED panels** driven by FlexIO + FlexPWM + eDMA (runtime `RGB/RBG/BGR` color order, FM6126A latch timing, `PANEL_IDENTIFY`, supporting `2× SN74HCT245N` breadboard wiring as well as SmartLED V5 / Coela presets).
- **Inputs:** `0` to `8` wired paw/chin buttons, where every button supports **both Tap and Hold** independently in `v1.0` (boop sensor, multi-button Combos, and Double-Tap arrive in `v1.1`).
- **Power safety:** Software LED current-budget brightness governor (computed live on `ProtoBrain`, zero extra hardware required).
- **Profiles:** Single active profile in `v1.0` (multi-profile slots arrive in `v1.1`).
- **Web UI (`web/`, single TypeScript + Vite + React Cloudflare app wrapping Rust/WASM, no desktop app):**
  - **`/` Config route (Owners):** Wootility v5-style layout: zero-driver WebHID connect, live WASM visor simulator / PC visor mirror, starter faces + PNG/GIF animation import (`64x32` / `128x32` specs, no pixel painter), live color tint & brightness, N-button **Tap + Hold** mapper, inline custom naming (helmet, faces, buttons), current-budget limiter, RAM preview vs atomic A/B flash save, and Restore Factory Snapshot.
  - **`/builder` Builder route (Makers & Studios):** Visually distinct **Hardware Workbench layout** (interactive Teensy 4.1 PCB pinout/wiring canvas + hardware inspector, not a recolored clone of Config) with clean route boundaries so it can also deploy to a separate subdomain later if desired. Includes in-browser WebHID Teensy 4.1 HalfKay firmware flasher, board/pinout presets, click-to-assign custom pins, Panel 1/2 orientation test (`PANEL_IDENTIFY`), fault log viewer, and Save Factory Snapshot.
- **Protocol:** `ProtoSpeak` Domain 0 (USB WebHID) active; 12-byte header already reserves `Domain` and `NodeId` fields for future wireless/mesh phases.

### Future Phases (Clean Trails Preserved)
- **`v1.1` (Helmet Sensors, Comfort & More Hardware):** Boop sensor (APDS-9960 I2C with continuous `live_cm <= threshold_cm` + `1.5 cm` hysteresis, plus digital-pin mode), 5V PWM cooling fan control, I2C `SENSOR_SCAN`, ESP32-S3 board support, WS2812B cheek/ear LEDs, MAX7219 panels, multi-profile slots, multi-button **Combos** (simultaneous presses) and **Double-Tap** button gestures, and optional mic/jaw lip-sync.
- **`v1.2` (Wired Modular Suit):** Turn on `ProtoSpeak` Domain 1 over a physical wire first (UART / CAN / RS-485). This proves real plug-and-play modularity before touching RF: plug a wired tail, ears, chestplate, or external sensor node into the head bus and it self-registers, shows up in `ProtoUI`, and reacts to boops and face changes.
- **`v1.3` (Wireless Modular Suit and Autonomy):** Reuse the already-proven `v1.2` modular node system over wireless PHYs (ESP-NOW, BLE, nRF24/LoRa), adding encrypted `SuitPSKey` pairing and the 3-second standalone fallback so a tail keeps running if you leave the head in another room.
- **`v2.0` (Social Mesh and Ecosystem):** Opt-in Proto-to-Proto social beacons, procedural vector faces, optional in-browser pixel editor, and Linux SBCs (Raspberry Pi / OrangePi).
