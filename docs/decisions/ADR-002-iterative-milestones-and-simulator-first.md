# ADR-002: Iterative Vertical Milestones, Simulator-First Testing, and Client Review Process

- **Status:** Accepted
- **Date:** 2026-11-26
- **Applies to:** All implementation work following `docs/05-implementation-handoff-checklist.md`

## Context

The discovery handoff (`docs/05-implementation-handoff-checklist.md`) defines a well-validated `v1.0` target, but building the entire `v1.0` scope in a single big-bang push carries high engineering risk:

1. **Physical Teensy 4.1 + dual 64x32 HUB75 hardware is not yet on the bench** (expected ~1 month out), and future open-source maintainers may never own every piece of hardware.
2. **The bare-metal Rust FlexIO + FlexPWM + eDMA HUB75 driver** is the highest technical risk in the project and must be isolated and validated directly on hardware rather than entangled with 20 unfinished UI features.
3. **Readability, maintainability, and continuous client visibility** require small, inspectable changes rather than huge multi-thousand-line code drops.

## Decision

### 1. Iterative Vertical Milestones (`v0.1` → `v1.0`)

Instead of a big-bang `v1.0`, ProtoStack will ship in small, runnable vertical slices:

- **`v0.1` — Walking Skeleton (Software-Only / Simulator-First):**
  - Minimal Rust workspace: `proto-speak` (12-byte envelope, CRC-16, `HELLO_INFO`, `PREVIEW_STATE`) + `proto-brain` (`128x32` RGB framebuffer, 2–3 starter faces, color tint, master brightness, basic boop + 1 paw button).
  - Compiled to WASM and wired to a minimal TypeScript Config UI running over an in-memory `ProtoSpeak` loopback transport.
  - Establishes CI, formatting (`cargo fmt`), linting (`clippy`, TypeScript strict mode), unit tests, and golden framebuffer tests.
- **`v0.2` — Live Hardware Spike ("The Wootility Moment"):**
  - Minimal `proto-bsp-teensy41` firmware: `imxrt-usbd` + `usbd-hid` speaking `ProtoSpeak` over real USB WebHID, plus first verified output / `PANEL_IDENTIFY` on physical dual 64x32 HUB75 panels.
  - Real browser WebHID connection updating the physical visor live in RAM.
- **`v0.3` — Real Inputs, Safety & Persistence:**
  - Continuous boop sensor evaluation (`live_cm <= threshold_cm` + `1.5 cm` hysteresis) and N wired paw buttons (independent Tap + Hold).
  - LED current-budget brightness governor, PWM cooling fan control, and 20 Hz telemetry stream.
  - Atomic A/B flash storage (`COMMIT_FLASH`) and Factory Snapshot save/restore.
- **`v0.4` — Media Import & Builder Workbench:**
  - PNG/GIF animation import + inline renaming with visible pencil icons.
  - Builder PCB workbench (`/builder`), `PANEL_IDENTIFY`, I2C `SENSOR_SCAN`, click-to-assign custom pins, and browser WebHID HalfKay firmware flasher.
- **`v1.0` — Hardening & Public Release:**
  - Real-helmet soak testing, Linux udev guidance, fault log viewer, Cloudflare Pages deployment, and contributor documentation.

### 2. Simulator-First & Hardware-Less Maintainer Design

Every hardware boundary in `proto-io` must ship with a virtual/simulated backend so any contributor without physical hardware can develop and verify changes:

- **Headless Rust unit & property tests:** Boop hysteresis, Tap/Hold state machine, current-budget capping, CRC-16 corruption handling, and A/B flash power-loss rollback.
- **Golden framebuffer visual tests:** Render deterministic `128x32` visor states (expressions, tints, brightness capping, `PANEL_IDENTIFY`) and verify pixel outputs / export visual test snapshots so rendering regressions are immediately obvious in PRs.
- **Browser Virtual Helmet:** Full interactive simulation over the exact same `ProtoSpeak` binary frames used by real USB WebHID.

### 3. Three-Gate "Client OK" Review Workflow

All engineering work follows a lightweight three-gate loop so changes stay small and every decision is logged:

1. **Gate 1 — Milestone Proposal (Before coding):** Agree on exact scope, non-goals, tradeoffs, and verification criteria. No code is written until the client gives the OK.
2. **Gate 2 — Small, Reviewable PRs / Commits (During build):** Feature branches with small, focused commits, passing automated checks, and plain-English summaries with screenshots or interactive previews when UI/visuals change.
3. **Gate 3 — Milestone Demo & Sign-Off (Before moving on):** Client tests the running milestone, feedback is logged, and only after client OK do we tag the milestone and scope the next slice.

## Consequences

- Every milestone produces a clean, readable, tested artifact that works without physical hardware.
- Hardware bring-up (`v0.2`) is cleanly decoupled and de-risked once the Teensy 4.1 + HUB75 bench arrives.
- Architectural changes are continuously captured under `docs/decisions/` for future contributors.
