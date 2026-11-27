import initWasm, {
  WasmHelmet,
  decode_hello_hid_reports,
  decode_preview_ack_hid_reports,
  encode_hello_hid_reports,
  encode_preview_hid_reports,
} from "../wasm/pkg/proto_wasm";
import { loadPngAs1BitMask, type StarterSlotSpec } from "./packs";
import type {
  HelloInfo,
  PacketStats,
  PreviewAck,
  PreviewState,
  ProtoTransport,
} from "./types";

const HID_REPORT_SIZE = 64;

/**
 * In-memory ProtoSpeak loopback transport backed by `crates/proto-wasm`.
 * Every command is serialized into real CRC-checked 64-byte HID reports and
 * fed through `WasmHelmet::push_hid_report`, exercising the exact wire format
 * used by real Teensy USB WebHID in v0.2.
 */
export class SimulatorTransport implements ProtoTransport {
  readonly mode = "simulator" as const;

  private helmet: WasmHelmet;
  private seq = 1;
  private stats: PacketStats = {
    txReports: 0,
    rxReports: 0,
    lastOpcode: "NONE",
    lastSeq: 0,
  };

  private constructor(helmet: WasmHelmet) {
    this.helmet = helmet;
  }

  static async create(buttonCount = 1): Promise<SimulatorTransport> {
    await initWasm();
    const helmet = new WasmHelmet(buttonCount);
    helmet.set_button_override_slot(2);
    return new SimulatorTransport(helmet);
  }

  async hello(): Promise<HelloInfo> {
    const seq = this.nextSeq();
    const txBytes = encode_hello_hid_reports(seq);
    const rxBytes = this.exchangeReports(txBytes, "HELLO_INFO (0x0001)", seq);
    const decoded = decode_hello_hid_reports(rxBytes);
    try {
      return {
        fwMajor: decoded.fw_major,
        fwMinor: decoded.fw_minor,
        fwPatch: decoded.fw_patch,
        protocolVersion: decoded.protocol_version,
        mcuKind: decoded.mcu_kind,
        visorWidth: decoded.visor_width,
        visorHeight: decoded.visor_height,
        buttonCount: decoded.button_count,
      };
    } finally {
      decoded.free();
    }
  }

  async sendPreview(state: PreviewState): Promise<PreviewAck> {
    const seq = this.nextSeq();
    const txBytes = encode_preview_hid_reports(
      seq,
      state.expressionId,
      state.tintRgb[0],
      state.tintRgb[1],
      state.tintRgb[2],
      state.brightness,
      state.virtualButtonMask,
    );
    const rxBytes = this.exchangeReports(txBytes, "PREVIEW_STATE (0x0010)", seq);
    const decoded = decode_preview_ack_hid_reports(rxBytes);
    try {
      return {
        activeExpression: decoded.active_expression,
        effectiveBrightness: decoded.effective_brightness,
      };
    } finally {
      decoded.free();
    }
  }

  async loadStarterPack(pack: StarterSlotSpec[]): Promise<void> {
    for (const slot of pack) {
      this.helmet.clear_slot(slot.slotId);
      for (let i = 0; i < slot.frameUrls.length; i++) {
        const packed = await loadPngAs1BitMask(slot.frameUrls[i]);
        this.helmet.load_slot_frame(
          slot.slotId,
          i,
          packed.width,
          packed.height,
          packed.mask,
        );
      }
      this.helmet.set_slot_frame_ticks(slot.slotId, slot.frameTicks);
    }
  }

  clearAllSlots(slotCount: number): void {
    for (let i = 0; i < slotCount; i++) {
      this.helmet.clear_slot(i);
    }
  }

  tick(): void {
    this.helmet.tick();
  }

  renderRgba(): Uint8Array {
    return this.helmet.render_rgba();
  }

  visorWidth(): number {
    return this.helmet.visor_width();
  }

  visorHeight(): number {
    return this.helmet.visor_height();
  }

  getStats(): PacketStats {
    return { ...this.stats };
  }

  dispose(): void {
    this.helmet.free();
  }

  private nextSeq(): number {
    const current = this.seq;
    this.seq = (this.seq + 1) & 0xffff;
    return current;
  }

  private exchangeReports(
    txFlat: Uint8Array,
    opcodeLabel: string,
    seq: number,
  ): Uint8Array {
    const chunkCount = txFlat.length / HID_REPORT_SIZE;
    let rxFlat: Uint8Array = new Uint8Array(0);

    for (let i = 0; i < chunkCount; i++) {
      const report = txFlat.subarray(
        i * HID_REPORT_SIZE,
        (i + 1) * HID_REPORT_SIZE,
      );
      const response = this.helmet.push_hid_report(report);
      this.stats.txReports += 1;
      if (response.length > 0) {
        rxFlat = response;
        this.stats.rxReports += response.length / HID_REPORT_SIZE;
      }
    }

    this.stats.lastOpcode = opcodeLabel;
    this.stats.lastSeq = seq;
    return rxFlat;
  }
}
