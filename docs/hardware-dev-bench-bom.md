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

## 2. Lean Tabletop Dev-Bench Cart (`v0.2` – `v1.0`)

Because early development is a **tabletop bench setup** (no helmet shell yet), only the visor display, Teensy 4.1, 5V power, and paw buttons are required for `v1.0`. Boop sensors and cooling fans are deferred to `v1.1` (see Section 3).

> **Brazil Import Note (Remessa Conforme):** AliExpress and US-store prices below show the **base item price before shipping and Brazilian import taxes** (typically ~44% combined II + ICMS on orders up to US$ 50, and ~92% above US$ 50). Mercado Livre prices shown are typical local/in-country listings.

| # | Status | Component | Exact Spec to Buy | USD Estimate (Base) | BRL Estimate (Mercado Livre / Local BR) | BRL Estimate (AliExpress Base, pre-tax) | Where to Buy / Notes |
|---|---|---|---|---:|---:|---:|---|
| 1 | **Need** | **Teensy 4.1 (with Pins)** | PJRC Teensy 4.1 w/ pre-soldered headers (`DEV-16996`) | **$35.95** *(PJRC / SparkFun)* | **R$ 400 – R$ 700** | *(Rare / avoid fakes)* | Official PJRC / SparkFun / authorized distributor (or Mercado Livre local stock). |
| 2 | **Need (pick A or B)** | **A: HUB75 Shield *(Easiest)*** | **Pixelmatix SmartLED Shield V5 for Teensy 4** (`DEV-17521` / Adafruit `4828`) + female headers | **$20 – $25** *(SparkFun / Adafruit / Mouser)* | *Not sold locally on ML* | *Not sold on AliExpress* | Plug-and-play; pinout matches our SmartLED V5 preset. **No full breadboard needed** if you choose Option A. Bundle with Teensy if importing from US/Mouser. |
| 2b | **Need (pick A or B)** | **B: HUB75 DIY Adapter *(BR Local Backup)*** | **2× `SN74HCT245N` DIP-20** level-shifters + 16-pin IDC header + **830-pt breadboard & jumpers** | **$8 – $14** | **R$ 50 – R$ 80** *(chips + protoboard + jumpers)* | **R$ 30 – R$ 50** | Cheap local fallback in Brazil if SmartLED Shield V5 is out of stock or expensive to import. You only need a full breadboard if you pick Option B. |
| 3 | **Need** | **2× RGB LED Matrix Panels** | **2× `64x32` P3 HUB75**, **1/16 scan (16S)** *(~192×96 mm each; DP5020B or FM6126A driver OK)* | **$24 – $44** *(for 2 panels, $12–$22 ea)* | **R$ 300 – R$ 600** *(for 2 panels)* | **R$ 130 – R$ 240** *(for 2 panels)* | **Critical:** Confirm **1/16 scan** and **HUB75** (avoid 1/8 scan, 1/4 scan, and HUB75E). Check that 16-pin IDC ribbon cables & 5V power cables are included. |
| 4 | **Need** | **5V Power Supply (Bench)** | **5V 5A regulated switching power supply** (*Fonte Chaveada 5V 5A*, bivolt) + P4-to-screw-terminal adapter | **$7 – $12** | **R$ 35 – R$ 65** | **R$ 30 – R$ 50** | Easier and cheaper to buy locally on Mercado Livre (already BR plug & heavy to ship). |
| 5 | **Need** | **Paw Buttons + Wires** | **2–4× tactile pushbuttons** + a few M-F/F-F DuPont wires *(+ optional tiny mini-breadboard if using Option A)* | **$2 – $4** | **R$ 15 – R$ 25** | **R$ 10 – R$ 15** | Wire directly from Teensy GPIO → `GND` using internal pull-ups. |
| 6 | **Need** | **USB Micro-B *Data* Cable** | Good shielded **USB-A/C to Micro-B *data* cable** | **$3 – $6** | **R$ 15 – R$ 35** | **R$ 10 – R$ 20** | Teensy 4.1 uses **Micro-B**. Avoid charge-only cables. *(Skip if you already own a known-good data cable.)* |
| 7 | *Optional* | **USB Logic Analyzer *(Debug Tool)*** | **8-channel 24 MHz USB Logic Analyzer** (Saleae clone, works with free Linux **PulseView / sigrok**) | **$4 – $8** | **R$ 35 – R$ 90** | **R$ 20 – R$ 35** | Not needed to run ProtoStack—cheap insurance only if we hit a HUB75 signal timing issue during `v0.2` bring-up. Fine to skip for now. |

---

## 3. Deferred to `v1.1` (Do NOT Buy Yet)

These are helmet-shell add-ons deferred out of `v1.0` to keep the tabletop bench lean and affordable:

| Component | Spec (For Later) | Typical Cost Saved Now |
|---|---|---:|
| **Boop Sensor** | 1× **APDS-9960** I2C proximity breakout (3.3V) *(+ optional TTP223 touch)* | ~$3–$5 / **R$ 30–45** |
| **Cooling Fan (5V PWM)** | Noctua `NF-A4x10 5V PWM` or generic 40mm **5V 4-wire PWM** *(never 12V)* | ~$15–$20 / **R$ 150–220** |

---

## 4. Rough Total Budget Estimate (`v1.0` Tabletop Cart)

- **Lean Local/AliExpress Mix (Option B: local `74HCT245` + breadboard + AliExpress panels + local ML basics + Teensy):**
  - **USD equivalent (base):** ~**$75 – $115 USD**
  - **BRL realistic total in Brazil (local ML + AliExpress w/ taxes + Teensy):** ~**R$ 680 – R$ 1,050 BRL**
- **Plug-and-Play Mix (Option A: importing SmartLED Shield V5 alongside Teensy + AliExpress/ML panels + local basics):**
  - **USD equivalent (base):** ~**$90 – $125 USD** *(before international shipping/taxes)*
  - **BRL realistic total in Brazil:** ~**R$ 950 – R$ 1,400 BRL**

---

## 5. Essential Bench Safety Notes (Read Before Plugging In!)

1. **Never power HUB75 panels from your computer's USB port.** Dual `64x32` panels can draw 3–5A at full brightness and will trip or damage a PC USB port.
2. **Prevent 5V back-feeding into USB:** Keep the external 5V panel power separate from the Teensy's `VIN` pin while plugged into USB (share **GND only**), *or* cut the Teensy 4.1 `VUSB-VIN` solder bridge on the bottom of the board if powering the Teensy from the shield's 5V supply.
3. **Verify HUB75 scan rate on arrival:** Check the silkscreen on the back of both panels for `64x32`, `16S` (or `1/16`), and identical driver IC labels so Panel 1 and Panel 2 match cleanly when daisy-chained.
