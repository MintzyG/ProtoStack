import { useEffect, useRef } from "react";

interface VisorCanvasProps {
  width: number;
  height: number;
  getRgbaFrame: () => Uint8Array | null;
  onTick: () => void;
}

const CELL_PITCH = 6;
const LED_RADIUS = 2.15;
const SEAM_GAP = 10;

/**
 * Renders a logical `width x height` RGBA framebuffer from Rust/WASM as
 * dual LED matrix panels with subtle dark pixel pitch and center seam.
 */
export function VisorCanvas({
  width,
  height,
  getRgbaFrame,
  onTick,
}: VisorCanvasProps) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const halfWidth = Math.floor(width / 2);
  const canvasWidth = width * CELL_PITCH + SEAM_GAP;
  const canvasHeight = height * CELL_PITCH;

  useEffect(() => {
    let rafId = 0;
    let lastTickMs = 0;
    const TICK_INTERVAL_MS = 50; // 20 FPS animation tick

    const draw = (nowMs: number) => {
      if (nowMs - lastTickMs >= TICK_INTERVAL_MS) {
        onTick();
        lastTickMs = nowMs;
      }

      const canvas = canvasRef.current;
      const rgba = getRgbaFrame();
      if (canvas && rgba && rgba.length === width * height * 4) {
        const ctx = canvas.getContext("2d");
        if (ctx) {
          ctx.fillStyle = "#07080a";
          ctx.fillRect(0, 0, canvasWidth, canvasHeight);

          // Center bezel seam between left and right panels.
          ctx.fillStyle = "#12151b";
          ctx.fillRect(halfWidth * CELL_PITCH + 2, 0, SEAM_GAP - 4, canvasHeight);

          for (let y = 0; y < height; y++) {
            const cy = y * CELL_PITCH + CELL_PITCH / 2;
            for (let x = 0; x < width; x++) {
              const offset = (y * width + x) * 4;
              const r = rgba[offset];
              const g = rgba[offset + 1];
              const b = rgba[offset + 2];
              const seamShift = x >= halfWidth ? SEAM_GAP : 0;
              const cx = x * CELL_PITCH + CELL_PITCH / 2 + seamShift;

              if (r === 0 && g === 0 && b === 0) {
                ctx.fillStyle = "#151820";
              } else {
                ctx.fillStyle = `rgb(${r},${g},${b})`;
              }

              ctx.beginPath();
              ctx.arc(cx, cy, LED_RADIUS, 0, Math.PI * 2);
              ctx.fill();
            }
          }
        }
      }

      rafId = requestAnimationFrame(draw);
    };

    rafId = requestAnimationFrame(draw);
    return () => cancelAnimationFrame(rafId);
  }, [width, height, halfWidth, canvasWidth, canvasHeight, getRgbaFrame, onTick]);

  return (
    <canvas
      ref={canvasRef}
      width={canvasWidth}
      height={canvasHeight}
      className="visor-canvas"
      aria-label={`Live ${width}x${height} LED visor preview`}
    />
  );
}
