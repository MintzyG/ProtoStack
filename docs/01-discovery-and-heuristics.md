# ProtoStack: Discovery, Competitive Review, and Heuristics Audit

Doc ID: PS-DISC-001
Status: Current (living specification — synced with ADR-001 & ADR-002)

## 1. What we are building

ProtoStack is an open-source software stack with a Rust embedded/WASM core and a TypeScript browser configurator for running and configuring custom Protogen helmets. The main goal is to fix the worst part of owning or building a Protogen today: changing faces, colors, or button settings usually requires installing VS Code and PlatformIO, editing C++ files, and recompiling firmware, or fighting a slow Wi-Fi captive portal on an ESP32.

The user experience benchmark is Wooting's Wootility. A suit owner should be able to plug their helmet (or tabletop dev bench) into a computer over USB, open a website hosted on Cloudflare, and change settings right away with no driver install.

### Core requirements from the client interviews

- Target users are DIY builders, fursuit commission studios building repeatable heads, and non-technical suit owners.
- Right to repair comes first. Builders and studios cannot lock owners out of hardware settings. Safety relies on clear warnings, current limits, and a one-click restore to a saved builder preset, not locks or DRM.
- Hardware priority starts with **Teensy 4.1** in `v1.0` (built incrementally: USB-powered Teensy 4.1 + wired paw buttons with live PC visor rendering over WebHID first, then physical dual `64x32` HUB75 panels in `v0.5`), with clean hardware traits leaving a trail for ESP32-S3, WS2812B/MAX7219, and Linux boards (Raspberry Pi / OrangePi) later.
- `v1.0` is a lean, fully wired single-head / tabletop visor system with `0` to `8` wired paw buttons (independent Tap + Hold) and a software LED current-budget cap. Helmet-shell comfort & sensor add-ons (boop sensor, 5V PWM cooling fan, I2C sensor scanning) were explored in initial discovery and intentionally moved to `v1.1` (see `ADR-002`) so `v1.0` stays fast to bring up on a tabletop bench.
- The system is split into four parts:
  1. `ProtoUI`: a single TypeScript + Vite + React web app hosted on Cloudflare Pages with two visually distinct routes (`/` Owner Config and `/builder` Hardware Workbench), backed by our shared Rust/WASM core *(originally explored as two separate Leptos micro-frontends; updated in `ADR-001`)*.
  2. `ProtoBrain`: the `#![no_std]` Rust runtime that drives faces, animations, triggers, and safety limits—compiled both for Teensy 4.1 and browser WASM.
  3. `ProtoIO`: physical hardware traits and board definitions (HUB75 panels, paw buttons, flash storage in `v1.0`; boop sensors, fans, auxiliary LEDs in `v1.1+`).
  4. `ProtoSpeak`: the shared binary protocol (USB WebHID in `v1.0`, with packet fields already reserved for wired/wireless intra-suit and Proto-to-Proto mesh in later versions).
- Modularity (`ProtoParts`): Every attachment (`Visor` and `ButtonBank(N)` in `v1.0`; `BoopSensor` and `CoolingFan` in `v1.1`; tails, wings, chest panels, body LEDs, extra sensors, or custom community accessories in `v1.2+`) describes itself as a `ProtoPart` with events, actions, and optional UI controls. Multi-piece modularity will be proven over a wired `ProtoSpeak` bus first (`v1.2`) before adding wireless (`v1.3`).
- Wireless autonomy (v1.3+): Once wireless `ProtoParts` are added, standalone pieces like a tail should keep running their own saved animation if the wearer leaves the head in another room, then resync automatically when the head or phone comes back. Wired links stay preferred whenever plugged in for lower latency and shared battery power.
- Visual design should stay neutral and brandless for now, using CSS variables so a brand palette and logo can be added later without rewriting the UI.

## 2. User personas

### Sam (Suit owner, non-technical)
Sam commissioned a Protogen head running a Teensy 4.1 and dual 64x32 HUB75 panels. Sam is comfortable with web apps, does not code, and is nervous about breaking an expensive helmet. Sam wants to plug in a USB cable, change visor colors, import a new expression, adjust boop sensitivity, map paw buttons, and check fan speed in under two minutes without worrying about bricking the head.

### Milo (DIY builder)
Milo is wiring a custom Protogen from scratch with a Teensy 4.1, a SmartLED or Coela shield, a boop sensor, wired paw buttons, and plans to plug in modular parts like a tail or chest panel later. Milo is tired of editing C++ `#define` lines and recompiling every time a panel is upside down or two pins are swapped. Milo wants a visual pin/panel tester and live sensor readouts.

### Kira (Commission studio lead)
Kira builds 12 to 18 heads a year using a consistent wiring layout. Kira wants to flash a new head from a saved studio template in a few minutes, verify current draw and fan behavior, and save a "factory preset" on the helmet so clients can always reset back to a working state if they experiment later.

