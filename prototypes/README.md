# ProtoUI Versioned Prototypes & Visuals Index

All HTML prototypes and charts created during discovery and usability testing are kept versioned here so implementation agents can inspect the exact evolution and test any build locally.

## Interactive Prototypes (`prototypes/`)

| Version | File(s) | Description | Paired Usability Log |
|---|---|---|---|
| **Draft 0a (Inline Hi-Fi)** | `protoui-v1-prototype.html` | First inline dual-site prototype before standalone wireframe testing | Pre-lab baseline |
| **Draft 0b (Inline Wireframe)** | `protoui-v1-wireframe.html` | Monochrome wireframe with visible UX notes (rejected for text clutter, driving our minimalist reset) | Pre-lab baseline |
| **v1 (Wireframe Test)** | `wireframe-test/index.html`<br>`wireframe-test/server.mjs` | Zero-clutter standalone monochrome wireframe + Connect/Simulate entry screen + PostHog recorder | `research/usability-sessions/wireframe-session.jsonl` (`01`, `02`, `03`) |
| **v2 (Wootility Hi-Fi)** | `hifi-test/index.html`<br>`hifi-test/server.mjs` | Wootility v5 layout, Device Switcher dropdown, Tap+Hold paw buttons, tactile feedback & toasts | `research/usability-sessions/hifi-session-02.jsonl` (`04`) |
| **v3 (Final Reference)** | `final-test/index.html`<br>`final-test/server.mjs` | **Primary visual reference for v1.0:** Inline renaming + distinct Teensy 4.1 PCB Hardware Workbench for `builder.protostack.dev` (apply the 4 final micro-interaction rules in `05-final-session-03-signoff.md`) | `research/usability-sessions/final-session-03.jsonl` (`05`) |

## Analysis Charts (`docs/visuals/`)

- `docs/visuals/heuristics-and-architecture.html`: Nielsen's 10 Heuristics baseline vs target chart + 4-pillar architecture diagram
- `docs/visuals/usability-comparison.html`: Session 01 vs Session 02 quantitative telemetry improvement chart

## Running Any Standalone Test Site Locally

```bash
node prototypes/wireframe-test/server.mjs  # http://127.0.0.1:4173
node prototypes/hifi-test/server.mjs       # http://127.0.0.1:4174
node prototypes/final-test/server.mjs      # http://127.0.0.1:4175
```
