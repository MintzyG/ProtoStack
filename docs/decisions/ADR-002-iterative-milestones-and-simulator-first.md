# ADR-002: Phased Milestones, Simulator-First Testing, and Review Gates

- **Status:** Accepted
- **Date:** 2026-11-26 (updated 2026-11-27 for Phase 1 USB-only bench)
- **Related docs:** `docs/05-implementation-handoff-checklist.md`, `docs/hardware-dev-bench-bom.md`

## Context

Early discovery covered a full helmet build (dual HUB75 panels, boop sensor, cooling fan). Building everything at once is slow, expensive to buy up front, and hard to debug.

Instead, we are starting on a simple desk setup: a **Teensy 4.1 and 8 breadboard buttons powered over USB**, while the computer renders the `128x32` LED visor live over USB WebHID. Physical HUB75 panels come in Phase 2, and helmet-only extras (boop sensor, cooling fan) move to v1.1.

Why this order works better:
1. We can start coding in v0.1 with zero hardware, then test real firmware, USB WebHID, physical GPIO buttons, flash saving, and browser HalfKay flashing on a cheap USB-only Teensy bench in v0.2-v0.4.
2. The bare-metal FlexIO/FlexPWM/eDMA HUB75 driver is the hardest low-level part. Isolating it in v0.5 keeps hardware timing bugs separate from protocol, button, flash, and UI bugs.
3. Boop sensors and cooling fans are only needed inside an actual helmet, and they fit cleanly into `proto-io` later without rewriting v1.0.

## Decision

### 1. Milestones (v0.1 to v1.0, sensors in v1.1)

- **v0.1 - Basic simulator (no hardware):**
  - `proto-speak`: 12-byte packet header, CRC-16, 64-byte WebHID chunking, `HELLO_INFO`, `PREVIEW_STATE`.
  - `proto-brain`: `128x32` RGB framebuffer, 3 starter faces, color tint, master brightness, 1 virtual paw button.
  - `proto-wasm` + minimal `web/` Config UI (`/`) over an in-memory loopback transport.
  - CI checks (`cargo fmt`, `clippy`, unit tests, golden framebuffer tests, TypeScript build).
- **v0.2 - Teensy 4.1 USB WebHID + PC visor mirror (Phase 1 hardware):**
  - `proto-bsp-teensy41`: `imxrt-usbd` + `usbd-hid` speaking `ProtoSpeak` over real USB WebHID, running `proto-brain` on a USB-powered Teensy 4.1.
  - Physical breadboard buttons update state on the Teensy and mirror live onto the browser's `128x32` canvas.
- **v0.3 - 8 paw buttons, power cap, and flash saving (Phase 1 hardware):**
  - 8 wired paw buttons (4 Left + 4 Right) with independent Tap and Hold actions.
  - Software LED current-budget limiter (computed in firmware, no extra hardware).
  - Atomic A/B flash saving (`COMMIT_FLASH`) and Factory Snapshot save/restore.
- **v0.4 - Custom art and browser flashing (Phase 1 hardware):**
  - PNG/GIF animation import and inline renaming (pencil icons).
  - `/builder` route with Teensy 4.1 PCB pin assignment, `PANEL_IDENTIFY` preview, and WebHID HalfKay firmware flasher.
- **v0.5 - Physical HUB75 LED panels (Phase 2 hardware):**
  - Bare-metal FlexIO + FlexPWM + eDMA HUB75 driver for two chained `64x32` P3 (1/16 scan) panels (`RGB/RBG/BGR` order and physical `PANEL_IDENTIFY`) using `2x SN74HCT245N` level shifters and a 5V 5A bench PSU.
- **v1.0 - Public release:**
  - Long-run stability testing, fault log viewer, Linux udev setup guide, Cloudflare Pages deploy, and contributor docs.
- **v1.1 - Helmet sensors and comfort (Phase 3 hardware):**
  - Boop sensor (APDS-9960 I2C with 1.5 cm hysteresis, plus digital pin mode), 5V PWM cooling fan, I2C `SENSOR_SCAN`, multi-profiles, Combos/Double-Tap, and ESP32-S3 port.

### 2. Simulator-first testing

Every hardware interface in `proto-io` ships with a simulated backend so anyone can develop and test without physical hardware:
- Unit tests in Rust for Tap/Hold timing, current-budget capping, CRC-16 error handling, and A/B flash rollback.
- Golden framebuffer tests that render fixed `128x32` states and fail CI if pixels change unexpectedly.
- Browser simulator and live PC visor mirror using the exact same 64-byte `ProtoSpeak` packets as real USB WebHID.

### 3. Three-gate review workflow

1. **Gate 1 (Before coding):** Agree on exact milestone scope, non-goals, and pass criteria. Wait for OK before writing code.
2. **Gate 2 (During build):** Keep commits small and focused, with passing tests and screenshots/previews when UI changes.
3. **Gate 3 (Before tagging):** Demo the working milestone, log any feedback, and get OK before moving to the next milestone.

## Consequences

- Each milestone produces a working build that can be tested in a browser with no hardware.
- Phase 1 hardware cost stays low (just Teensy 4.1, breadboard, buttons, and a USB cable).
- HUB75 panel bring-up (v0.5) stays isolated behind `proto-io` until panels are on the desk.
