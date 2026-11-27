# ProtoUI: Interface Design, Brandless Tokens, and User Testing Plan

Doc ID: PS-UX-004
Stack: TypeScript + Vite + React (`web/`) wrapping Rust/WASM (`crates/proto-wasm`), deployed on Cloudflare Pages
Status: Current (synced with ADR-001 and ADR-002)

## 1. Single web app with two distinct routes (`/` Config and `/builder` Builder)

Our review of Coela's ProtoTracer uploader showed that Teensy 4.1 can be configured and flashed directly from Chrome/Edge over WebHID, so v1.0 needs no desktop app installer.

To keep non-technical owners out of low-level hardware settings without adding locks, `ProtoUI` is one TypeScript + Vite + React app (`web/`) wrapping `crates/proto-wasm` with two separate route layouts (earlier drafts considered two Leptos micro-frontends; simplified in `ADR-001`, while keeping clean route boundaries if split subdomains are wanted later):

- **`/` (Config route - Wootility-style layout):** Left icon rail, floating sidebar with a Device Switcher dropdown, top action bar (`Undo`, `Redo`, `Save to Head`), and a borderless live dual `64x32` visor preview (running in Simulator Mode or mirroring a connected Phase 1 Teensy 4.1 over WebHID). Supports inline renaming for the helmet name, expressions, and paw buttons, PNG/GIF animation import, color tint, Tap + Hold bindings on 0-8 paw buttons, and LED current-budget protection. (Boop calibration and fan settings join in v1.1.)
- **`/builder` (Builder route - Hardware Workbench layout):** Separate engineering layout built around an interactive Teensy 4.1 PCB pinout diagram, paired with a hardware inspector for WebHID HalfKay firmware flashing, board presets (Breadboard `74HCT245`, SmartLED V5, Coela shields, custom pinouts), Panel 1/2 orientation testing (`PANEL_IDENTIFY`), wiring 0-8 paw buttons, fault log viewer, and saving a Factory Snapshot. (I2C `SENSOR_SCAN` joins in v1.1.)

## 2. Layout and navigation (`/` Config route in v1.0)

1. **Top bar:**
   - Neutral logo slot (`[BRAND_SLOT]` + `ProtoStack`)
   - Connection pill (`Connected: Teensy 4.1` or `Simulator Mode`) + Connect button
   - Status strip: estimated LED power draw (e.g. `1.2A / 3.0A`; adds temp and fan % in v1.1)
   - Save controls: `Previewing in RAM` badge, Undo, Redo, and `Save to Head` button
2. **Left navigation (4 tabs):**
   - **Faces:** Built-in expression gallery, drag-and-drop PNG/GIF animation import (`64x32` per-panel / `128x32` combined), color tint, and mirror toggle.
   - **Buttons (adds Boop in v1.1):** List of 0-8 wired paw buttons with independent Tap and Hold actions and inline renaming.
   - **Power (adds Fan in v1.1):** Master brightness slider and 5V power supply current-budget limiter (soft warnings, never locked).
   - **Profile & Reset:** Export/import `.protoprofile`, one-click Restore Factory Snapshot, and a link to `/builder`.
3. **Center stage (always visible):**
   - Live dual `64x32` HUB75 visor rendered by Rust `ProtoBrain` in WASM (and mirrored live from a connected Teensy 4.1 over WebHID).
   - Virtual `Paw Button 1..N` test buttons underneath so you can test faces and triggers with or without hardware plugged in.

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

## 4. Discovery usability validation (Completed - Sessions 01-03)

Before writing production code, we built clickable HTML prototypes (`prototypes/wireframe-test`, `hifi-test`, `final-test`) and ran three recorded usability sessions covering the main owner and builder flows. (These early sessions also included boop calibration, fan settings, and I2C sensor scan, which are now scheduled for v1.1 per `ADR-002`.)

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
