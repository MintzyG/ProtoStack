# ADR-002: Iterative Vertical Milestones, Simulator-First Testing, and Client Review Process

- **Status:** Accepted
- **Date:** 2026-11-26 *(Updated 2026-11-27 for Phase 1 Teensy + Buttons + PC Visor bench)*
- **Applies to:** All implementation work following `docs/05-implementation-handoff-checklist.md`

## Context

The discovery handoff (`docs/05-implementation-handoff-checklist.md`) defines a rich full-helmet target, including dual HUB75 panels, boop sensors, and PWM cooling fans. However, initial development will start as a **lean, USB-powered tabletop bench** (**Teensy 4.1 + 8 breadboard paw buttons**, with the computer rendering the `128x32` LED visor live over USB WebHID), adding physical HUB75 LED panels in Phase 2 and helmet-shell sensors/fan in `v1.1`:

1. **Zero-hardware start (`v0.1`), then low-cost USB-only hardware (`v0.2`–`v0.4`):** Almost the entire stack—bare-metal Rust firmware, real USB WebHID, 8-button Tap+Hold, atomic A/B flash saving, Factory Snapshot, custom GIFs, power-budget math, and browser HalfKay firmware flashing—can be built and tested on a real Teensy 4.1 powered purely off USB before buying LED panels, level shifters, or an external 5V 5A power supply.
2. **Isolating the HUB75 DMA driver (`v0.5`):** The bare-metal Rust FlexIO + FlexPWM + eDMA HUB75 driver is the highest electrical/timing risk in the project. Decoupling it behind `ProtoIO`'s display trait lets us ship and validate the entire controller + configurator first, then plug in physical `2× 64x32` HUB75 panels cleanly in Phase 2.
3. **Helmet-only peripherals (boop proximity sensor, PWM cooling fan, I2C sensor scanning)** are not needed for a tabletop visor bench and bolt cleanly onto `ProtoIO` traits in `v1.1` without rewrites.
4. **Readability, maintainability, and continuous client visibility** require small, inspectable changes rather than huge multi-thousand-line code drops.

## Decision

### 1. Phased Vertical Milestones (`v0.1` → `v1.0`, Sensors in `v1.1`)

ProtoStack will ship in small, runnable vertical slices aligned with our staged hardware bench:

- **`v0.1` — Walking Skeleton (Software-Only / Browser Simulator):**
  - Minimal Rust workspace: `proto-speak` (12-byte envelope, CRC-16, `HELLO_INFO`, `PREVIEW_STATE`) + `proto-brain` (`128x32` RGB framebuffer, 2–3 starter faces, color tint, master brightness, virtual paw button).
  - Compiled to WASM and wired to a minimal TypeScript + Vite + React Config UI (`/`) running over an in-memory `ProtoSpeak` loopback transport.
  - Establishes CI, formatting (`cargo fmt`), linting (`clippy`, TypeScript strict mode), unit tests, and golden framebuffer tests.
- **`v0.2` — Real Teensy 4.1 USB WebHID + Live PC Visor Mirror (Phase 1 Hardware):**
  - Minimal `proto-bsp-teensy41` firmware: `imxrt-usbd` + `usbd-hid` speaking real `ProtoSpeak` over USB WebHID, running `proto-brain` on the Teensy 4.1 powered purely by USB.
  - Physical breadboard paw buttons wired to Teensy GPIOs update the Teensy's live state and mirror immediately onto ProtoUI's `128x32` LED canvas in the browser over real WebHID.
- **`v0.3` — 8 Paw Buttons, Power Cap & Atomic Saving (Phase 1 Hardware):**
  - Full 8 wired paw buttons (4 Left + 4 Right, independent Tap + Hold state machine).
  - Software LED current-budget brightness governor (computed live on Teensy, zero extra hardware required).
  - Atomic A/B flash storage (`COMMIT_FLASH`) and Factory Snapshot save/restore.
- **`v0.4` — Custom Art & Browser Flashing (Phase 1 Hardware):**
  - PNG/GIF animation import + inline renaming with visible pencil icons.
  - `/builder` route with interactive Teensy 4.1 PCB pin assignment, `PANEL_IDENTIFY` preview, and browser WebHID HalfKay firmware flasher.
- **`v0.5` — Physical HUB75 LED Panel Bring-Up (Phase 2 Hardware):**
  - Bare-metal Rust FlexIO + FlexPWM + eDMA HUB75 driver for chained `2× 64x32` P3 (1/16 scan) panels (`RGB/RBG/BGR` channel ordering + physical `PANEL_IDENTIFY`), driven through `2× SN74HCT245N` level shifters and an external 5V 5A PSU.
- **`v1.0` — Solid First Release:**
  - Bench soak testing, fault log viewer, Linux udev guidance, Cloudflare Pages deployment, and contributor documentation.
- **`v1.1` — Helmet Sensors & Comfort Add-ons (Phase 3 Hardware):**
  - Boop sensor support (APDS-9960 I2C with continuous `live_cm <= threshold_cm` + `1.5 cm` hysteresis, plus digital-pin mode), 5V PWM cooling fan control, I2C `SENSOR_SCAN`, multi-profiles, Combos/Double-Tap, and ESP32-S3 port.

### 2. Simulator-First & Hardware-Less Maintainer Design

Every hardware boundary in `proto-io` must ship with a virtual/simulated backend so any contributor without physical hardware (or on Phase 1 hardware without panels) can develop and verify changes:

- **Headless Rust unit & property tests:** Tap/Hold state machine, current-budget capping, CRC-16 corruption handling, and A/B flash power-loss rollback (plus boop hysteresis when `v1.1` lands).
- **Golden framebuffer visual tests:** Render deterministic `128x32` visor states (expressions, tints, brightness capping, `PANEL_IDENTIFY`) and verify pixel outputs / export visual test snapshots so rendering regressions are immediately obvious in PRs.
- **Browser Virtual Helmet & Live PC Visor Mirror:** Full interactive simulation over the exact same `ProtoSpeak` binary frames used by real USB WebHID, plus live state mirroring when connected to a Phase 1 Teensy 4.1 without physical LED panels attached.

### 3. Three-Gate "Client OK" Review Workflow

All engineering work follows a lightweight three-gate loop so changes stay small and every decision is logged:

1. **Gate 1 — Milestone Proposal (Before coding):** Agree on exact scope, non-goals, tradeoffs, and verification criteria. No code is written until the client gives the OK.
2. **Gate 2 — Small, Reviewable PRs / Commits (During build):** Feature branches with small, focused commits, passing automated checks, and plain-English summaries with screenshots or interactive previews when UI/visuals change.
3. **Gate 3 — Milestone Demo & Sign-Off (Before moving on):** Client tests the running milestone, feedback is logged, and only after client OK do we tag the milestone and scope the next slice.

## Consequences

- Every milestone produces a clean, readable, tested artifact that works without physical hardware and incrementally lights up real hardware as parts arrive.
- Phase 1 hardware cost stays minimal (~Teensy 4.1 + breadboard + 8 buttons, USB-powered) while validating real embedded Rust, WebHID, GPIO, flash persistence, and HalfKay flashing.
- Physical HUB75 bring-up (`v0.5`) is cleanly decoupled behind `ProtoIO` traits and de-risked until the panels arrive.
- Architectural changes are continuously captured under `docs/decisions/` for future contributors.
