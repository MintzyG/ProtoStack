# ProtoStack v1.0: Engineering Handoff Checklist

Doc ID: PS-HANDOFF-005
Status: Approved for Implementation Handoff (Specification Studio Complete)

This document is the day-one map for the engineering team implementing **ProtoStack v1.0**. Every requirement below is backed by primary-source repo teardowns (`research/`) and three recorded usability test sessions (`research/usability-sessions/`).

## 1. Locked v1.0 Scope (Build This, Nothing Extra)

- **Hardware target:** Teensy 4.1 (i.MX RT1062) + dual 64x32 HUB75 RGB LED panels only. Fully wired single-head system.
- **Inputs:** 1 boop sensor (APDS9960 over I2C or digital trigger pin) + N wired paw buttons (0 to 8), where **each button supports both Tap and Hold independently**.
- **Power and thermal:** Real-time LED current-budget governor (caps brightness softly to protect 5V power banks) + PWM cooling fan control with soft non-locking warnings under 20%.
- **Profiles and expressions:** Single active profile in v1.0. Built-in starter faces + drag-and-drop PNG/GIF/sprite import (64x32 per panel or 128x32 combined, up to 30 FPS). No built-in pixel-art painter in v1.0.
- **Custom naming:** Inline rename (with visible pencil icons) for helmet name (`<= 24 B`), expressions (`<= 20 B`), and paw buttons (`<= 16 B`).
- **Right to repair:** Zero builder locks. Live edits preview in RAM first; explicit `Save to Head` writes atomically to A/B flash slots; one-click `Restore Factory Snapshot` (plus physical 5-second hold at boot) reverts safely.
- **Two Cloudflare Rust/WASM micro-frontends (no desktop app in v1.0):**
  1. `config.protostack.*`: Wootility v5-inspired consumer layout (entry screen with Connect / Simulate, narrow icon rail, floating rounded sidebar with Device Switcher dropdown, top action pill bar with Undo/Redo/Save, borderless live WASM visor, spaced cards, visible pencil rename icons).
  2. `builder.protostack.*`: Separate Hardware Workbench layout (dark grid, interactive Teensy 4.1 PCB pinout canvas with click-to-assign pins in Custom mode, panel test strip with `RGB/RBG/BGR` + `Test Panel 1/2`, live diagnostics console, WebHID HalfKay firmware flasher, Save Factory Snapshot).

## 2. Rust Workspace Crates and Clean Future Trails

1. **`crates/proto-speak` (`#![no_std]` + WASM):**
   - 12-byte packet envelope (`Ver`, `Domain`, `Flags`, `Opcode`, `SrcNodeId`, `DstNodeId`, `Seq`, `Len`, `Postcard` payload, `CRC-16`).
   - v1.0 implements Domain 0 (USB WebHID, 64-byte reports, Vendor Usage Page `0xFF60` / Usage `0x61`, serialized FIFO command queue, `BOOTLOADER_JUMP = 0x00FE`).
   - Includes the lightweight `ProtoPart` self-describing descriptor struct (v1.0 uses only the 4 local head parts: `Visor`, `BoopSensor`, `ButtonBank(N)`, `CoolingFan`).
2. **`crates/proto-brain` (`#![no_std]` + WASM):**
   - Logical `128x32` framebuffer compositor, continuous boop threshold evaluator (`live_cm <= threshold_cm` with 1.5 cm hysteresis), Tap + Hold paw state machine, current-budget brightness governor, fixed-size `PartRegistry`.
   - Compiles directly to WASM so the browser simulator runs the exact same engine as the Teensy 4.1.
3. **`crates/proto-io` (`#![no_std]`):**
   - Hardware traits (`VisorDisplay`, `SensorInput`, `ButtonBank`, `FanPwm`, `PowerMonitor`, `FlashStore`) keeping ESP32-S3, MAX7219, WS2812B, and Linux SBCs cleanly decoupled.
4. **`crates/proto-bsp-teensy41` (`thumbv7em-none-eabihf`):**
   - `teensy4-bsp 0.6` + `imxrt-hal 0.6` + `imxrt-usbd 0.4` + `rtic 2`.
   - FlexIO + FlexPWM + eDMA double-buffered HUB75 driver with verified SmartLED Shield V5 defaults (`18.462 MHz` clock divider 26, `100 ns` FM6126A latch, `400 ns` latch-to-clock, runtime `RGB/RBG/BGR` swap).
   - `BOOTLOADER_JUMP` triggers `bkpt #251` into HalfKay (`0x16C0:0x0479`) for zero-disassembly browser flashing (1088-byte HID pages, 1500 ms block-0 erase wait, 5 ms per block, `[0xFF, 0xFF, 0xFF]` reboot).
5. **`crates/proto-ui-core`, `crates/proto-ui-config`, `crates/proto-ui-builder`:**
   - Rust/WASM (`Leptos`) with neutral `--ps-*` CSS design tokens so final branding drops into one token file later.

## 3. Deferred Roadmap (Do Not Build in v1.0)

- **v1.1:** ESP32-S3, WS2812B cheeks/ears, MAX7219, multi-profile slots, multi-button **Combos** (simultaneous presses only) + **Double-Tap**, optional mic/jaw lip-sync, plug-and-detect head sensors.
- **v1.2:** Wired modular suit (`ProtoSpeak` Domain 1 over UART/CAN/RS-485: plug in custom `ProtoParts` like tails, wings, chest panels, or body LEDs that self-register over the bus).
- **v1.3:** Wireless modular suit (same Domain 1 over ESP-NOW/BLE/nRF24, encrypted `SuitPSKey`, 3-second autonomous standalone fallback when the head leaves the room).
- **v2.0:** Opt-in Proto-to-Proto social beacons, procedural vector faces, optional in-browser pixel editor, Linux SBCs (Raspberry Pi / OrangePi).

## 4. Artifact & Evidence Index

- Specs: `docs/01-discovery-and-heuristics.md`, `docs/02-architecture-and-critique.md`, `docs/03-protoio-protocol-spec.md`, `docs/04-protoui-design-and-test-plan.md`, `docs/05-implementation-handoff-checklist.md`
- Source teardowns: `research/01-prototracer-teardown.md`, `research/02-esp32-ecosystems-teardown.md`, `research/03-webhid-and-rust-teensy-feasibility.md`
- Usability recordings & reports: `research/usability-sessions/01` through `05` plus raw `.jsonl` PostHog telemetry logs
- Versioned interactive prototypes: `prototypes/wireframe-test/`, `prototypes/hifi-test/`, `prototypes/final-test/` (visual reference target)
