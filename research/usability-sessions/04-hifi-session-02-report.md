# Usability Session 02 (Hi-Fi Prototype): Comparative Telemetry Report

Session ID: `hifi02_2026-11-26T18-14-52-651Z`
Raw Log: `research/usability-sessions/hifi-session-02.jsonl` (127 events, 4m 12s total)
Baseline Compared Against: Session 01 (`wf_2026-11-26T17-56-11-682Z`, 220 events, 4m 30s)

## 1. Before vs. After Quantitative Comparison

- **Participant Ease Score:** Jumped from **3 / 5** (Session 01) to **4 / 5** (Session 02).

| Metric | Session 01 (Wireframe) | Session 02 (Wootility Hi-Fi) | Change |
|---|---:|---:|---|
| Rage clicks | 4 | 0 | -100% |
| Dead clicks | 10 | 1 | -90% |
| Accidental disconnects | 1 | 0 | -100% |
| Redundant clicks from missing feedback (Save Factory / Export / Panel ID) | 7 | 0 | -100% |
| Entry-to-connected time | 12.1s (1 hesitation) | 5.3s (0 hesitations) | -56% |
| Missed Builder `+` stepper clicks | 3 dead + 1 rage | 0 | -100% |

## 2. What Worked Well in Session 02

- **Wootility v5 layout clicked immediately:** Zero confusion finding Faces, Power & Fan, Boop Sensor, Paw Buttons, and Settings.
- **Device dropdown solved accidental disconnects:** Participant opened and closed the sidebar `Teensy 4.1` dropdown (`65.9s-68.4s`) without disconnecting.
- **Tap + Hold per Paw Button worked cleanly:** Participant tested both `Test Tap` and `Test Hold` across Paw 2 and Paw 3 (`149s-171s`) with zero errors.
- **Tactile feedback eliminated double/triple clicks:** Single clean clicks on Save to Head, Export JSON, Save as Factory Snapshot, Test Panel 1/2, and Scan I2C Bus.

## 3. Remaining Findings to Lock Into Final v1 Spec

### F7. Builder site needs its own engineering layout, not a recolored Config clone (Severity: Medium-High)
- **Evidence:** 8-second hesitation immediately upon entering `builder/flash` (`195.5s-203.5s`).
- **Participant feedback:** An amber banner over the same layout is the wrong approach; Config and Builder should feel genuinely different because their jobs are different.
- **Spec decision:**
  - **Config (`config.protostack.dev`)** keeps the clean Wootility v5 consumer layout (live visor hero, friendly faces, boop, paw buttons, power/fan).
  - **Builder (`builder.protostack.dev`)** uses a distinct **Hardware Workbench** layout (compact engineering header + **interactive Teensy 4.1 PCB pinout & HUB75/sensor wiring diagram** as the main canvas + right-side inspector for firmware flashing, panel timings/identify, bus diagnostics, and Factory Snapshot).

### F8. Custom naming/renaming across the suit (Severity: Medium)
- **Participant feedback:** Expected to rename the helmet itself (character name), custom and built-in expressions, and individual paw buttons (e.g. "Left Index", "Thumb", "Chin").
- **Spec decision:** Add inline click-to-rename fields in `ProtoUI` and store compact UTF-8 labels in `ProtoProfile` (`helmet_name: <= 24 B`, `face.name: <= 20 B`, `button.label: <= 16 B`).
