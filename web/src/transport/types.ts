export interface HelloInfo {
  fwMajor: number;
  fwMinor: number;
  fwPatch: number;
  protocolVersion: number;
  mcuKind: number;
  visorWidth: number;
  visorHeight: number;
  buttonCount: number;
}

export interface PreviewState {
  expressionId: number;
  tintRgb: [number, number, number];
  brightness: number;
  virtualButtonMask: number;
}

export interface PreviewAck {
  activeExpression: number;
  effectiveBrightness: number;
}

export interface PacketStats {
  txReports: number;
  rxReports: number;
  lastOpcode: string;
  lastSeq: number;
}

/**
 * Shared transport contract implemented by Simulator Mode (v0.1) and
 * real browser WebHID (v0.2).
 */
export interface ProtoTransport {
  readonly mode: "simulator" | "webhid";
  hello(): Promise<HelloInfo>;
  sendPreview(state: PreviewState): Promise<PreviewAck>;
  getStats(): PacketStats;
}
