import { useCallback, useEffect, useRef, useState } from "react";
import { VisorCanvas } from "./components/VisorCanvas";
import { LUMIFUR_STARTER_PACK } from "./transport/packs";
import { SimulatorTransport } from "./transport/simulator";
import type { HelloInfo, PacketStats, PreviewAck } from "./transport/types";

const COLOR_SWATCHES: { name: string; hex: string }[] = [
  { name: "Cyan", hex: "#38bdf8" },
  { name: "Mint", hex: "#34d399" },
  { name: "Amber", hex: "#fbbf24" },
  { name: "Pink", hex: "#f472b6" },
  { name: "Purple", hex: "#a78bfa" },
  { name: "White", hex: "#ffffff" },
];

function hexToRgb(hex: string): [number, number, number] {
  const clean = hex.replace("#", "");
  const r = parseInt(clean.slice(0, 2), 16) || 0;
  const g = parseInt(clean.slice(2, 4), 16) || 0;
  const b = parseInt(clean.slice(4, 6), 16) || 0;
  return [r, g, b];
}

export function App() {
  const transportRef = useRef<SimulatorTransport | null>(null);

  const [hello, setHello] = useState<HelloInfo | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [packLoaded, setPackLoaded] = useState(false);

  const [selectedSlot, setSelectedSlot] = useState(0);
  const [tintHex, setTintHex] = useState("#38bdf8");
  const [brightness, setBrightness] = useState(80);
  const [pawPressed, setPawPressed] = useState(false);

  const [ack, setAck] = useState<PreviewAck | null>(null);
  const [stats, setStats] = useState<PacketStats>({
    txReports: 0,
    rxReports: 0,
    lastOpcode: "NONE",
    lastSeq: 0,
  });

  const pushPreview = useCallback(
    async (
      slot: number,
      hex: string,
      bright: number,
      buttonDown: boolean,
    ) => {
      const transport = transportRef.current;
      if (!transport) return;
      try {
        const res = await transport.sendPreview({
          expressionId: slot,
          tintRgb: hexToRgb(hex),
          brightness: bright,
          virtualButtonMask: buttonDown ? 1 : 0,
        });
        setAck(res);
        setStats(transport.getStats());
      } catch (err) {
        setError(String(err));
      }
    },
    [],
  );

  useEffect(() => {
    let cancelled = false;
    let instance: SimulatorTransport | null = null;

    async function boot() {
      try {
        instance = await SimulatorTransport.create(1);
        if (cancelled) {
          instance.dispose();
          return;
        }
        transportRef.current = instance;

        const info = await instance.hello();
        if (cancelled) return;
        setHello(info);
        setStats(instance.getStats());

        // Preload optional community PNG starter pack into simulator slots.
        await instance.loadStarterPack(LUMIFUR_STARTER_PACK);
        if (cancelled) return;
        setPackLoaded(true);

        const initialAck = await instance.sendPreview({
          expressionId: 0,
          tintRgb: hexToRgb("#38bdf8"),
          brightness: 80,
          virtualButtonMask: 0,
        });
        if (cancelled) return;
        setAck(initialAck);
        setStats(instance.getStats());
      } catch (err) {
        if (!cancelled) {
          setError(String(err));
        }
      }
    }

    void boot();
    return () => {
      cancelled = true;
      if (instance) {
        instance.dispose();
      }
      transportRef.current = null;
    };
  }, []);

  const handleSelectSlot = (slotId: number) => {
    setSelectedSlot(slotId);
    void pushPreview(slotId, tintHex, brightness, pawPressed);
  };

  const handleTintChange = (hex: string) => {
    setTintHex(hex);
    void pushPreview(selectedSlot, hex, brightness, pawPressed);
  };

  const handleBrightnessChange = (value: number) => {
    setBrightness(value);
    void pushPreview(selectedSlot, tintHex, value, pawPressed);
  };

  const handlePawChange = (pressed: boolean) => {
    setPawPressed(pressed);
    void pushPreview(selectedSlot, tintHex, brightness, pressed);
  };

  const handleLoadPack = async () => {
    const transport = transportRef.current;
    if (!transport) return;
    try {
      await transport.loadStarterPack(LUMIFUR_STARTER_PACK);
      setPackLoaded(true);
      await pushPreview(selectedSlot, tintHex, brightness, pawPressed);
    } catch (err) {
      setError(String(err));
    }
  };

  const handleClearSlots = () => {
    const transport = transportRef.current;
    if (!transport) return;
    transport.clearAllSlots(LUMIFUR_STARTER_PACK.length);
    setPackLoaded(false);
  };

  const getRgbaFrame = useCallback(() => {
    return transportRef.current ? transportRef.current.renderRgba() : null;
  }, []);

  const handleTick = useCallback(() => {
    transportRef.current?.tick();
  }, []);

  const visorWidth = hello?.visorWidth ?? 128;
  const visorHeight = hello?.visorHeight ?? 32;

  return (
    <div className="app-shell">
      <header className="topbar">
        <div className="brand-group">
          <div className="brand-slot">LOGO</div>
          <div>
            <div className="brand-title">ProtoStack Config</div>
            <div className="brand-sub">v0.1 Simulator (Rust/WASM + ProtoSpeak)</div>
          </div>
        </div>

        <div className="status-pills">
          <span className="pill pill--ok">
            <span className="dot" />
            Simulator Mode
          </span>
          {hello && (
            <span className="pill pill--mono">
              FW v{hello.fwMajor}.{hello.fwMinor}.{hello.fwPatch} | {hello.visorWidth}x
              {hello.visorHeight} | ProtoSpeak v{hello.protocolVersion}
            </span>
          )}
          <span className="pill pill--warn">
            <span className="dot" />
            Previewing in RAM
          </span>
        </div>
      </header>

      {error && <div className="error-banner">Error: {error}</div>}

      <main className="workspace">
        <aside className="sidebar">
          <section>
            <div className="section-header">
              <span className="section-title">Face Slots</span>
              <span className="slot-meta">
                {packLoaded ? "LumiFur Pack" : "Empty (Blank)"}
              </span>
            </div>
            <div className="slot-grid">
              {LUMIFUR_STARTER_PACK.map((slot) => {
                const isSelected = selectedSlot === slot.slotId;
                const isActive = ack?.activeExpression === slot.slotId;
                return (
                  <button
                    key={slot.slotId}
                    type="button"
                    className={`slot-btn ${isSelected ? "slot-btn--active" : ""}`}
                    onClick={() => handleSelectSlot(slot.slotId)}
                  >
                    <span>
                      Slot {slot.slotId}: {slot.label}
                    </span>
                    <span className="slot-meta">
                      {isActive ? "ACTIVE" : `${slot.frameUrls.length}f`}
                    </span>
                  </button>
                );
              })}
            </div>
            <div className="row-actions">
              <button
                type="button"
                className="btn btn--primary"
                onClick={() => void handleLoadPack()}
              >
                Load Starter Pack
              </button>
              <button
                type="button"
                className="btn"
                onClick={handleClearSlots}
              >
                Clear Slots
              </button>
            </div>
          </section>

          <section>
            <div className="section-header">
              <span className="section-title">Visor Color Tint</span>
              <span className="slot-meta">{tintHex.toUpperCase()}</span>
            </div>
            <div className="swatches">
              {COLOR_SWATCHES.map((s) => (
                <button
                  key={s.hex}
                  type="button"
                  title={s.name}
                  className={`swatch ${
                    tintHex.toLowerCase() === s.hex ? "swatch--active" : ""
                  }`}
                  style={{ background: s.hex }}
                  onClick={() => handleTintChange(s.hex)}
                />
              ))}
              <input
                type="color"
                className="color-input"
                aria-label="Custom visor tint color"
                value={tintHex}
                onChange={(e) => handleTintChange(e.target.value)}
              />
            </div>
          </section>

          <section>
            <div className="section-header">
              <span className="section-title">Master Brightness</span>
              <span className="slot-meta">{brightness}%</span>
            </div>
            <input
              type="range"
              min={0}
              max={100}
              value={brightness}
              className="slider"
              aria-label="Master brightness"
              onChange={(e) => handleBrightnessChange(Number(e.target.value))}
            />
          </section>

          <p className="notice">
            Starter PNG bitmaps are loaded by the web app from{" "}
            <code>public/packs/lumifur/</code> (Stefan Ritchie, BSD 3-Clause).
            The Rust <code>proto-brain</code> engine ships with zero hardcoded
            faces.
          </p>
        </aside>

        <section className="stage">
          <div className="visor-card">
            <div className="visor-frame">
              <VisorCanvas
                width={visorWidth}
                height={visorHeight}
                getRgbaFrame={getRgbaFrame}
                onTick={handleTick}
              />
              <div className="visor-caption">
                <span>PANEL 1 ({visorWidth / 2}x{visorHeight})</span>
                <span>
                  ACTIVE SLOT: {ack?.activeExpression ?? selectedSlot} |
                  BRIGHTNESS: {ack?.effectiveBrightness ?? brightness}%
                </span>
                <span>PANEL 2 ({visorWidth / 2}x{visorHeight})</span>
              </div>
            </div>

            <div className="paw-bar">
              <div>
                <strong>Virtual Paw Button 1</strong>
                <div className="brand-sub">
                  Hold to temporarily trigger Slot 2 (Surprised), release to
                  return to Slot {selectedSlot}.
                </div>
              </div>
              <button
                type="button"
                className={`paw-btn ${pawPressed ? "paw-btn--pressed" : ""}`}
                onPointerDown={() => handlePawChange(true)}
                onPointerUp={() => handlePawChange(false)}
                onPointerLeave={() => {
                  if (pawPressed) handlePawChange(false);
                }}
                onKeyDown={(e) => {
                  if (e.key === " " || e.key === "Enter") handlePawChange(true);
                }}
                onKeyUp={(e) => {
                  if (e.key === " " || e.key === "Enter") handlePawChange(false);
                }}
              >
                {pawPressed ? "Paw 1 Pressed" : "Hold Paw Button 1"}
              </button>
            </div>
          </div>

          <div className="telemetry-card">
            <span>ProtoSpeak Loopback (64-byte HID reports + CRC-16)</span>
            <span>
              TX: {stats.txReports} | RX: {stats.rxReports} | Last:{" "}
              {stats.lastOpcode} #{stats.lastSeq}
            </span>
          </div>
        </section>
      </main>
    </div>
  );
}
