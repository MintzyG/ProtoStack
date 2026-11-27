# ProtoSpeak v1: USB, Intra-Suit, and Wireless Protocol Spec

Doc ID: PS-PROTO-003
Crate: `crates/proto-speak` (`#![no_std]`, compiles for MCUs, Linux, and WASM)
Status: Current (living specification — synced with ADR-001 & ADR-002)

## 1. Overview

`ProtoSpeak` is a single compact binary protocol shared across all ProtoStack components. Every message uses `postcard` (compact serde encoding) plus a CRC-16 check (or authentication tag on encrypted wireless links).

It covers three communication domains:

1. **Control (Domain 0):** Host-to-device configuration over USB WebHID (and WebBLE later) between `ProtoUI` and `ProtoBrain`.
2. **Suit Mesh (Domain 1):** Encrypted, low-latency communication between parts of one suit (head, tail, ears, chestplate, paw buttons, handler fob). Works over a wired bus (UART/CAN) or wirelessly (ESP-NOW, BLE, nRF24/LoRa) with no Wi-Fi router needed.
3. **Social Mesh (Domain 2):** Opt-in, short-range proximity beacons between different Protogens at conventions.

Only Domain 0 needs to be implemented on hardware in v0.1, while the header already reserves the fields for Domains 1 and 2 so the wire format does not have to break later.

## 2. Packet format

Every `ProtoSpeak` packet uses the same 12-byte header regardless of transport:

```text
Byte 0:     [Ver (4 bits) | Domain (4 bits)]
Byte 1:     Flags (ACK_REQ, IS_ACK, ENCRYPTED, FRAGMENTED, PRIORITY_SAFETY)
Bytes 2-3:  Opcode (u16, little-endian)
Bytes 4-5:  Source NodeId (u16; 0x0000 = Host, 0x0001 = Head, 0xFFFF = Broadcast)
Bytes 6-7:  Dest NodeId (u16)
Bytes 8-9:  Sequence number (u16)
Bytes 10-11: Payload length (u16)
Bytes 12..: Postcard-encoded payload
Last 2-4 B: CRC-16 (or 4-byte auth MIC on encrypted Suit Mesh frames)
```

### Transport framing

- **USB WebHID (v0.1 primary):** 64-byte HID reports. Report ID `0x01`, followed by `[chunk_index, total_chunks, 61 bytes of payload]`. Typical round-trip latency is 1 to 4 ms.
- **Wired intra-suit (UART / RS-485 / CAN-FD):** COBS framing over UART, or native CAN-FD frames. Under 2 ms latency, lets parts share battery power and puts radios to sleep.
- **ESP-NOW (ESP32 native or Teensy UART bridge to an ESP32-C3):** 250-byte frames with no Wi-Fi AP or router needed. Roughly 2 to 8 ms latency.
- **BLE Advertising / GATT:** Used for phone companion control, paw fobs, and social beacons. Roughly 15 to 40 ms latency.
- **nRF24L01+ / SX1262 LoRa:** Optional SPI radio option for Teensy builds without an ESP32 co-processor.

## 3. Control opcodes (ProtoUI to ProtoBrain)

The device describes its own hardware on connect so the website does not need hardcoded board lists.

| Opcode | Name | Direction | Target | Purpose |
|---|---|---|---|---|
| `0x0001` | `HELLO_INFO` | Both | `v0.1` | Firmware version, MCU type, preset ID, protocol version, and capability flags. |
| `0x0002` | `GET_MANIFEST` | Both | `v0.2` | Reads the active hardware manifest (display size, panel layout, driver type, color order, buttons, current limit). |
| `0x0003` | `SET_MANIFEST` | Host to Dev | `v0.4` | Updates hardware mapping (from `/builder` view) in RAM or flash. |
| `0x0010` | `PREVIEW_STATE` | Host to Dev | `v0.1` | Live volatile preview in RAM: switch face, tint, brightness, or push a test frame with zero flash wear. |
| `0x0011` | `PROFILE_UPLOAD` | Host to Dev | `v0.4` | Chunked upload of face assets, animations, and button trigger settings. |
| `0x0012` | `COMMIT_FLASH` | Host to Dev | `v0.3` | Atomically saves RAM config to flash using A/B ping-pong slots so power loss cannot corrupt it. |
| `0x0013` | `SNAPSHOT_CTRL` | Host to Dev | `v0.3` | Save or restore the Factory Snapshot (right-to-repair safe rollback). |
| `0x0020` | `TELEMETRY_SUB` | Both | `v0.2` | Live stream of active expression, physical paw button states, estimated LED mA, and FPS (adds boop/temp/fan in `v1.1`). |
| `0x0030` | `PANEL_IDENTIFY` | Host to Dev | `v0.4` / `v0.5` | Draws an orientation arrow and panel index on Panel N so builders can verify wiring visually. |
| `0x0031` | `SENSOR_SCAN` | Both | `v1.1` | Scans I2C/SPI/ADC buses and reports detected chips (APDS9960, BNO055, INA219, SSD1306). |
| `0x0032` | `CALIBRATE_POSE` | Host to Dev | `v1.1+` | Saves the current 3-axis IMU reading as Neutral, Tilt, or Look Up without manual coordinate editing. |
| `0x00F0` | `GET_FAULT_LOG` | Both | `v1.0` | Reads the last 8 brownout, watchdog, or crash entries from retained memory. |
| `0x00FE` | `BOOTLOADER_JUMP` | Host to Dev | `v0.4` | Blanks LEDs safely and reboots into the USB bootloader (Teensy HalfKay via `bkpt #251`) for browser firmware updates. |

