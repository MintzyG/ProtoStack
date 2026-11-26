# ProtoStack Dev-Bench Hardware BOM (Brazil & USD Reference)

Doc ID: PS-HW-001
Last Updated: 2026-11-26
Reference Exchange Rate: **1 USD ≈ R$ 5.75 – R$ 5.85** *(prices are estimates and fluctuate with exchange rate, seller stock, shipping, and Brazil's Remessa Conforme import taxes).*

---

## 1. Plain-English Hardware Primer

### What is HUB75?
**HUB75** is the standard **16-pin ribbon cable connector and wiring format** on the back of full-color `64x32` RGB LED matrix panels. When people say "two HUB75 panels," they mean two `64x32` RGB LED display boards that plug together via HUB75 ribbon cables (Panel 1 `OUT` → Panel 2 `IN`) to form a `128x32` Protogen visor.

### What is a Shield / Carrier (SmartLED Shield V5)?
A **shield** (or **carrier board**) is a small adapter circuit board that sits *between* your Teensy 4.1 and the HUB75 LED panel:

1. **Voltage translation (3.3V → 5V):** The Teensy 4.1 outputs **3.3V** logic signals, while HUB75 panels expect **5V** logic signals. A shield contains fast buffer chips (like `74HCT245`) that cleanly boost the signals so panels don't flicker or glitch.
2. **Plug-and-play wiring:** Instead of running 14+ fragile jumper wires by hand, you plug your pre-pinned Teensy 4.1 into female headers on the shield, plug the HUB75 ribbon cable into the shield's 16-pin socket, and screw your 5V power wire into the shield's power terminal.

> **Pre-pinned Teensy 4.1 compatibility:** A standard "Teensy 4.1 with Pins" has **male headers pointing down**. It plugs straight into a breadboard or into **female headers** soldered onto a SmartLED Shield V5—no soldering directly onto the Teensy required.

---

## 2. Complete Dev-Bench Shopping Cart

> **Brazil Import Note (Remessa Conforme):** AliExpress and US-store prices below show the **base item price before shipping and Brazilian import taxes** (typically ~44% combined II + ICMS on orders up to US$ 50, and ~92% above US$ 50). Mercado Livre prices shown are typical local/in-country listings.

| # | Component | Exact Spec to Buy | USD Estimate (Base) | BRL Estimate (Mercado Livre / Local BR) | BRL Estimate (AliExpress Base, pre-tax) | Where to Buy / Notes |
|---|---|---|---:|---:|---:|---|
| 1 | **Teensy 4.1 (with Pins)** | PJRC Teensy 4.1 w/ pre-soldered headers (`DEV-16996`) | **$35.95** *(PJRC / SparkFun)* | **R$ 400 – R$ 700** | *(Rare / avoid fakes)* | Official PJRC / SparkFun / authorized distributor (or Mercado Livre local stock). |
| 2 | **HUB75 Shield (Option A — Easiest)** | **Pixelmatix SmartLED Shield V5 for Teensy 4** (`DEV-17521` / Adafruit `4828`) + female headers | **$20 – $25** *(SparkFun / Adafruit / Mouser)* | *Not sold locally on ML* | *Not sold on AliExpress* | Best experience; pinout matches our verified SmartLED V5 preset. Often out of stock—buy alongside Teensy if ordering from US/Mouser. |
| 2b | **HUB75 Adapter (Option B — Brazil Local DIY Backup)** | **2× `SN74HCT245N` DIP-20** level-shifter ICs + 16-pin IDC header (on breadboard/perfboard) | **$2 – $4** | **R$ 18 – R$ 30** *(kit 2–5 pcs)* | **R$ 10 – R$ 15** | Cheap local fallback in Brazil if importing SmartLED Shield V5 is out of stock or too expensive. Requires hand-wiring ~14 jumpers on a breadboard. |
| 3 | **2× RGB LED Matrix Panels** | **2× `64x32` P3 HUB75**, **1/16 scan (16S)** *(~192×96 mm each; DP5020B or FM6126A driver OK)* | **$24 – $44** *(for 2 panels, $12–$22 ea)* | **R$ 300 – R$ 600** *(for 2 panels)* | **R$ 130 – R$ 240** *(for 2 panels)* | **Critical:** Confirm **1/16 scan** and **HUB75** (avoid 1/8 scan, 1/4 scan, and HUB75E). Check that 16-pin IDC ribbon cables & 5V power cables are included. |
| 4 | **5V Power Supply (Bench)** | **5V 5A regulated switching power supply** (*Fonte Chaveada 5V 5A*, bivolt) + P4-to-screw-terminal adapter | **$7 – $12** | **R$ 35 – R$ 65** | **R$ 30 – R$ 50** | Easier and cheaper to buy locally on Mercado Livre (already BR plug & heavy to ship). Optionally add a 5V 3A USB-C power bank later for real-suit testing. |
| 5 | **Boop Sensor** | **1× APDS-9960** I2C proximity/gesture breakout (3.3V) | **$3 – $5** | **R$ 30 – R$ 45** | **R$ 15 – R$ 25** | Connects to Teensy I2C (`SDA=18`, `SCL=19`, `3.3V`, `GND`). Optional: add 1× **TTP223** touch sensor (~R$ 5–10 / ~$1) to test digital-pin boop mode too. |
| 6 | **Breadboard, Jumpers & Paw Buttons** | **Protoboard 830 pts + M-M / M-F DuPont jumpers + 4× tactile pushbuttons (6mm)** | **$8 – $14** | **R$ 50 – R$ 80** *(combo kit)* | **R$ 25 – R$ 45** | Buy as a single local kit on Mercado Livre (fast shipping, no import tax). Buttons wire directly GPIO → `GND` using internal pull-ups. |
| 7 | **Cooling Fan (PWM)** | **Option A:** Noctua `NF-A4x10 5V PWM` (4-pin, 5V)<br>**Option B:** Generic 40mm **5V 4-wire PWM** fan | **A:** $15 – $20<br>**B:** $4 – $8 | **A:** R$ 150 – R$ 220<br>**B:** *Rare locally (mostly 2-wire 5V or 4-wire 12V)* | **A:** R$ 90 – R$ 120<br>**B:** R$ 25 – R$ 45 | **Make sure it is 5V and 4-wire PWM** (not 12V!). 4-wire PWM connects straight to a Teensy PWM pin without needing an external MOSFET. |
| 8 | **USB Micro-B Data Cable** | Good shielded **USB-A/C to Micro-B *data* cable** | **$3 – $6** | **R$ 15 – R$ 35** | **R$ 10 – R$ 20** | Teensy 4.1 uses **Micro-B**. Avoid cheap charge-only cables. *(Skip if you already own a known-good data cable.)* |
| 9 | **USB Logic Analyzer *(Recommended Dev Tool)*** | **8-channel 24 MHz USB Logic Analyzer** (Saleae clone, works with free Linux **PulseView / sigrok**) | **$4 – $8** | **R$ 35 – R$ 90** | **R$ 20 – R$ 35** | Saves hours during `v0.2` HUB75 FlexIO bring-up by letting you inspect real `CLK`, `LATCH`, and `OE` waveforms on screen. |

---

## 3. Rough Total Budget Estimate

- **Lean Local/AliExpress Mix (using local `74HCT245` breadboard wiring + generic 5V PWM fan):**
  - **USD equivalent (base):** ~**$95 – $140 USD**
  - **BRL realistic total in Brazil (mix of local ML + AliExpress w/ taxes + Teensy):** ~**R$ 850 – R$ 1,300 BRL**
- **Plug-and-Play Comfort Mix (importing SmartLED Shield V5 + Noctua 5V PWM fan + local ML parts):**
  - **USD equivalent (base):** ~**$125 – $175 USD** *(before international shipping/taxes)*
  - **BRL realistic total in Brazil:** ~**R$ 1,250 – R$ 1,750 BRL**

---

## 4. Essential Bench Safety Notes (Read Before Plugging In!)

1. **Never power HUB75 panels from your computer's USB port.** Dual `64x32` panels can draw 3–5A at full brightness and will trip or damage a PC USB port.
2. **Prevent 5V back-feeding into USB:** Keep the external 5V panel power separate from the Teensy's `VIN` pin while plugged into USB (share **GND only**), *or* cut the Teensy 4.1 `VUSB-VIN` solder bridge on the bottom of the board if powering the Teensy from the shield's 5V supply.
3. **Double-check fan voltage:** Confirm your fan label says **5V** (not 12V) before wiring to the 5V rail.
4. **Verify HUB75 scan rate on arrival:** Check the silkscreen on the back of both panels for `64x32`, `16S` (or `1/16`), and identical driver IC labels so Panel 1 and Panel 2 match cleanly when daisy-chained.