## 3. Existing tools we compared

We reviewed real source code and docs from competing Protogen projects (see `research/` for full teardowns) plus three strong hardware configurators from outside the Protogen space:

- **Coela's ProtoTracer (Teensy 4.0/4.1, HUB75/WS35):** Very capable 3D blendshape renderer on Teensy, and Coela even wrote a working WebHID firmware uploader for Teensy 4.x. However, all expressions, colors, pins, and feature flags are hardcoded in C++ and built across 10 PlatformIO environments. There is no runtime configurator.
- **ProtoESP (ESP32-S3, MAX7219/WS2812):** Includes a local web UI, a frame animator, and a separate XIAO ESP32 BLE remote. Setup still requires VS Code, PlatformIO, and two separate uploads (firmware plus filesystem). Using the ESP32 Wi-Fi access point (`192.168.4.1`) forces users to drop their phone data and internet connection.
- **LumiFur (ESP32-S3, HUB75):** Clean MatrixPortal ESP32-S3 codebase with NimBLE control, ambient light auto-brightness, and a Web Bluetooth OTA updater. Still requires editing C++ headers and recompiling to remap pins or add faces.
- **Wooting Wootility (reference):** Proves that a hosted web app using WebHID can configure USB hardware with zero drivers, instant live feedback, and clean onboarding.
- **VIA / QMK (reference):** Proves that many different custom community boards can share one zero-driver WebHID website by having the device describe its own layout and capabilities.
- **WLED (reference):** Strong example of a real-time LED current-budget limiter that caps brightness to keep power supplies from browning out.

The gap is clear: nobody in the Protogen space offers a Wootility-style, zero-driver web configurator that works across custom Teensy and ESP32 builds without touching C++ or fighting a Wi-Fi captive portal.

## 4. Usability heuristics audit

We scored current Protogen workflows against Jakob Nielsen's 10 usability heuristics (1 = broken, 5 = great) to set clear UX targets for `ProtoUI`. Current tools average about **1.8 / 5**. Our target for ProtoStack is **4.4+ / 5**.

1. **Visibility of system status (Current: 1.5, Target: 4.8)**
   Today users flash blindly and cannot tell if a sensor is wired wrong, miscalibrated, or drawing too much power. ProtoUI will show a clear connection status, a live virtual visor preview, live sensor bars, and estimated current draw.
2. **Match between system and real world (Current: 1.4, Target: 4.7)**
   Current tools expose raw pin numbers, C++ flags, and ADC numbers to end users. The Owner view will use plain Protogen terms (Boop Distance, Cheek Brightness, Cooling Fan) and keep pinouts and driver timings in the Builder view.
3. **User control and freedom (Current: 1.5, Target: 4.8)**
   Bad uploads can leave a helmet blank with no easy undo. ProtoUI will preview edits in RAM first, require an explicit Save to write flash, support Undo/Redo, and provide a one-click Restore Factory Preset.
4. **Consistency and standards (Current: 2.0, Target: 4.6)**
   Every repo invents its own config format and UI layout. ProtoStack uses one profile format and one UI across boards.
5. **Error prevention (Current: 1.3, Target: 4.7)**
   It is too easy to brown out a 5V power bank, miswire a panel, or turn fans off by mistake. ProtoUI will estimate LED current draw, flag pin conflicts in Builder view, and show clear (dismissible) warnings on thermal or power limits.
6. **Recognition rather than recall (Current: 2.0, Target: 4.5)**
   Makers should not have to memorize matrix chaining order or animation numbers. Builder view will let you flash an orientation test pattern onto Panel 1 or Panel 2, and faces will be picked from visual previews.
7. **Flexibility and efficiency of use (Current: 2.2, Target: 4.5)**
   Owners need fast, simple controls; studios need templates and quick flashing. Keeping Owner and Builder views cleanly separated (while sharing the same core) serves both without cluttering either.
8. **Aesthetic and minimalist design (Current: 2.1, Target: 4.5)**
   Many ESP32 portals dump dozens of ungrouped sliders on one loud page. ProtoUI will stick to a clean dark workbench with a small number of focused sections and no visual noise.
9. **Help users recognize and recover from errors (Current: 1.6, Target: 4.6)**
   Silent USB failures and mysterious brownout reboots are common. ProtoUI will detect unsupported browsers (Firefox/Safari), guide users through WebHID connection issues, and display a short crash/brownout log stored on the board.
10. **Help and documentation (Current: 2.2, Target: 4.3)**
    Setup steps are usually buried in Discord pins. ProtoUI will include a no-hardware Simulator Mode so anyone can explore the app immediately, plus short inline hints where settings are non-obvious.