## 4. Intra-Suit Modular Bus and Mesh (Domain 1: v1.2 Wired -> v1.3 Wireless)

We roll out multi-piece suits in two deliberate steps so we prove plug-and-play modularity over a reliable wire before adding radio complexity:

### Step 1: Wired Modular Suit and Custom `ProtoParts` (v1.2)
- Peripheral nodes (tails, wings, ears, chest panels, body LEDs, paw pads, extra sensor pods, or custom props) plug into the head over a wired `ProtoSpeak` bus (UART / RS-485 / CAN-FD, under 2 ms latency, shared battery power).
- **Self-describing `ProtoPart` registration (hardware plugin model):** When a node powers up on the bus, it sends `HELLO_INFO` plus its `ProtoPart` descriptor:
  - `part_id`, `display_name`, and `kind` (`Visor`, `LightZone`, `Sensor`, `ButtonBank`, `Fan`, or `Custom`).
  - `events[]`: named typed signals it can emit (for example, `ButtonPressed(u8)`, `MotionTriggered(u8)`, `SensorActive(bool)`).
  - `actions[]`: named typed commands it accepts (for example, `SetColor(rgb)`, `TriggerEffect(u8)`, `SetPosition(u8)`).
  - `controls[]`: optional simple UI metadata (toggle, slider, color picker, dropdown) so `ProtoUI` can render a control card for any custom part it has never seen before.
- The head registers the part in its `PartRegistry` and exposes its events and actions to `ProtoUI`, letting users wire up cross-part rules (like syncing chest panels, wings, body LEDs, or a tail with visor faces and boops) with zero firmware edits or code plugins.

### Step 2: Wireless Modular Suit and Autonomous Fallback (v1.3)
- Runs the exact same Domain 1 node registration and event messages over wireless PHYs (ESP-NOW, BLE, or nRF24/LoRa).
- **Pairing and security:** Each suit generates a random 128-bit key (`SuitPSKey`) in `ProtoUI` and stores it on every node in that suit. Wireless suit messages are encrypted and authenticated (ChaCha20-Poly1305 or AES-CCM with a counter) so nearby Protogens cannot trigger your tail or fans.
- **Autonomous fallback ("head left the room" rule):** Every wireless peripheral runs a small `ProtoBrain-Lite` loop with three states:
  1. `SYNCED`: follows heartbeat/state messages from the highest-priority coordinator (Wired Head first, then Wireless Head, then Phone or Paw Fob).
  2. `AUTONOMOUS`: if no coordinator heartbeat arrives for 3 seconds, smoothly crossfades over 500 ms to its own saved standalone animation (never goes black).
  3. `REJOINING`: as soon as the head or phone comes back into range (or plugs back in), it crossfades smoothly back into sync.
- **Wired preference:** If a node sees a live wired bus (UART/CAN), it uses the wire and sleeps its radio to save battery. If unplugged, it wakes the radio automatically.
- **Handler fob (`PRIORITY_SAFETY`):** Receives a short telemetry ping every 5 seconds (battery, head temp, fan speed) and has a physical button to force fans to 100% and dim the visor in an emergency.

## 5. Proto-to-Proto Social Mesh (Domain 2, v2.0)

Opt-in proximity interactions between different Protogens at conventions, designed to stay private and non-annoying:

- **Opt-in modes:** Off, Friends Only, or Public Con Mode.
- **Beacon format:** A tiny 24-byte unencrypted broadcast every 2 seconds containing a rotating session ID (changes every 15 minutes unless paired as friends, to prevent tracking), optional short display name, primary color, and mood tag.
- **Anti-spam rules:**
  - Signal strength (RSSI) gate so only Protogens within roughly 1.5 to 3 meters trigger a reaction.
  - Per-peer and global cooldown (default 15 seconds).
  - Lowest priority: social reactions never interrupt speaking/lip-sync, button presses, or low-battery/thermal alerts.
  - Quick mute: double-tapping a hidden touch/chin button toggles Do Not Disturb right away.
