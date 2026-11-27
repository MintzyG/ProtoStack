export interface StarterSlotSpec {
  slotId: number;
  label: string;
  frameTicks: number;
  frameUrls: string[];
}

/**
 * Optional community starter pack adapted from Stefan Ritchie's
 * LumiFur_Controller (BSD 3-Clause; see THIRD_PARTY_NOTICES.md).
 * Loaded from normal PNG files in public/packs/lumifur/ - not baked into firmware.
 */
export const LUMIFUR_STARTER_PACK: StarterSlotSpec[] = [
  {
    slotId: 0,
    label: "Neutral",
    frameTicks: 18,
    frameUrls: [
      "/packs/lumifur/neutral.png",
      "/packs/lumifur/neutral.png",
      "/packs/lumifur/neutral.png",
      "/packs/lumifur/neutral_blink.png",
    ],
  },
  {
    slotId: 1,
    label: "Happy",
    frameTicks: 10,
    frameUrls: [
      "/packs/lumifur/happy.png",
      "/packs/lumifur/happy_open.png",
    ],
  },
  {
    slotId: 2,
    label: "Surprised",
    frameTicks: 10,
    frameUrls: ["/packs/lumifur/surprised.png"],
  },
];

export interface PackedMask {
  width: number;
  height: number;
  mask: Uint8Array;
}

/**
 * Fetches a PNG image and converts non-transparent bright pixels into
 * an MSB-first 1-bit packed mask matching `proto-brain`'s `Bitmap1Bit` format.
 */
export async function loadPngAs1BitMask(url: string): Promise<PackedMask> {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Failed to fetch ${url} (${response.status})`);
  }
  const blob = await response.blob();
  const bitmap = await createImageBitmap(blob);

  const width = bitmap.width;
  const height = bitmap.height;
  const canvas = document.createElement("canvas");
  canvas.width = width;
  canvas.height = height;

  const ctx = canvas.getContext("2d");
  if (!ctx) {
    bitmap.close();
    throw new Error("Could not create 2D canvas context for PNG decode");
  }
  ctx.drawImage(bitmap, 0, 0);
  bitmap.close();

  const imageData = ctx.getImageData(0, 0, width, height).data;
  const rowBytes = Math.ceil(width / 8);
  const mask = new Uint8Array(rowBytes * height);

  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const idx = (y * width + x) * 4;
      const r = imageData[idx];
      const g = imageData[idx + 1];
      const b = imageData[idx + 2];
      const a = imageData[idx + 3];
      const lit = a > 127 && (r + g + b) / 3 > 127;
      if (lit) {
        mask[y * rowBytes + (x >> 3)] |= 0x80 >> (x & 7);
      }
    }
  }

  return { width, height, mask };
}
