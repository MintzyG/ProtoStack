# Feasibility Study: WebHID Configurator and Rust on Teensy 4.1

Repos audited:
- `research/sources/via-app` (`https://github.com/the-via/app`, commit `75a7275`)
- `research/sources/teensy4-rs` (`https://github.com/mciantyre/teensy4-rs`, commit `f16c826`)
- `research/sources/imxrt-usbd` (`https://github.com/imxrt-rs/imxrt-usbd`, commit `88e9c97`)

## 1. WebHID patterns proven by VIA (`the-via/app`)

Reading VIA's WebHID implementation (`src/shims/node-hid.ts` and `src/utils/keyboard-api.ts`) gives us five concrete engineering rules for `ProtoUI`:

1. **Filter by Vendor Usage Page, not just VID/PID (`node-hid.ts`, lines 19-25, 73-81):**
   VIA requests devices matching `usagePage: 0xFF60` and `usage: 0x61`. Doing the same in ProtoStack means any custom community PCB running our firmware shows up automatically in the web app without updating a hardcoded USB ID list.
2. **Serialize outgoing HID commands (`keyboard-api.ts`, lines 185-196):**
   Async browser HID calls will collide if a user drags two sliders or clicks quickly. VIA uses a per-device FIFO `globalCommandQueue`. `proto-ui-core` must serialize control requests with sequence numbers and throttle rapid slider drags (for example, 30 Hz max).
3. **Separate unsolicited push reports from request/response (`node-hid.ts`, lines 148-160):**
   Incoming HID reports first pass through push listeners (for live telemetry or device-initiated UI sync) before resolving pending request waiters. This lets 20 Hz sensor telemetry share the same HID endpoint as config commands.
4. **Software bootloader jump (`keyboard-api.ts`, line 33):**
   VIA exposes `BOOTLOADER_JUMP = 0x0b` so the browser can reboot firmware straight into flashing mode. On Teensy 4.1, running `bkpt #251` enters HalfKay bootloader mode (`0x16C0:0x0479`), letting the browser flash updates without opening the helmet.
5. **Handle missing OS paths (`node-hid.ts`, lines 39-68):**
   WebHID does not expose OS device paths, so the web client tags each `HIDDevice` with a session UUID and exposes `device.forget()` to let users reset browser permissions cleanly.

## 2. Pure-Rust Teensy 4.1 stack (`teensy4-rs` and `imxrt-usbd`)

Inspecting `teensy4-rs/Cargo.toml` (lines 70-109) and `examples/rtic_usb_log.rs` confirms the Rust Teensy 4.1 ecosystem is active, on Rust 2024 edition, and no longer depends on C USB code:

- `teensy4-bsp = "0.6.0"` (target `thumbv7em-none-eabihf`, MIT/Apache-2.0)
- `imxrt-hal = "0.6.0"` (`imxrt1060` feature: FlexIO, FlexPWM, eDMA, LPI2C, LPSPI, LPUART, ADC, timers)
- `imxrt-usbd = "0.4.0"` + `usb-device = "0.3"` (native Rust USB bus driver, compatible with `usbd-hid` for 64-byte Vendor HID reports)
- `rtic = "2"` (`thumbv7-backend`): clean interrupt priorities so HUB75 DMA refresh gets highest priority, `USB_OTG1` (`rtic_usb_log.rs`, lines 115-118) runs right below it, and `ProtoBrain` frame rendering + sensor polling run smoothly at 60 Hz / 20 Hz.

## 3. Known risks and how we handle them

1. **No drop-in Rust `SmartMatrix` crate for Teensy 4.1 HUB75:**
   Writing the FlexIO + FlexPWM + eDMA HUB75 driver in `proto-bsp-teensy41` is the single hardest embedded piece of v0.1. Mitigation: port the exact verified timings from SmartLED Shield V5 (`research/01-prototracer-teardown.md`) and test with a simple color-bar and `PANEL_IDENTIFY` pattern before layering on anything else.
2. **Linux WebHID udev permissions:**
   Chrome on Linux blocks non-root WebHID access until a udev rule is installed. Mitigation: detect permission errors in the connection helper and show a single copy-paste terminal command to add the rule.
3. **No WebHID in Firefox or Safari:**
   WebHID is Chromium-only today. Mitigation: open the site in Live Simulator Mode automatically so anyone can try it, and show a plain message explaining that USB connection requires Chrome, Edge, or Brave.
