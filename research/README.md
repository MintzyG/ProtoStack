# ProtoStack Research Folder

This folder holds our primary-source research, source-code teardowns of competing Protogen projects, and technical feasibility checks backing the design docs in `docs/`.

Every major decision in `docs/` should point back to concrete evidence here (real code, measured power numbers, or user testing notes) rather than guesswork.

## Research notes

- [01-prototracer-teardown.md](01-prototracer-teardown.md): Source review of Coela's ProtoTracer (Teensy 4.0/4.1, HUB75 SmartMatrix timing, FM6126A/RBG panel quirks, and proof that Teensy 4.x HalfKay flashing works over browser WebHID).
- [02-esp32-ecosystems-teardown.md](02-esp32-ecosystems-teardown.md): Source review of ProtoESP and LumiFur (real setup problems from `troublefaq.md`, sensor calibration fields, BLE remote battery draw, and ambient light scaling).
- [03-webhid-and-rust-teensy-feasibility.md](03-webhid-and-rust-teensy-feasibility.md): Reference study of VIA's WebHID client, `teensy4-rs`, and `imxrt-usbd` (Vendor Usage Page discovery, serialized HID command queues, software bootloader jump, and pure-Rust Teensy 4.1 USB stack).

## Cloned reference repos (`research/sources/`)

The third-party repos live locally inside `research/sources/` and are ignored in `.gitignore` so ProtoStack's own git history stays clean. Audited commits:

| Folder | Repo | Commit | Why we audited it |
|---|---|---:|---|
| `sources/ProtoTracer` | `https://github.com/coelacant1/ProtoTracer` | `15e8c3d` | Teensy 4.1 + HUB75 architecture and WebHID HalfKay uploader proof |
| `sources/ProtogenHelmet-ESP32` | `https://github.com/NCPlyn/ProtogenHelmet-ESP32` | `15419f5` | ESP32-S3 AP web UI, BLE remote, config schema, and user FAQ |
| `sources/LumiFur_Controller` | `https://github.com/stef1949/LumiFur_Controller` | `6693e35` | ESP32-S3 HUB75, NimBLE GATT, and Web Bluetooth OTA |
| `sources/teensy4-rs` | `https://github.com/mciantyre/teensy4-rs` | `f16c826` | Rust Teensy 4.x BSP, `imxrt-hal`, and RTIC 2 setup |
| `sources/imxrt-usbd` | `https://github.com/imxrt-rs/imxrt-usbd` | `88e9c97` | Pure-Rust i.MX RT1062 USB device driver |
| `sources/via-app` | `https://github.com/the-via/app` | `75a7275` | Production browser WebHID configurator patterns |
