<img src="assets/icon.svg" width="64" height="64" alt="">

# ProtoStack

Open-source Rust embedded runtime and zero-driver browser configurator for custom Protogen helmets.

Built around four clean pillars: **ProtoUI** (Cloudflare Config + Builder micro-frontends), **ProtoSpeak** (shared USB WebHID and future suit/social protocol with self-describing `ProtoParts`), **ProtoIO** (hardware traits and board drivers), and **ProtoBrain** (core visor engine and safety governor, compiled for both Teensy 4.1 and browser WASM simulation).

## Specification Package (`docs/`)

- [01-discovery-and-heuristics.md](docs/01-discovery-and-heuristics.md) — Discovery summary, personas, competitive review, and Nielsen's 10 heuristics audit
- [02-architecture-and-critique.md](docs/02-architecture-and-critique.md) — 4-pillar architecture, `ProtoParts` model, Teensy 4.1 technical design, and locked v1.0 to v2.0 roadmap
- [03-protoio-protocol-spec.md](docs/03-protoio-protocol-spec.md) — `ProtoSpeak` v1 binary framing, WebHID control opcodes, wired/wireless modular suit spec, and social beacons
- [04-protoui-design-and-test-plan.md](docs/04-protoui-design-and-test-plan.md) — Config vs Builder micro-frontend IA, brandless CSS tokens, and validated UX micro-interaction rules
- [05-implementation-handoff-checklist.md](docs/05-implementation-handoff-checklist.md) — Day-one engineering handoff checklist

## Primary-Source Research & Usability Evidence (`research/` & `prototypes/`)

- [research/README.md](research/README.md) — Source-code teardowns of ProtoTracer, ProtoESP, LumiFur, VIA WebHID, `teensy4-rs`, and `imxrt-usbd`
- `research/usability-sessions/` — Pre/post interviews, Wootility v5.4.3 study, raw PostHog-style `.jsonl` recordings, and comparative telemetry reports across Sessions 01-03
- [prototypes/README.md](prototypes/README.md) — Versioned interactive browser prototypes (`wireframe-test`, `hifi-test`, `final-test`) with local telemetry servers
