# ADR-001: TypeScript UI + Rust/WASM Core, and Single Web App with Distinct Routes

- **Status:** Accepted
- **Date:** 2026-11-26
- **Supersedes (partially):** `docs/02-architecture-and-critique.md` (Section 2, Leptos / two micro-frontends) and `docs/04-protoui-design-and-test-plan.md` (Section 1, two Cloudflare micro-frontends)

## Context

The discovery specification proposed writing the entire `ProtoUI` frontend in Rust/WASM (`Leptos`) and deploying two separate Cloudflare micro-frontends (`proto-ui-config` and `proto-ui-builder`) from day one.

During pre-implementation engineering review with the client, we evaluated the practical tradeoffs for maintainability, contributor accessibility, and shipping speed:

1. **Rust earns its place in embedded and shared core logic**, where `#![no_std]` safety and identical execution between Teensy 4.1 and the browser WASM simulator (`proto-brain` + `proto-speak`) eliminate simulation drift.
2. **Writing the browser UI itself in Leptos adds friction without user-facing benefit:** `web-sys` WebHID bindings are incomplete/unstable, browser-heavy tasks (HTML5 canvas, drag-and-drop PNG/GIF decoding, HalfKay Intel `.hex` parsing, interactive SVG PCB pinout workbench) are far more ergonomic in TypeScript, and Rust/WASM rebuild loops slow down UI polish.
3. **Config vs. Builder separation is a UX requirement, not a deployment requirement on day one.** Usability Session 03 proved that users must never confuse the consumer Config view with the Maker/Studio Builder PCB workbench. Maintaining two separate build/deploy pipelines early adds duplication and version-sync overhead with zero benefit until the shared UI stabilizes.

## Decision

1. **Hybrid Architecture (TypeScript UI + Rust/WASM Core):**
   - `crates/proto-speak` and `crates/proto-brain` remain pure `#![no_std]` Rust and compile to `wasm32-unknown-unknown` via `wasm-bindgen`.
   - `ProtoUI` (`web/`) will be built with **TypeScript + Vite + React** and brandless `--ps-*` CSS tokens, consuming the compiled Rust/WASM module for visor rendering, boop hysteresis, Tap+Hold state machine, current-budget governor, and `ProtoSpeak` packet framing.
   - All WebHID transport (`navigator.hid`), file/image import, and HalfKay `.hex` flashing live cleanly in TypeScript behind a thin transport interface (allowing an in-memory loopback transport in Simulator Mode).

2. **Single Web App, Two Visually Distinct Routes:**
   - Start with one web application serving two completely distinct layouts:
     - `/` — **Config** (Wootility v5-style consumer layout)
     - `/builder` — **Builder** (Teensy 4.1 PCB Hardware Workbench layout)
   - Keep the route boundary clean so they can still be deployed to separate Cloudflare subdomains (`config.*` and `builder.*`) later with minimal effort if desired.

## Tradeoffs & Consequences

- **Pros:**
  - Faster UI iteration and much lower barrier for open-source web contributors.
  - Native, strongly typed WebHID, Canvas, and file APIs without unstable `web-sys` wrappers.
  - Still runs the **exact same Rust `ProtoBrain` and `ProtoSpeak` code** in the browser simulator and on the Teensy 4.1.
  - Single repo/build pipeline during early milestones (`v0.1`–`v0.4`), zero duplicated state or tokens.
- **Cons:**
  - Repo requires both Rust (`cargo` + `wasm-pack`) and JS/TS (`pnpm`) toolchains.
  - Requires keeping the TypeScript ↔ WASM boundary small, explicit, and well-typed.
