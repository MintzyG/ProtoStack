# ProtoStack Usability Lab Archive

Complete chronological archive of every prototype HTML tested, every PostHog-style `.jsonl` telemetry recording, and every pre/post interview and analysis report.

## Chronological Session Index

| Stage | Tested HTML Artifact | Telemetry Recording | Interview & Analysis Docs | Key Outcome |
|---|---|---|---|---|
| **Pre-Lab Drafts (Inline)** | `../../prototypes/protoui-v1-wireframe.html`<br>`../../prototypes/protoui-v1-prototype.html` | None (inline review) | Captured in `01-wireframe-pre-interview.md` | Caught severe text clutter from inline UX notes/helper text; led to zero-clutter standalone browser tests. |
| **Session 01 (Wireframe)** | `../../prototypes/wireframe-test/index.html` (`server.mjs`, port 4173) | `wireframe-session.jsonl` (220 events, 4m 30s) | `01-wireframe-pre-interview.md`<br>`02-wireframe-telemetry-analysis.md`<br>`03-wireframe-post-interview-and-wootility-study.md` | Found 4 rage clicks, 10 dead clicks, accidental top-bar disconnect, missing Tap+Hold on paw buttons, and Config/Builder confusion. Audited real Wootility v5.4.3 (`wootility.io`). Ease: `3/5`. |
| **Session 02 (Wootility Hi-Fi)** | `../../prototypes/hifi-test/index.html` (`server.mjs`, port 4174) | `hifi-session-02.jsonl` (127 events, 4m 12s) | `04-hifi-session-02-report.md`<br>`../../docs/visuals/usability-comparison.html` | Rage clicks `4 -> 0`, dead clicks `10 -> 1`, accidental disconnects `1 -> 0`, connect time `12.1s -> 5.3s`. Identified need for distinct Builder layout and custom renaming. Ease: `4/5`. |
| **Session 03 (Final PCB + Rename)** | `../../prototypes/final-test/index.html` (`server.mjs`, port 4175) | `final-session-03.jsonl` (92 events, 3m 15s) | `05-final-session-03-signoff.md` | Connect time `2.8s`, 0 rage clicks, Builder PCB Workbench completely distinct from Config. Locked final 4 micro-interaction rules (pencil icons on cards + main bar, helmet rename in device menu, continuous boop threshold with hysteresis, click-to-assign Custom Teensy pins). |
