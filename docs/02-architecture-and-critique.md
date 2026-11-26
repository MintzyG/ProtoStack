# ProtoStack: Architecture and Technical Design

Doc ID: PS-ARCH-002
Status: Draft for client review

## 1. The four-pillar architecture

We started from your three-part idea (`ProtoUI`, `ProtoIO`, `ProtoBrain`) and ran into one clean structural issue: `ProtoIO` was trying to be both the shared communication protocol (which has to compile for the web browser and every chip) and the chip-specific wiring drivers (which only make sense on a physical microcontroller).

Splitting those into two clear parts gives us four pillars with zero overlap:

- **ProtoUI (`proto-ui-*`):** The browser configurator and builder interface written in Rust/WASM.
- **ProtoSpeak (`proto-speak`):** The shared binary protocol for USB configuration, wired/wireless intra-suit sync, and Proto-to-Proto social messages. Pure `no_std` Rust with no hardware dependencies, so it compiles identically for firmware and the browser.
- **ProtoIO (`proto-io` + `proto-bsp-*`):** Physical hardware traits and board-specific drivers (HUB75 panels, WS2812B strips, sensors, fans, radio chips, flash storage).
- **ProtoBrain (`proto-brain`):** The core engine that runs expressions, animations, sensor triggers, and power/thermal safety limits. Because it is also pure `no_std` Rust, it compiles to WASM too, meaning the browser simulator runs the exact same Rust engine as the Teensy 4.1.

### Modular attachments via `ProtoParts`
Rather than hardcoding every possible accessory (tail, ears, sensors, props) into the brain and UI, every piece of hardware is modeled as a **`ProtoPart`** descriptor defined in `proto-speak` and tracked in a small fixed-size `PartRegistry` inside `proto-brain`:
- Each `ProtoPart` declares its name, kind (`Visor`, `BoopSensor`, `ButtonBank`, `Fan`, `LightZone`, or `Custom`), the **events** it emits, the **actions** it accepts, and optional **UI controls** (sliders, toggles, color pickers).
- **In v1.0:** Only the 4 local head parts exist (`Visor`, `BoopSensor`, `ButtonBank(N)`, `CoolingFan`), keeping RAM and complexity tiny.
- **Future hardware plugin trail (v1.2+):** Because parts are self-describing (similar to USB HID descriptors or Matter device clusters, with no unsafe code upload), a maker can build any custom suit part or accessory (tails, wings, chest panels, body LEDs, or handheld props), connect it over wired `ProtoSpeak` (v1.2) or Bluetooth/ESP-NOW (v1.3), and have it automatically appear in `ProtoBrain` and `ProtoUI` so you can bind events and actions without modifying the helmet firmware or the website.

## 2. Planned Rust workspace layout

```text
protostack/
  Cargo.toml
  crates/
    proto-speak/          # [no_std + wasm] Packet framing, opcodes, profile schema, mesh packets
    proto-brain/          # [no_std + wasm] Frame compositor, trigger rules, safety governor
    proto-io/             # [no_std] Hardware traits: VisorDisplay, AuxLeds, SensorBus, FanPwm, RadioPhy
    proto-bsp-teensy41/   # [ARM Cortex-M7] Teensy 4.1 drivers: imxrt-hal, FlexIO HUB75, imxrt-usbd WebHID
    proto-bsp-esp32s3/    # [Future] ESP32-S3 drivers: esp-hal, native USB HID, ESP-NOW / BLE
    proto-bsp-linux/      # [Future] Raspberry Pi / OrangePi backend
    proto-ui-core/        # [wasm] Shared Rust UI (Leptos), WASM ProtoBrain simulator, CSS tokens
    proto-ui-config/      # Cloudflare micro-frontend: Owner Config site (WebHID + Simulator)
    proto-ui-builder/     # Cloudflare micro-frontend: Builder site (HalfKay flasher, pinouts, panel test)
  boards/
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

## 4. Locked v1 scope and future roadmap

We deliberately cut scope for v1 so we can ship a tight, reliable product with the best web UX in the Protogen space, while leaving clean architectural seams for everything else:

### Locked v1 Scope ("The Wootility Moment")
- **Hardware:** Teensy 4.1 + dual 64x32 HUB75 panels only (with SmartLED Shield V5 / Coela presets, FM6126A latch timing, and runtime RGB/RBG color order). Fully wired single-head system.
- **Inputs:** Boop sensor (APDS9960 or digital pin) + N wired paw/chin buttons, where every button supports **both Tap and Hold** independently in v1.0 (no mic/mouth lip-sync in v1.0; multi-button Combos and Double-Tap move to v1.1).
- **Profiles:** Single active profile in v1.0 (multi-profile slots move to v1.1).
- **Web UI (two Cloudflare micro-frontends, no desktop app):**
  - **Config site (Owners):** Wootility v5-style layout: zero-driver WebHID connect, live WASM visor simulator, starter faces + PNG/GIF animation import (64x32 / 128x32 specs, no pixel painter), live color tint & brightness, boop threshold bar, N-button **Tap + Hold** mapper, inline custom naming (helmet, faces, buttons), current-budget limiter, fan control, RAM preview vs flash save, and Restore Factory Snapshot.
  - **Builder site (Makers & Studios):** Distinct Cloudflare micro-frontend with its own **Hardware Workbench layout** (interactive Teensy 4.1 pinout/wiring canvas + hardware inspector, not a recolored clone of Config) to prevent accidental exploration errors by owners while keeping zero lockouts. Includes in-browser WebHID Teensy 4.1 HalfKay firmware flasher, board/pinout presets, Panel 1/2 orientation test (`PANEL_IDENTIFY`), I2C `SENSOR_SCAN`, and Save Factory Snapshot.
- **Protocol:** `ProtoSpeak` Domain 0 (USB WebHID) active; 12-byte header already reserves `Domain` and `NodeId` fields for future wireless/mesh phases.

### Future Phases (Clean Trails Preserved)
- **v1.1 (More Head Hardware, Gestures, and Pluggable Sensors):** ESP32-S3 board support, WS2812B cheek/ear LEDs, MAX7219 panels, multi-profile slots, multi-button **Combos** (simultaneous presses) and **Double-Tap** button gestures, optional mic/jaw lip-sync, and easy plug-and-detect head sensors.
- **v1.2 (Wired Modular Suit):** Turn on `ProtoSpeak` Domain 1 over a physical wire first (UART / CAN / RS-485). This proves real plug-and-play modularity before touching RF: plug a wired tail, ears, chestplate, or external sensor node into the head bus and it self-registers, shows up in `ProtoUI`, and reacts to boops and face changes.
- **v1.3 (Wireless Modular Suit and Autonomy):** Reuse the already-proven v1.2 modular node system over wireless PHYs (ESP-NOW, BLE, nRF24/LoRa), adding encrypted `SuitPSKey` pairing and the 3-second standalone fallback so a tail keeps running if you leave the head in another room.
- **v2.0 (Social Mesh and Ecosystem):** Opt-in Proto-to-Proto social beacons, procedural vector faces, optional in-browser pixel editor, and Linux SBCs (Raspberry Pi / OrangePi).
