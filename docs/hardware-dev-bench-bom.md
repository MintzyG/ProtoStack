# ProtoStack Dev-Bench Hardware BOM (Brazil & USD Reference)

Doc ID: PS-HW-001
Last Updated: 2026-11-27
Reference Exchange Rate: **1 USD ≈ R$ 5.75 – R$ 5.85** *(prices are estimates and fluctuate with exchange rate, seller stock, shipping, and Brazil's Remessa Conforme import taxes).*

---

## 1. Phase 1 — Starter Bench Cart (Buy Now: `v0.2` – `v0.4`)

**Setup:** Teensy 4.1 + 8 tactile paw buttons on a breadboard, powered **100% safely off your PC's USB port** (~100 mA draw). The Teensy runs real bare-metal Rust firmware (`proto-brain` + `proto-speak` + real USB WebHID), reads your physical paw buttons, saves configs to flash, and mirrors the live `128x32` LED visor onto your computer monitor in **ProtoUI**.

> **No external 5V power supply or `74HCT245` level-shifter chips are needed in Phase 1**—those are only required once you add physical LED panels in Phase 2!

| # | Status | Component | Exact Spec / Mercado Livre Link | USD Estimate (Base) | BRL Estimate (Mercado Livre / Local BR) | Notes |
|---|---|---|---|---:|---:|---|
| 1 | **Buy Now** | **Teensy 4.1 (with Pins)** | [Teensy 4.1 com pinos (ML)](https://lista.mercadolivre.com.br/teensy-4.1-pinos) *(PJRC / SparkFun `DEV-16996`)* | **$35.95** *(SparkFun / DigiKey)* | **R$ 400 – R$ 700** | Buy from a well-rated ML seller (MercadoLíder / Full) or SparkFun/DigiKey. Verify on arrival within the return window by pressing the board button and confirming USB HalfKay bootloader (`16c0:0478`). |
| 2 | **Buy Now** | **Breadboard (1660-pt), Jumpers & 8× Paw Buttons** | [Protoboard 1660 pontos (ML)](https://lista.mercadolivre.com.br/protoboard-1660-pontos) + [Jumpers M-M / M-F (ML)](https://lista.mercadolivre.com.br/kit-jumpers-macho-femea-protoboard) + [Chave Táctil 6×6mm (ML)](https://lista.mercadolivre.com.br/chave-tactil-6x6) | **$12 – $20** | **R$ 75 – R$ 115** *(board + jumpers + 10 buttons)* | One solid metal-backed 1660-pt breadboard holds the Teensy 4.1 on top, all **8 paw buttons** (4L + 4R) cleanly on the bottom, won't slide when clicking, and already has room left for Phase 2 level-shifter chips later. *(An 830-pt board also fits Phase 1 if you want to save ~R$ 30, though Phase 2 chips will want the 1660-pt.)* |
| 3 | **Buy Now** | **USB Micro-B *Data* Cable** | [Cabo USB Micro-B de dados (ML)](https://lista.mercadolivre.com.br/cabo-usb-micro-b-dados) | **$3 – $6** | **R$ 15 – R$ 35** | Teensy 4.1 uses **Micro-B**. Avoid charge-only cables. *(R$ 0 if you already own a known-good data cable.)* |

### Phase 1 Total Estimate
- **100% Mercado Livre:** **R$ 490 – R$ 850 BRL** *(~R$ 475 – R$ 815 if you already own a Micro-B data cable)*

---

## 2. Phase 2 — Physical LED Visor Add-On (Buy Later: `v0.5`)

Add these when you're ready to move the visor off your computer monitor and onto real glowing LED panels.

> **Store Stock Notes (Verified 2026-11-26):**
> - **SparkFun** retired both its `32x64` panel (`COM-14718`) and SmartLED Shield V5 (`DEV-17521`).
> - **Adafruit** discontinued Teensy boards and SmartLED Shield V5 (`4828`); they still sell `64x32` P3 panels (`2279`), but at ~$44.95 *each* (2–3× AliExpress pricing).
> - **Why `2× SN74HCT245N` chips?** A HUB75 cable carries **13 signal wires** (`R1/G1/B1/R2/G2/B2`, `A/B/C/D`, `CLK/LAT/OE`) that need 3.3V → 5V boosting. Each `SN74HCT245N` boosts 8 channels, so 2 chips sit side-by-side (`16` channels) on your 1660-pt breadboard—the exact same two chips that sat on the retired SmartLED Shield V5. Panel 1 passes the boosted 5V signals directly to Panel 2 via the included flat ribbon cable.
> - **Brazil Import Note (Remessa Conforme):** AliExpress prices below show the **base price before shipping and Brazilian import taxes** (~44% combined II + ICMS up to US$ 50, ~92% above US$ 50).

| # | Status | Component | Exact Spec / Mercado Livre Link | USD Estimate (Base) | BRL Estimate (Mercado Livre / Local BR) | BRL Estimate (AliExpress Base, pre-tax) | Notes |
|---|---|---|---|---:|---:|---:|---|
| 4 | **Phase 2** | **2× RGB LED Matrix Panels** | [Painel LED RGB 64x32 HUB75 (ML)](https://lista.mercadolivre.com.br/painel-led-rgb-64x32-hub75) *(2× `64x32` P3, 1/16 scan)* | **$24 – $44** *(pair)* | **R$ 300 – R$ 600** *(pair)* | **R$ 130 – R$ 240** *(pair)* | **Critical:** Confirm **1/16 scan (16S)** and **HUB75** (avoid 1/8 scan, 1/4 scan, and HUB75E). Check that 16-pin IDC ribbon cables & 5V power cables are included. |
| 5 | **Phase 2** | **Level Shifter Chips (3.3V → 5V)** | [CI SN74HCT245N DIP-20 (ML)](https://lista.mercadolivre.com.br/sn74hct245n-dip) *(2× pcs; `74AHCT245N` also OK)* | **$2 – $4** | **R$ 15 – R$ 30** *(2–5 pcs)* | **R$ 10 – R$ 15** | **Do NOT buy cheap blue 4-channel I2C MOSFET level-shifter boards**—they are too slow for HUB75 clocks. Always use `74HCT245` / `74AHCT245`. |
| 6 | **Phase 2** | **5V Power Supply (Bench)** | [Fonte 5V 5A P4 (ML)](https://lista.mercadolivre.com.br/fonte-5v-5a-p4) + [Adaptador P4 fêmea borne (ML)](https://lista.mercadolivre.com.br/adaptador-p4-femea-borne) *(bivolt)* | **$7 – $12** | **R$ 35 – R$ 65** | **R$ 30 – R$ 50** | Enclosed 5V 5A wall-wart + P4-to-screw-terminal adapter is much safer than an open metal *fonte colmeia* (no exposed 110V/220V wires). |
| 7 | *Optional* | **USB Logic Analyzer *(Debug Tool)*** | [Analisador Lógico USB 24MHz 8ch (ML)](https://lista.mercadolivre.com.br/analisador-logico-usb-24mhz-8-canais) *(works with Linux PulseView)* | **$4 – $8** | **R$ 35 – R$ 90** | **R$ 20 – R$ 35** | Optional insurance only if we hit a HUB75 signal timing issue during `v0.5` bring-up. |

### Phase 2 Add-On Estimate
- **100% Mercado Livre:** **R$ 350 – R$ 695 BRL** *(or ~**R$ 240 – R$ 445 BRL** if ordering the 2 panels from AliExpress with Remessa Conforme taxes)*

---

## 3. Phase 3 — Helmet Sensors & Comfort (`v1.1`, Do NOT Buy Yet)

Helmet-shell add-ons deferred out of `v1.0`:

| Component | Spec / Mercado Livre Link (For Later) | Typical Cost Saved Now |
|---|---|---:|
| **Boop Sensor** | [APDS-9960 I2C (ML)](https://lista.mercadolivre.com.br/apds-9960) *(3.3V proximity breakout; + optional TTP223 touch)* | ~$3–$5 / **R$ 30–45** |
| **Cooling Fan (5V PWM)** | [Noctua NF-A4x10 5V PWM (ML)](https://lista.mercadolivre.com.br/noctua-nf-a4x10-5v-pwm) *(or generic 40mm 5V 4-wire PWM—never 12V)* | ~$15–$20 / **R$ 150–220** |

---

## 4. Essential Bench Safety Notes

1. **Phase 1 (USB-Only):** Teensy 4.1 + 8 tactile buttons wired GPIO → `GND` (using internal pull-ups) is 100% safe over PC USB. Verify your Teensy 4.1 on day 1 by pressing the onboard button and confirming PJRC HalfKay (`16c0:0478`) while inside your marketplace return window.
2. **Phase 2 (When Adding HUB75 Panels) — Never power HUB75 panels from PC USB.** Dual `64x32` panels can draw 3–5A at full brightness and will trip or damage a PC USB port.
3. **Phase 2 — Prevent 5V back-feeding into USB:** Keep the external 5V panel power rail **completely disconnected** from the Teensy's `VIN` / `5V` pin while plugged into USB—connect **GND only** between the Teensy and the external 5V supply! *(Only tie them together if you intentionally cut the Teensy 4.1 `VUSB-VIN` solder bridge on the bottom of the board.)*
4. **Phase 2 — Verify HUB75 scan rate on arrival:** Check the silkscreen on the back of both panels for `64x32`, `16S` (or `1/16`), and identical driver IC labels so Panel 1 and Panel 2 match cleanly when daisy-chained.
