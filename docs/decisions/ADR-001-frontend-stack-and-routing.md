# ADR-001: TypeScript UI + Rust/WASM Core, Single Web App with Two Routes

- **Status:** Accepted
- **Date:** 2026-11-26
- **Related docs:** `docs/01-discovery-and-heuristics.md`, `docs/02-architecture-and-critique.md`, `docs/04-protoui-design-and-test-plan.md`, `docs/05-implementation-handoff-checklist.md`

## Context

Early discovery notes proposed writing the entire web UI in Rust/WASM (`Leptos`) and deploying two separate Cloudflare sites (`proto-ui-config` and `proto-ui-builder`) from day one.

Before starting implementation, we reviewed the practical tradeoffs:

1. Rust makes sense for the embedded firmware and the shared core (`proto-brain` and `proto-speak`). Compiling that core to WASM guarantees the browser simulator runs the exact same code as the Teensy 4.1.
2. Writing the UI itself in Leptos adds friction. Browser WebHID bindings in `web-sys` are awkward, canvas rendering, PNG/GIF decoding, HalfKay `.hex` parsing, and SVG pinout maps are much simpler in TypeScript, and Rust/WASM rebuild times slow down UI work.
3. Separating Config and Builder is a UX requirement, not a deployment requirement on day one. Users should never confuse the consumer Config page with the Builder hardware page, which only requires two distinct routes and layouts. Running two separate build pipelines early just adds boilerplate.

## Decision

1. **TypeScript UI + Rust/WASM core:**
   - `crates/proto-speak` and `crates/proto-brain` stay `#![no_std]` Rust and compile to WASM via `crates/proto-wasm` (`wasm-bindgen`).
   - `web/` uses TypeScript, Vite, React, and neutral `--ps-*` CSS tokens. It calls into `proto-wasm` for visor rendering, Tap/Hold button logic, the current-budget limiter, and `ProtoSpeak` packet framing (plus boop hysteresis in v1.1).
   - Browser WebHID (`navigator.hid`), file/image import, and HalfKay `.hex` flashing live in TypeScript behind a small transport interface, with an in-memory loopback transport for Simulator Mode.

2. **One web app, two distinct routes:**
   - `/` is the **Config** view (Wootility-style consumer layout).
   - `/builder` is the **Builder** view (Teensy 4.1 PCB hardware workbench layout).
   - Both routes stay cleanly separated in code so they can still be split into two subdomains later if needed.

## Tradeoffs

- **Pros:**
  - Faster UI iteration and lower barrier for web contributors.
  - Direct access to browser WebHID, Canvas, and File APIs without `web-sys` workarounds.
  - Browser simulator still runs the exact same Rust `proto-brain` and `proto-speak` code as the Teensy 4.1.
  - One build pipeline and one token file during v0.1-v1.0.
- **Cons:**
  - Contributors need both Rust (`cargo`, `wasm-pack`) and Node/JS (`pnpm`) toolchains.
  - The TypeScript <-> WASM boundary needs to stay small and well-typed.
