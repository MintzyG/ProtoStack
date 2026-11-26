# Usability Session 01 (Wireframe): Post-Use Questionnaire & Wootility v5 Study

Session ID: `wf_2026-11-26T17-56-11-682Z`
Date: 2026-11-26

## 1. Post-Use Questionnaire Results

1. **Overall ease of navigation (1-5):** `3 / 5` (Neutral / mixed - clear structure, real friction to fix before hi-fi)
2. **Layout feedback:** Did not yet feel genuinely like Wootility; needed closer alignment with Wootility v5's real visual hierarchy and spacing.
3. **Paw button bindings decision:**
   - **v1.0:** Every wired paw button supports **both Tap AND Hold** independently on the same button.
   - **v1.1:** **Combos** (strictly multi-button presses like `Btn 1 + Btn 2`, not double-taps) and **Double-Tap** as a separate gesture.
4. **Config vs Builder separation:** Distinct Builder header/accent + opens as its own site/tab with an explicit **Exit to Config** button in the top-right bar (never disguised as a sidebar tab).
5. **Profiles scope:** Single active profile only in v1.0 (keep it minimal; multi-profile slots in v1.1).

## 2. Firsthand Wootility v5 Visual Study (`https://wootility.io`, v5.4.3)

We opened Wootility Web directly in the browser and inspected the entry screen, main Quick Settings workbench, and Advanced Keys (Mod Tap) view. Five concrete layout patterns make Wootility feel effortlessly clean:

1. **Two-tier floating dark navigation (not flat hairline boxes):**
   - Far-left narrow icon rail (`~76px`): Logo at top, then high-level sections (`Visor`, `Settings`, `Help`).
   - Elevated rounded secondary sidebar card (`~250px`, `#1a1c20` on `#111316` canvas) with collapsible header, a **Device Switcher card with thumbnail + dropdown chevron** right at the top (so clicking the device opens a selector, never randomly disconnects), and clean grouped nav items.
2. **Floating top pill header over the canvas:**
   - Rounded pill bar floating directly above the hardware preview: active helmet/profile badge on the left; crisp **Undo / Redo** icons and one bold high-contrast primary action CTA (`Save to Head` / `Exit Simulator`) on the right.
3. **Borderless hero hardware canvas:**
   - The hardware visual floats freely in generous dark negative space (no stacked dashed borders or noisy chrome boxes around it).
4. **Click-on-hardware editing:**
   - Controls below the hardware preview live in clean, spaced rounded cards (`#1a1c20`) with subtle `?` help icons and tactile pill switches. On Advanced Keys / Mod Tap, Wootility pairs a clean two-column layout (selection/config on the left, active bindings summary on the right).
5. **Entry screen:**
   - Full-viewport dark canvas, centered bold title, short two-line subtitle, and two clean buttons (`Try out / Simulate` secondary, `Connect Device` primary).

## 3. Action Plan for Iteration 2 (Detailed Hi-Fi Prototype)

All 6 telemetry findings + the Wootility v5 layout study roll straight into our next browser-testable Hi-Fi Prototype (with the same PostHog-style recorder so we can measure the improvement quantitatively against Session 01):
- Rebuild the shell directly on Wootility v5's two-tier floating sidebar + floating top pill bar + borderless Dual 64x32 HUB75 canvas.
- Replace instant-disconnect device click with Wootility's **Device Switcher dropdown card** in the sidebar.
- Give **every** action crisp feedback (save toast/checkmark, live color tint on visor, active press animations + live boop meter pulse, larger 36-40px hit targets to eliminate dead/rage clicks).
- Upgrade Paw Buttons to support **Tap + Hold** simultaneously per button (click a button tile to edit both Tap and Hold cleanly).
- Separate Builder into a visually unmistakable Builder shell (amber/industrial header accent, top-right **Exit to Config** button, zero sidebar confusion).
