# ProtoUI: Interface Design, Brandless Tokens, and User Testing Plan

Doc ID: PS-UX-004
Stack: TypeScript + Vite + React (`web/`) wrapping Rust/WASM (`crates/proto-wasm`), deployed on Cloudflare Pages
Status: Current (living specification — synced with ADR-001 & ADR-002)

## 1. Single web app with two visually distinct routes (`/` Config vs `/builder` Builder)

Our research into Coela's ProtoTracer uploader proved that Teensy 4.1 can be both configured and flashed straight from Chrome/Edge over WebHID. That means `v1.0` needs **no desktop app installer** at all.

To minimize accidental tinkering by non-technical owners while keeping strict right-to-repair (zero locks), `ProtoUI` ships as **one TypeScript + Vite + React web app** (`web/`) wrapping our compiled Rust/WASM core (`crates/proto-wasm`) with **two completely distinct route layouts** *(originally explored as two separate Leptos micro-frontends; simplified in `ADR-001` while keeping clean route boundaries if split subdomains are ever wanted later)*:

- **`/` — Config route (for Suit Owners — Wootility v5 layout):** Clean everyday controls built around a narrow left icon rail, a floating rounded sidebar with a proper Device Switcher dropdown card, a floating top action pill bar (`Undo`, `Redo`, `Save to Head`), and a borderless live dual `64x32` visor preview (running in Simulator Mode or mirroring a connected Phase 1 Teensy 4.1 live over WebHID). Supports **inline renaming** for the helmet name, expressions, and paw buttons (e.g. "Thumb", "Index"), PNG/GIF animation import, live color tint, **Tap + Hold** bindings on `0`–`8` wired paw buttons, and LED current-budget protection. *(Boop calibration and fan comfort join in `v1.1`.)*
- **`/builder` — Builder route (for DIY Makers and Commission Studios — distinct Hardware Workbench layout):** Not a recolored clone of Config. Uses a dedicated engineering workbench layout centered on an **interactive Teensy 4.1 PCB pinout and HUB75/button wiring diagram** as the main canvas, paired with a hardware inspector for one-click WebHID HalfKay firmware flashing, board presets (Breadboard `74HCT245`, SmartLED V5, Coela shields, custom pinouts), Panel 1/2 orientation testing (`PANEL_IDENTIFY`), wiring `0`–`8` paw buttons, fault log viewer, and saving a Factory Snapshot. *(I2C `SENSOR_SCAN` joins in `v1.1`.)*

## 2. Layout and navigation (`/` Config route in `v1.0`)

The layout uses a clean three-part workbench similar to Wootility so users always see what their visor is doing:

1. **Top bar:**
   - Neutral logo slot (`[BRAND_SLOT]` + `ProtoStack`)
   - Connection pill (`Connected: Teensy 4.1` or `Simulator Mode`) + Connect button
   - Live status strip: estimated LED power draw (e.g. `1.2A / 3.0A`) *(adds temp & fan % in `v1.1`)*
   - Save controls: `Previewing in RAM` indicator, Undo, Redo, and primary `Save to Head` button
2. **Left navigation (4 focused tabs, zero bloat):**
   - **Faces:** Built-in expression gallery, drag-and-drop PNG/GIF animation import (showing exact `64x32` per-panel / `128x32` specs), live color tint, and mirror toggle.
   - **Buttons (`v1.0`; adds Boop in `v1.1`):** Clean list of `0`–`8` wired paw buttons with independent **Tap** and **Hold** actions + inline renaming.
   - **Power (`v1.0`; adds Fan in `v1.1`):** Master brightness slider and 5V power bank current-budget limiter (soft warnings, never locked).
   - **Profile & Reset:** Export/import `.protoprofile`, one-click Restore Factory Snapshot, and a quiet link to `/builder`.
3. **Center stage (always visible):**
   - Live dual `64x32` HUB75 virtual visor powered by the real Rust `ProtoBrain` compiled to WASM (and mirrored live from physical Teensy 4.1 hardware over WebHID).
   - Virtual test controls underneath (live `Paw Button 1..N` buttons) so you can verify every face and trigger with or without hardware plugged in.

## 3. Brandless CSS token setup

