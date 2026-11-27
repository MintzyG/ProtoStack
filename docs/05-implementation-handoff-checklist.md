# ProtoStack v1.0: Engineering Handoff Checklist

Doc ID: PS-HANDOFF-005
Status: Current (synced with ADR-001 and ADR-002)

This is the active implementation checklist for **ProtoStack v1.0**. Requirements are backed by the repo teardowns (`research/`), three usability sessions (`research/usability-sessions/`), and `docs/decisions/ADR-001` and `ADR-002`.

## 1. Current v1.0 Scope

- **Hardware (two phases):**
  - **Phase 1 (v0.2 - v0.4):** USB-powered Teensy 4.1 (i.MX RT1062) + 0 to 8 breadboard paw buttons running Rust firmware and mirroring the `128x32` visor to `ProtoUI` over real USB WebHID.
  - **Phase 2 (v0.5 - v1.0):** Physical dual `64x32` HUB75 RGB LED panels (1/16 scan) via `2x SN74HCT245N` level shifters + 5V 5A bench PSU (plus SmartLED V5 / Coela presets).
- **Inputs:** 0 to 8 wired paw buttons, each supporting independent Tap and Hold actions. (Boop sensor and 5V PWM cooling fan were reviewed in early discovery and moved to v1.1; see `ADR-002`.)
- **Power safety:** Software LED current-budget limiter (scales brightness smoothly to protect 5V power supplies/banks, no extra hardware needed).
- **Profiles and expressions:** Single active profile in v1.0. Built-in starter faces + drag-and-drop PNG/GIF import (`64x32` per panel or `128x32` combined, up to 30 FPS). No pixel-art editor in v1.0.
- **Custom naming:** Inline rename (visible pencil icons) for helmet name (`<= 24 B`), expressions (`<= 20 B`), and paw buttons (`<= 16 B`).
- **Right to repair:** No builder locks. Edits preview in RAM first; explicit `Save to Head` writes atomically to A/B flash slots; one-click `Restore Factory Snapshot` (plus 5-second button hold at boot) reverts safely.
- **Single TypeScript + Vite + React web app (`web/`) wrapping Rust/WASM (`crates/proto-wasm`), no desktop app (`ADR-001`):**
  1. **`/` (Config route):** Wootility-style consumer layout (Connect / Simulate entry screen, left icon rail, floating sidebar with Device Switcher dropdown, top Undo/Redo/Save bar, borderless live WASM visor, spaced cards, pencil rename icons).
  2. **`/builder` (Builder route):** Separate Hardware Workbench layout (dark grid, interactive Teensy 4.1 PCB pinout canvas with click-to-assign pins in Custom mode, panel test strip with `RGB/RBG/BGR` + `Test Panel 1/2`, fault log / diagnostics console, WebHID HalfKay firmware flasher, Save Factory Snapshot).

## 2. Workspace Layout

1. **`crates/proto-speak` (`#![no_std]` + WASM):**
   - 12-byte packet envelope (`Ver`, `Domain`, `Flags`, `Opcode`, `SrcNodeId`, `DstNodeId`, `Seq`, `Len`, `Postcard` payload, `CRC-16`).
   - v1.0 implements Domain 0 (USB WebHID, 64-byte reports, Vendor Usage Page `0xFF60` / Usage `0x61`, serialized FIFO command queue, `BOOTLOADER_JUMP = 0x00FE`).
   - Includes the `ProtoPart` descriptor struct (v1.0 uses `Visor` and `ButtonBank(N)`; v1.1 adds `BoopSensor` and `CoolingFan`).
2. **`crates/proto-brain` (`#![no_std]` + WASM):**
   - Logical `128x32` framebuffer compositor, Tap + Hold button state machine, current-budget brightness limiter, fixed-size `PartRegistry` (plus continuous boop evaluator with 1.5 cm hysteresis in v1.1).
   - Compiles to WASM so the browser simulator and PC visor mirror run the exact same engine as the Teensy 4.1.
3. **`crates/proto-io` (`#![no_std]`):**
   - Hardware traits (`VisorDisplay`, `ButtonBank`, `FlashStore`; plus `SensorInput`, `FanPwm`, `PowerMonitor` for v1.1+) keeping ESP32-S3, MAX7219, WS2812B, and Linux SBCs decoupled.
4. **`crates/proto-wasm` (`wasm-bindgen`):**
   - Thin WASM bridge exposing `ProtoBrain` and 64-byte `ProtoSpeak` packet processing to `web/`.
5. **`crates/proto-bsp-teensy41` (`thumbv7em-none-eabihf`):**
   - `teensy4-bsp` + `imxrt-hal` + `imxrt-usbd` + `rtic 2`.
   - Phase 1 (v0.2 - v0.4): USB WebHID (`ProtoSpeak`), 8-button GPIO input, atomic A/B flash storage, and `BOOTLOADER_JUMP` (`bkpt #251`) into HalfKay (`0x16C0:0x0478/0479`, 1088-byte HID pages, 1500 ms block-0 erase wait, 5 ms per block, `[0xFF, 0xFF, 0xFF]` reboot).
   - Phase 2 (v0.5): FlexIO + FlexPWM + eDMA double-buffered HUB75 driver (18.462 MHz clock divider 26, 100 ns FM6126A latch, 400 ns latch-to-clock, runtime `RGB/RBG/BGR` swap).
6. **`web/` (`ProtoUI` - TypeScript + Vite + React):**
   - Single web app with `/` Config and `/builder` Builder routes, native WebHID + HalfKay `.hex` flasher, and neutral `--ps-*` CSS tokens.

## 3. Deferred Roadmap (Do Not Build in v1.0)

- **v1.1:** Boop sensor (APDS-9960 I2C with 1.5 cm hysteresis + digital pin), 5V PWM cooling fan, I2C `SENSOR_SCAN`, ESP32-S3, WS2812B cheeks/ears, MAX7219, multi-profile slots, multi-button Combos (simultaneous presses only) + Double-Tap, optional mic/jaw lip-sync, and plug-and-detect head sensors.
- **v1.2:** Wired modular suit (`ProtoSpeak` Domain 1 over UART/CAN/RS-485: plug in custom `ProtoParts` like tails, wings, chest panels, or body LEDs that self-register over the bus).
- **v1.3:** Wireless modular suit (same Domain 1 over ESP-NOW/BLE/nRF24, encrypted `SuitPSKey`, 3-second autonomous standalone fallback when the head leaves the room).
- **v2.0:** Opt-in Proto-to-Proto social beacons, procedural vector faces, optional in-browser pixel editor, Linux SBCs (Raspberry Pi / OrangePi).

## 4. Artifact & Evidence Index

- Specs: `docs/01-discovery-and-heuristics.md`, `docs/02-architecture-and-critique.md`, `docs/03-protoio-protocol-spec.md`, `docs/04-protoui-design-and-test-plan.md`, `docs/05-implementation-handoff-checklist.md`
- Source teardowns: `research/01-prototracer-teardown.md`, `research/02-esp32-ecosystems-teardown.md`, `research/03-webhid-and-rust-teensy-feasibility.md`
- Usability recordings & reports: `research/usability-sessions/01` through `05` plus raw `.jsonl` PostHog telemetry logs
- Versioned interactive prototypes: `prototypes/wireframe-test/`, `prototypes/hifi-test/`, `prototypes/final-test/` (visual reference target)
