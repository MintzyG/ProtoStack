# Usability Session 01 (Wireframe): Telemetry Analysis and Findings

Session ID: `wf_2026-11-26T17-56-11-682Z`
Raw Log: `research/usability-sessions/wireframe-session.jsonl` (220 events, 4m 30s total)
Viewport: `1956x1087` (Chrome Linux)

## 1. Quantitative Summary

- Total duration: `270.1s` (4m 30s)
- Total tracked events: `220`
- Page dwell breakdown:
  - `config/connect`: `12.1s` (1 hesitation at 8.0s before clicking Connect USB)
  - `config/faces`: `55.9s` across 2 visits
  - `config/inputs`: `74.7s` across 2 visits (1 hesitation at 8.0s while inspecting Paw Button Tap/Hold)
  - `config/power`: `46.3s`
  - `config/reset`: `19.4s` across 2 visits
  - `builder/flash`: `11.2s` across 2 visits
  - `builder/panels`: `24.3s` across 2 visits
  - `builder/wiring`: `17.9s` across 2 visits
- **Rage clicks detected: 4**
  - `sim_paw_1` (2 rage-click bursts at `155.0s` and `158.5s`)
  - `sim_boop` (1 rage-click burst at `156.6s`)
  - `builder/wiring` next to the `+` paw-button stepper (`245.5s`, 3 missed clicks 33px off target)
- **Dead clicks detected: 10** (clicking non-interactive card backgrounds, slider tracks, and beside small stepper buttons)

## 2. Concrete Friction Findings (Telemetry + Participant Debrief)

### F1. Missing visual feedback on actions (Severity: High)
- **Evidence:** Participant clicked `Save Factory` 3 times in 1.3s (`207.2s`, `208.0s`, `208.6s`), `Export` twice (`198.2s`, `199.7s`), all 6 monochrome tint swatches (`37.6s-43.9s`), and rage-clicked `Boop` / `Paw 1` virtual triggers (`154.4s-158.6s`).
- **Root cause:** Actions updated internal state silently with no toast/checkmark, no active button pulse, and monochrome swatches did not change the visor preview.
- **Fix:** Every action must produce immediate tactile feedback (button press state + brief checkmark/toast on Save/Export/Snapshot, real color tint on visor, live ripple + boop meter spike when virtual triggers fire).

### F2. Accidental disconnect from clicking the device name pill (Severity: High)
- **Evidence:** At `54.2s`, participant clicked `Teensy 4.1` in the top bar; it immediately disconnected USB and switched to `Simulator`, forcing a 3.8s reconnect (`55.9s-58.1s`).
- **Participant comment:** Did not expect clicking the helmet name to disconnect and switch into Simulator mode.
- **Fix:** Clicking the top-bar device pill opens a clean **Device Popover** (board/firmware info, Switch Device, and an explicit Disconnect / Simulator action), never an instant toggle.

### F3. Confusing Config <-> Builder transition (Severity: High)
- **Evidence:** Rapid bounce between `config/reset` -> `builder/flash` -> `panels` -> `wiring` -> `config/faces` -> `config/reset` -> `builder/flash` (`202s-262s`).
- **Participant comment:** Clicking `Open Builder` barely changed the visual shell, and having `Config` as a regular item inside Builder's sidebar felt confusing.
- **Fix:** Give the two micro-frontends distinct visual framing (separate Builder accent header and clear `Exit to Config` top-bar link instead of a sidebar tab, plus a clear `Open Builder` handoff with unmistakable visual shift).

### F4. Paw buttons lacked simultaneous Tap + Hold and Button Combos (Severity: High - Feature Gap)
- **Evidence:** 8-second hesitation (`91.5s-99.5s`) right before opening `paw_1_mode` (`Tap / Hold`), followed by leaving the tab without changing it.
- **Participant comment:** Expected to bind **both** a Tap and a Hold on the same paw button, plus **button combinations** (e.g. Btn 1 + Btn 2).
- **Fix:** Upgrade `ProtoBrain` and `ProtoUI` Inputs to a Wootility Mod-Tap style binding model:
  - Each wired paw button supports **both** a `Tap` action and an optional `Hold` action simultaneously.
  - Add a clean **Combos** section (e.g. `Btn 1 + Btn 2 -> Blush`) using a tiny bitmask (`buttons_mask: u8`) in `ProtoBrain`—zero extra hardware required!

### F5. Small hit targets causing dead clicks and rage clicks (Severity: Medium)
- **Evidence:** 3 dead clicks + rage click at `x=1461, y=407` missing the `+` button at `x=1428, y=407` (`244.9s-245.5s`); 6 additional dead clicks near Boop and Power sliders.
- **Fix:** Larger hit targets (36px stepper buttons), full-width clickable slider rows, and interactive Wootility-style key/button tiles.

### F6. Layout polish toward real Wootility v5 structure (Severity: Medium)
- **Participant comment:** Wants the layout to be more genuinely inspired by Wootility's clean visual structure.
- **Fix:** Adopt Wootility v5's proven layout: dedicated left sidebar (`Profiles`, `Faces`, `Inputs`, `Power & Fan`, `System`), top-bar profile switcher + device popover + Undo/Redo + unmistakable Save CTA, spacious visor hero preview, and interactive click-to-bind paw button diagram (click a button tile to edit Tap/Hold, or select two to add a Combo).
