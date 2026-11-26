# ProtoStack Dev-Bench Hardware BOM (Brazil & USD Reference)

Doc ID: PS-HW-001
Last Updated: 2026-11-26
Reference Exchange Rate: **1 USD ≈ R$ 5.75 – R$ 5.85** *(prices are estimates and fluctuate with exchange rate, seller stock, shipping, and Brazil's Remessa Conforme import taxes).*

---

## 1. Plain-English Hardware Primer

### What is HUB75?
**HUB75** is **not** a separate box you buy—it is the standard **16-pin ribbon cable plug and wiring format** already built into the back of full-color `64x32` RGB LED matrix panels (think of it like "HDMI" for LED panels). Two `64x32` panels chain together directly (`Panel 1 OUT` → `Panel 2 IN`) with the included flat ribbon cable to form a `128x32` Protogen visor.

### Why `2× SN74HCT245N` Chips + a Breadboard?
The Teensy 4.1 speaks **3.3V** logic, while HUB75 LED panels expect **5V** logic signals. Without 3.3V → 5V level shifting, panels can flicker, glitch, or stay blank.

- **Why 2 chips instead of 1?** A HUB75 cable carries **13 signal wires** (`6` colors `R1/G1/B1/R2/G2/B2` + `4` row lines `A/B/C/D` + `3` control lines `CLK/LAT/OE`). Each `SN74HCT245N` chip boosts **8 channels**, so **2 chips sit side-by-side** (`2 × 8 = 16` channels) to cover all 13 wires. *(You do **not** chain them—Chip 1 handles `R1/G1/B1/R2/G2/B2/CLK/LAT` and Chip 2 handles `A/B/C/D/OE`. Panel 1 then passes the boosted 5V signals straight through to Panel 2.)*
- **Why a breadboard?** Each `SN74HCT245N` is a bare 20-leg DIP chip—you can't plug jumper wires directly onto bare legs. A **1660-point breadboard** (with a sturdy metal base plate) lets you push in the pre-pinned Teensy 4.1, both chips, and all **8 tactile paw buttons** (4 Left + 4 Right) with **zero soldering**, and gives you tidy shared `5V` and `GND` rails.
- **What happened to SmartLED Shield V5?** Pixelmatix's **SmartLED Shield V5** (`DEV-17521` at SparkFun / `4828` at Adafruit)—which is literally just two `74HCT245` chips pre-soldered onto a carrier PCB—has been **retired/discontinued** at both SparkFun and Adafruit. *(Its PCB design is open-source on GitHub if you ever want to order custom boards from JLCPCB later, but for a tabletop bench, the breadboard + `2× SN74HCT245N` setup is cheaper, locally available in Brazil, and requires zero soldering.)*

> **Pre-pinned Teensy 4.1 compatibility:** A "Teensy 4.1 with Pins" (*com pinos soldados*) has **male headers pointing down**, so it plugs straight into a standard breadboard with no soldering required.

---

## 2. Lean Tabletop Dev-Bench Cart (`v0.2` – `v1.0`)

Because early development is a **tabletop bench setup** (no helmet shell yet), only the Teensy 4.1, two `64x32` HUB75 panels, level-shifting breadboard kit, 5V power supply, and paw buttons are required for `v1.0`. Boop sensors and cooling fans are deferred to `v1.1` (see Section 3).

> **Store Stock Notes (Verified 2026-11-26):**
> - **SparkFun** actively stocks Teensy 4.1 with Pins (`DEV-16996`), but **retired** both its `32x64` panel (`COM-14718`) and SmartLED Shield V5 (`DEV-17521`).
> - **Adafruit** discontinued Teensy boards and SmartLED Shield V5 (`4828`); they still sell `64x32` P3 panels (`2279`), but at ~$44.95 *each* (2–3× AliExpress pricing).
> - **Brazil Import Note (Remessa Conforme):** AliExpress prices below show the **base item price before shipping and Brazilian import taxes** (~44% combined II + ICMS up to US$ 50, ~92% above US$ 50). Mercado Livre prices are typical local BR listings.

| # | Status | Component | Exact Spec / Mercado Livre Link | USD Estimate (Base) | BRL Estimate (Mercado Livre / Local BR) | BRL Estimate (AliExpress Base, pre-tax) | Where to Buy / Notes |
|---|---|---|---|---:|---:|---:|---|
| 1 | **Need** | **Teensy 4.1 (with Pins)** | [Teensy 4.1 com pinos (ML)](https://lista.mercadolivre.com.br/teensy-4.1-pinos) *(PJRC / SparkFun `DEV-16996`)* | **$35.95** *(SparkFun / DigiKey)* | **R$ 400 – R$ 700** | *(Avoid)* | Buy from a well-rated ML seller (MercadoLíder / Full) or SparkFun/DigiKey. Verify on arrival within the return window by pressing the board button and confirming USB HalfKay bootloader (`16c0:0478`). |
| 2 | **Need** | **2× RGB LED Matrix Panels** | [Painel LED RGB 64x32 HUB75 (ML)](https://lista.mercadolivre.com.br/painel-led-rgb-64x32-hub75) *(2× `64x32` P3, 1/16 scan)* | **$24 – $44** *(pair)* | **R$ 300 – R$ 600** *(pair)* | **R$ 130 – R$ 240** *(pair)* | **Critical:** Confirm **1/16 scan (16S)** and **HUB75** (avoid 1/8 scan, 1/4 scan, and HUB75E). Check that each panel includes its 16-pin IDC ribbon cable & 5V power cable. |
| 3 | **Need** | **Level Shifter Chips (3.3V → 5V)** | [CI SN74HCT245N DIP-20 (ML)](https://lista.mercadolivre.com.br/sn74hct245n-dip) *(2× pcs; `74AHCT245N` also OK)* | **$2 – $4** | **R$ 15 – R$ 30** *(2–5 pcs)* | **R$ 10 – R$ 15** | **Do NOT buy cheap blue 4-channel I2C MOSFET level-shifter boards**—they are too slow for HUB75 clocks. Always use `74HCT245` / `74AHCT245`. |
| 4 | **Need** | **Breadboard (1660-pt), Jumpers & 8× Paw Buttons** | [Protoboard 1660 pontos (ML)](https://lista.mercadolivre.com.br/protoboard-1660-pontos) + [Jumpers M-M / M-F (ML)](https://lista.mercadolivre.com.br/kit-jumpers-macho-femea-protoboard) + [Chave Táctil 6×6mm (ML)](https://lista.mercadolivre.com.br/chave-tactil-6x6) | **$12 – $20** | **R$ 75 – R$ 115** *(board + jumpers + buttons)* | **R$ 45 – R$ 75** | Buy locally on Mercado Livre. One solid metal-backed 1660-pt board holds Teensy 4.1 + both `74HCT245` chips on the top half and all **8 paw buttons** (4L + 4R) cleanly on the bottom half—zero soldering, won't slide when clicking. |
| 5 | **Need** | **5V Power Supply (Bench)** | [Fonte 5V 5A P4 (ML)](https://lista.mercadolivre.com.br/fonte-5v-5a-p4) + [Adaptador P4 fêmea borne (ML)](https://lista.mercadolivre.com.br/adaptador-p4-femea-borne) *(bivolt)* | **$7 – $12** | **R$ 35 – R$ 65** | **R$ 30 – R$ 50** | Buy locally on Mercado Livre. An enclosed 5V 5A wall-wart + P4-to-screw-terminal adapter is much safer for beginners than an open metal *fonte colmeia* (no exposed 110V/220V wall wires). |
| 6 | **Need** | **USB Micro-B *Data* Cable** | [Cabo USB Micro-B de dados (ML)](https://lista.mercadolivre.com.br/cabo-usb-micro-b-dados) | **$3 – $6** | **R$ 15 – R$ 35** | **R$ 10 – R$ 20** | Teensy 4.1 uses **Micro-B**. Avoid charge-only cables. *(Skip if you already own a known-good data cable.)* |
| 7 | *Optional* | **USB Logic Analyzer *(Debug Tool)*** | [Analisador Lógico USB 24MHz 8ch (ML)](https://lista.mercadolivre.com.br/analisador-logico-usb-24mhz-8-canais) *(works with Linux PulseView)* | **$4 – $8** | **R$ 35 – R$ 90** | **R$ 20 – R$ 35** | Not needed to run ProtoStack—optional insurance only if we hit a HUB75 signal timing issue during `v0.2` bring-up. Fine to skip for now. |

---

## 3. Deferred to `v1.1` (Do NOT Buy Yet)

These are helmet-shell add-ons deferred out of `v1.0` to keep the tabletop bench lean and affordable:

| Component | Spec / Mercado Livre Link (For Later) | Typical Cost Saved Now |
|---|---|---:|
| **Boop Sensor** | [APDS-9960 I2C (ML)](https://lista.mercadolivre.com.br/apds-9960) *(3.3V proximity breakout; + optional TTP223 touch)* | ~$3–$5 / **R$ 30–45** |
| **Cooling Fan (5V PWM)** | [Noctua NF-A4x10 5V PWM (ML)](https://lista.mercadolivre.com.br/noctua-nf-a4x10-5v-pwm) *(or generic 40mm 5V 4-wire PWM—never 12V)* | ~$15–$20 / **R$ 150–220** |

---

## 4. Rough Total Budget Estimate (`v1.0` Tabletop Cart)

- **100% Mercado Livre (Fastest, All-Local Brazil):** ~**R$ 810 – R$ 1,500 BRL** *(mostly driven by local ML pricing for Teensy 4.1 + 2× panels)*
- **Hybrid Mix (AliExpress 2× panels w/ Remessa Conforme + Local ML for Teensy, chips, breadboard & PSU):** ~**R$ 680 – R$ 1,100 BRL** *(~$75 – $115 USD base equivalent)*

---

## 5. Essential Bench Safety Notes (Read Before Plugging In!)

1. **Never power HUB75 panels from your computer's USB port.** Dual `64x32` panels can draw 3–5A at full brightness and will trip or damage a PC USB port.
2. **Prevent 5V back-feeding into USB:** Keep the external 5V panel power rail **completely disconnected** from the Teensy's `VIN` / `5V` pin while plugged into USB—connect **GND only** between the Teensy and the external 5V supply! *(Only tie them together if you intentionally cut the Teensy 4.1 `VUSB-VIN` solder bridge on the bottom of the board.)*
3. **Verify your Teensy 4.1 on day 1:** Plug it into USB and press the onboard button; confirm it enters PJRC's HalfKay bootloader (`16c0:0478`) while still inside your marketplace return window.
4. **Verify HUB75 scan rate on arrival:** Check the silkscreen on the back of both panels for `64x32`, `16S` (or `1/16`), and identical driver IC labels so Panel 1 and Panel 2 match cleanly when daisy-chained.