No brand colors are hardcoded into UI components. Everything reads from one token file (`tokens.css`) so a real brand identity can be dropped in later in five minutes:

```css
:root {
  /* Brand slot: swap these when brand styling is ready */
  --ps-brand-accent: #38bdf8;
  --ps-brand-accent-hover: #0ea5e9;
  --ps-brand-surface: rgba(56, 189, 248, 0.12);
  --ps-brand-radius: 10px;
  --ps-font-sans: 'Inter', system-ui, sans-serif;
  --ps-font-mono: 'JetBrains Mono', ui-monospace, monospace;

  /* Neutral UI chrome */
  --ps-bg-canvas: #0b0e14;
  --ps-bg-panel: #131822;
  --ps-bg-elevated: #1b2230;
  --ps-border: rgba(255, 255, 255, 0.08);
  --ps-text: #eef2f8;
  --ps-text-muted: #8b95a8;

  /* Safety and telemetry states */
  --ps-ok: #22c55e;
  --ps-warn: #f59e0b;
  --ps-danger: #ef4444;
}
```

## 4. Discovery usability validation (Completed — Sessions 01–03)

Before writing production code, we built clickable interactive prototypes (`prototypes/wireframe-test`, `hifi-test`, `final-test`) and ran three recorded usability sessions covering the core owner and builder flows (note: discovery sessions also tested boop calibration, fan comfort, and I2C sensor scan, which are now scheduled for `v1.1` per `ADR-002`):

### Historical discovery test tasks

1. **T1 (First plug-in and safe preview):** Connect the simulated Teensy 4.1, switch to Happy, change visor tint to mint green, and confirm whether it is saved permanently yet. Target: under 45 seconds, user clearly understands RAM preview vs saved flash.
2. **T2 (Import a custom animation):** Check the required visor resolution/format specs, import a custom face animation, preview it live on the dual 64x32 visor, and save it to the helmet. Target: under 60 seconds.
3. **T3 (Calibrate boop and map a paw button):** Tighten the boop trigger distance so nearby people stop setting it off, map Paw Button 2 to "Angry", and test both live using the virtual Boop and Paw Button controls *(boop moves to `v1.1`; paw buttons remain in `v1.0`)*. Target: under 60 seconds.
4. **T4 (Prevent a battery brownout):** Lower the 5V power bank budget to 2.0A while brightness is high, observe the live current limiter protecting the helmet, and check fan cooling settings *(fan control moves to `v1.1`; current limiter remains in `v1.0`)*. Target: under 45 seconds.
5. **T5 (Builder route and recovery):** Open the distinct `/builder` Hardware Workbench, switch panel color order to RBG, flash the Panel 1 orientation test pattern, run an I2C sensor scan *(moves to `v1.1`)*, simulate a WebHID firmware flash, and return to `/` Config to test Restore Factory Snapshot. Target: under 75 seconds.

### Validated micro-interaction rules (from Usability Sessions 01-03)

- **Visible edit affordances:** Place subtle pencil icons directly on the helmet name (plus a "Rename Helmet" option inside the sidebar Device Switcher menu), directly on every expression card, on every paw button card, and on the main-screen virtual paw buttons for quick editing without sidebar hunting.
- **Continuous boop evaluation:** Dragging the Boop threshold slider past the live proximity reading must trigger the visor reaction live (with ~1.5 cm release hysteresis), matching real helmet behavior.
- **Clickable custom pin assignment in Builder:** In `Custom` pinout mode, clicking any pin on the Teensy 4.1 PCB opens an inline signal assignment selector directly in the pin bar.
- **No silent actions:** Every persistence or hardware action (Save, Export, Restore, Save Factory Snapshot, Panel Identify, Bus Scan) must show an unmistakable checkmark/toast or active visual pulse.

### Success bars (Verified across Sessions 01-03)

- Rage clicks reduced from **4 -> 0** (`-100%`).
- Entry-to-connected time reduced from **12.1s -> 2.8s** (`-77%`).
- Accidental top-bar disconnects reduced from **1 -> 0** via Wootility's sidebar Device Switcher dropdown.
- Config vs. Builder confusion eliminated via distinct **Wootility v5 Consumer** vs. **Teensy 4.1 PCB Hardware Workbench** layouts.
