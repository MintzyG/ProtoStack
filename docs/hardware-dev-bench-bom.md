# ProtoStack Dev-Bench Hardware BOM (Brazil & USD Reference)

Doc ID: PS-HW-001
Last Updated: 2026-11-27
Reference rate: 1 USD ~ R$ 5.75 - 5.85 (prices vary with exchange rate, shipping, seller stock, and Brazil Remessa Conforme import taxes).

---

## 1. Phase 1: Starter Bench (Buy Now for v0.2 - v0.4)

This setup is just a **Teensy 4.1 + 8 pushbuttons on a breadboard**, powered straight from your PC's USB port (~100 mA). The Teensy runs the real Rust firmware, reads your physical buttons, saves settings to flash, and sends live visor state over USB WebHID so your browser renders the `128x32` LED display on your monitor.

You do **not** need an external 5V power supply or `74HCT245` level-shifter chips in Phase 1. Those are only needed when you add physical LED panels in Phase 2.

| # | Item | Spec / Mercado Livre Link | USD (Base) | BRL (Mercado Livre) | Notes |
|---|---|---|---:|---:|---|
| 1 | **Teensy 4.1 (with pins)** | [Teensy 4.1 com pinos (ML)](https://lista.mercadolivre.com.br/teensy-4.1-pinos) (PJRC / SparkFun `DEV-16996`) | $35.95 | R$ 400 - 700 | Make sure the listing says *com pinos soldados*. Buy from a well-rated seller and test on day 1 (press the board button and confirm HalfKay USB ID `16c0:0478` inside the return window). |
| 2 | **Breadboard (1660-pt), jumpers & 8 buttons** | [Protoboard 1660 pontos (ML)](https://lista.mercadolivre.com.br/protoboard-1660-pontos) + [Jumpers M-M / M-F (ML)](https://lista.mercadolivre.com.br/kit-jumpers-macho-femea-protoboard) + [Chave Tactil 6x6mm (ML)](https://lista.mercadolivre.com.br/chave-tactil-6x6) | $12 - 20 | R$ 75 - 115 | A metal-backed 1660-pt breadboard fits the Teensy on top, all 8 buttons (4L + 4R) on the bottom, won't slide when clicking, and leaves room for Phase 2 chips. (An 830-pt board also fits Phase 1 if you want to save ~R$ 30.) |
| 3 | **USB Micro-B data cable** | [Cabo USB Micro-B de dados (ML)](https://lista.mercadolivre.com.br/cabo-usb-micro-b-dados) | $3 - 6 | R$ 15 - 35 | Teensy 4.1 uses Micro-B. Skip if you already own a good data cable (avoid charge-only cables). |

**Phase 1 total (100% Mercado Livre):** R$ 490 - 850 (or R$ 475 - 815 if you already have a Micro-B data cable).

---

## 2. Phase 2: Physical LED Panels (Buy Later for v0.5)

Add these when you want real glowing LED panels on your desk instead of rendering on your monitor.

Notes on parts and stock (checked 2026-11-26):
- **HUB75 is not a separate box.** It is just the 16-pin ribbon cable connector on the back of the LED panels. Panel 1 chains to Panel 2 using the included flat cable.
- **Why 2x `SN74HCT245N` chips?** Teensy outputs 3.3V signals while HUB75 panels expect 5V. A HUB75 cable has 13 signal wires (`R1/G1/B1/R2/G2/B2`, `A/B/C/D`, `CLK/LAT/OE`). Each `SN74HCT245N` boosts 8 wires, so two chips sit side-by-side (16 channels total) on your breadboard. Panel 1 passes the boosted 5V signals straight into Panel 2.
- **SmartLED Shield V5 is discontinued** at both SparkFun (`DEV-17521`) and Adafruit (`4828`), and SparkFun also retired its `32x64` panel (`COM-14718`). Using `2x SN74HCT245N` on the breadboard does the exact same job with local parts and zero soldering.
- **AliExpress import taxes:** AliExpress prices below are base item prices before shipping and Remessa Conforme taxes (~44% up to US$ 50, ~92% above US$ 50).

| # | Item | Spec / Mercado Livre Link | USD (Base) | BRL (Mercado Livre) | BRL (AliExpress base, pre-tax) | Notes |
|---|---|---|---:|---:|---:|---|
| 4 | **2x RGB LED Matrix Panels** | [Painel LED RGB 64x32 HUB75 (ML)](https://lista.mercadolivre.com.br/painel-led-rgb-64x32-hub75) (2x `64x32` P3, 1/16 scan) | $24 - 44 (pair) | R$ 300 - 600 (pair) | R$ 130 - 240 (pair) | Must be **1/16 scan (16S)** and **HUB75** (avoid 1/8 scan, 1/4 scan, and HUB75E). Check that 16-pin ribbon cables and 5V power cables are included. |
| 5 | **2x Level Shifter Chips (3.3V -> 5V)** | [CI SN74HCT245N DIP-20 (ML)](https://lista.mercadolivre.com.br/sn74hct245n-dip) (`74AHCT245N` also OK) | $2 - 4 | R$ 15 - 30 | R$ 10 - 15 | Do not buy cheap blue 4-channel I2C MOSFET level shifters. They are too slow for HUB75 clocks. |
| 6 | **5V 5A Power Supply** | [Fonte 5V 5A P4 (ML)](https://lista.mercadolivre.com.br/fonte-5v-5a-p4) + [Adaptador P4 femea borne (ML)](https://lista.mercadolivre.com.br/adaptador-p4-femea-borne) (bivolt) | $7 - 12 | R$ 35 - 65 | R$ 30 - 50 | An enclosed 5V 5A wall-wart + P4 screw-terminal adapter is safer than an open metal *fonte colmeia* (no exposed mains wires). |
| 7 | **USB Logic Analyzer (optional)** | [Analisador Logico USB 24MHz 8ch (ML)](https://lista.mercadolivre.com.br/analisador-logico-usb-24mhz-8-canais) | $4 - 8 | R$ 35 - 90 | R$ 20 - 35 | Optional debug tool (works with Linux PulseView). Only needed if we hit a HUB75 timing issue in v0.5. |

**Phase 2 add-on total:** R$ 350 - 695 on Mercado Livre (or ~R$ 240 - 445 buying the 2 panels from AliExpress with taxes).

---

## 3. Phase 3: Helmet Sensors & Fan (v1.1 - Do Not Buy Yet)

| Item | Spec / Mercado Livre Link | Typical Cost Saved Now |
|---|---|---:|
| **Boop Sensor** | [APDS-9960 I2C (ML)](https://lista.mercadolivre.com.br/apds-9960) (3.3V proximity breakout; optional TTP223 touch) | $3 - 5 / R$ 30 - 45 |
| **Cooling Fan (5V PWM)** | [Noctua NF-A4x10 5V PWM (ML)](https://lista.mercadolivre.com.br/noctua-nf-a4x10-5v-pwm) (or generic 40mm 5V 4-wire PWM, never 12V) | $15 - 20 / R$ 150 - 220 |

---

## 4. Safety Notes

1. **Phase 1 (USB only):** Teensy 4.1 + 8 tactile buttons wired GPIO -> GND (internal pull-ups) is safe over PC USB. Verify the Teensy HalfKay bootloader (`16c0:0478`) on day 1 while inside the return window.
2. **Phase 2 - never power HUB75 panels from PC USB.** Two `64x32` panels can pull 3-5A at full brightness and trip or damage a USB port.
3. **Phase 2 - do not tie external 5V to Teensy `VIN` while on USB.** Keep external 5V connected only to the panels and `74HCT245` chips, sharing **GND only** with the Teensy (unless you intentionally cut the `VUSB-VIN` pad on the bottom of the Teensy).
4. **Phase 2 - check panel labels on arrival.** Confirm both panels say `64x32`, `16S` (1/16 scan), and use the same driver IC so they chain cleanly.
