<img src="assets/icon.svg" width="64" height="64" alt="">

# ProtoStack

Open-source Rust embedded runtime and zero-driver browser configurator for custom Protogen helmets.

Built around four clean pillars: **ProtoUI** (TypeScript + Vite + React web app with distinct Owner Config and Builder routes, powered by our Rust/WASM core), **ProtoSpeak** (shared USB WebHID and future suit/social protocol with self-describing `ProtoParts`), **ProtoIO** (hardware traits and board drivers), and **ProtoBrain** (core visor engine and safety governor, compiled for both Teensy 4.1 and browser WASM simulation).

## Specification & Architecture Decisions (`docs/`)

- [01-discovery-and-heuristics.md](docs/01-discovery-and-heuristics.md) — Discovery summary, personas, competitive review, and Nielsen's 10 heuristics audit
- [02-architecture-and-critique.md](docs/02-architecture-and-critique.md) — 4-pillar architecture, `ProtoParts` model, Teensy 4.1 technical design, and roadmap
- [03-protoio-protocol-spec.md](docs/03-protoio-protocol-spec.md) — `ProtoSpeak` v1 binary framing, WebHID control opcodes, wired/wireless modular suit spec, and social beacons
- [04-protoui-design-and-test-plan.md](docs/04-protoui-design-and-test-plan.md) — Config vs Builder IA, brandless CSS tokens, and validated UX micro-interaction rules
- [05-implementation-handoff-checklist.md](docs/05-implementation-handoff-checklist.md) — Discovery handoff checklist
- [decisions/ADR-001-frontend-stack-and-routing.md](docs/decisions/ADR-001-frontend-stack-and-routing.md) — Decision: TypeScript UI + Rust/WASM core, single web app with distinct Config/Builder routes
- [decisions/ADR-002-iterative-milestones-and-simulator-first.md](docs/decisions/ADR-002-iterative-milestones-and-simulator-first.md) — Decision: Iterative `v0.1`→`v1.0` vertical milestones, simulator-first testing, and 3-gate review loop
- [hardware-dev-bench-bom.md](docs/hardware-dev-bench-bom.md) — Plain-English HUB75/shield primer and dev-bench shopping cart (USD & BRL)

## Primary-Source Research & Usability Evidence (`research/` & `prototypes/`)

- [research/README.md](research/README.md) — Source-code teardowns of ProtoTracer, ProtoESP, LumiFur, VIA WebHID, `teensy4-rs`, and `imxrt-usbd`
- `research/usability-sessions/` — Pre/post interviews, Wootility v5.4.3 study, raw PostHog-style `.jsonl` recordings, and comparative telemetry reports across Sessions 01-03
- [prototypes/README.md](prototypes/README.md) — Versioned interactive browser prototypes (`wireframe-test`, `hifi-test`, `final-test`) with local telemetry servers
