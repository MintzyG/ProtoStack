<img src="assets/icon.svg" width="64" height="64" alt="">

# ProtoStack

Open-source Rust firmware and browser configurator for custom Protogen helmets.

Four main parts:
- **ProtoUI** (`web/`): TypeScript + Vite + React web app with an Owner Config view (`/`) and a Builder view (`/builder`), backed by our Rust/WASM core (`crates/proto-wasm`).
- **ProtoSpeak** (`crates/proto-speak`): Shared binary protocol for USB WebHID today, and wired/wireless suit parts later.
- **ProtoIO** (`crates/proto-io`, `crates/proto-bsp-*`): Hardware traits and board drivers.
- **ProtoBrain** (`crates/proto-brain`): Core visor renderer, button handler, and power limiter, compiled for both Teensy 4.1 and browser WASM.

## Docs (`docs/`)

- [01-discovery-and-heuristics.md](docs/01-discovery-and-heuristics.md) - Project goals, personas, competitor review, and usability heuristics
- [02-architecture-and-critique.md](docs/02-architecture-and-critique.md) - Architecture, `ProtoParts` model, Teensy 4.1 design, and roadmap
- [03-protoio-protocol-spec.md](docs/03-protoio-protocol-spec.md) - `ProtoSpeak` v1 binary packet format and opcodes
- [04-protoui-design-and-test-plan.md](docs/04-protoui-design-and-test-plan.md) - Config and Builder UI design, CSS tokens, and usability test results
- [05-implementation-handoff-checklist.md](docs/05-implementation-handoff-checklist.md) - Current v1.0 implementation checklist
- [decisions/ADR-001-frontend-stack-and-routing.md](docs/decisions/ADR-001-frontend-stack-and-routing.md) - Why the web UI uses TypeScript/React + Rust/WASM and a single app with two routes
- [decisions/ADR-002-iterative-milestones-and-simulator-first.md](docs/decisions/ADR-002-iterative-milestones-and-simulator-first.md) - Phased milestones (v0.1 to v1.0), simulator-first testing, and review gates
- [hardware-dev-bench-bom.md](docs/hardware-dev-bench-bom.md) - Dev-bench parts list and Brazil/USD price reference

## Research and Prototypes (`research/` and `prototypes/`)

- [research/README.md](research/README.md) - Code teardowns of ProtoTracer, ProtoESP, LumiFur, VIA WebHID, `teensy4-rs`, and `imxrt-usbd`
- `research/usability-sessions/` - Interview notes, Wootility study, raw `.jsonl` session logs, and reports from Sessions 01-03
- [prototypes/README.md](prototypes/README.md) - Interactive HTML prototypes (`wireframe-test`, `hifi-test`, `final-test`)
