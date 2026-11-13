import type { SnippetSlot } from "$lib/types";

export type CapSizeMm = 12 | 13.5;

export interface PrintCapCell {
  key: number;
  label: string;
  /** amber | cyan | rose | none */
  dot: "amber" | "cyan" | "rose" | "none";
}

const DPI = 300;
const MM_PER_IN = 25.4;

function mmToPx(mm: number): number {
  return Math.round((mm / MM_PER_IN) * DPI);
}

function slotDot(
  slot: SnippetSlot | undefined,
  mode: "A" | "B",
): PrintCapCell["dot"] {
  if (slot?.secret) return "rose";
  if (mode === "B") return "cyan";
  if (slot?.body?.trim()) return "amber";
  return "none";
}

export function buildLayerCells(
  slots: SnippetSlot[],
  layer: number,
  modeFor: (layer: number, key: number) => "A" | "B",
): PrintCapCell[] {
  const cells: PrintCapCell[] = [];
  for (let key = 1; key <= 16; key++) {
    const slot = slots.find((s) => s.layer === layer && s.key === key);
    cells.push({
      key,
      label: slot?.label?.trim() || "",
      dot: slotDot(slot, modeFor(layer, key)),
    });
  }
  return cells;
}

const DOT_HEX: Record<PrintCapCell["dot"], string> = {
  amber: "#F59E0B",
  cyan: "#06B6D4",
  rose: "#F43F5E",
  none: "transparent",
};

/** High-res PNG of layers 1–3 (Zero-Network: canvas → data URL). */
export function exportKeycapSheetPng(
  layers: PrintCapCell[][],
  sizeMm: CapSizeMm,
): string {
  const gapMm = 2;
  const marginMm = 8;
  const titleMm = 6;
  const layerGapMm = 6;
  const gridW = 4 * sizeMm + 3 * gapMm;
  const gridH = 4 * sizeMm + 3 * gapMm;
  const pageW = marginMm * 2 + gridW;
  const pageH =
    marginMm * 2 +
    titleMm +
    3 * (titleMm + gridH) +
    2 * layerGapMm;

  const canvas = document.createElement("canvas");
  canvas.width = mmToPx(pageW);
  canvas.height = mmToPx(pageH);
  const ctx = canvas.getContext("2d");
  if (!ctx) throw new Error("canvas 2d unavailable");

  const sx = canvas.width / pageW;
  const sy = canvas.height / pageH;
  ctx.scale(sx, sy);
  ctx.fillStyle = "#ffffff";
  ctx.fillRect(0, 0, pageW, pageH);
  ctx.fillStyle = "#111827";
  ctx.font = "bold 4px sans-serif";
  ctx.fillText(`EasyKeyboard · ${sizeMm}×${sizeMm} mm`, marginMm, marginMm + 3);

  let y = marginMm + titleMm;
  for (let li = 0; li < 3; li++) {
    const cells = layers[li] ?? [];
    ctx.fillStyle = "#374151";
    ctx.font = "3.2px sans-serif";
    ctx.fillText(`Layer ${li + 1}`, marginMm, y + 3);
    y += titleMm;
    drawGrid(ctx, cells, marginMm, y, sizeMm, gapMm);
    y += gridH + layerGapMm;
  }

  return canvas.toDataURL("image/png");
}

function drawGrid(
  ctx: CanvasRenderingContext2D,
  cells: PrintCapCell[],
  ox: number,
  oy: number,
  sizeMm: number,
  gapMm: number,
): void {
  for (let i = 0; i < 16; i++) {
    const row = Math.floor(i / 4);
    const col = i % 4;
    const x = ox + col * (sizeMm + gapMm);
    const y = oy + row * (sizeMm + gapMm);
    const cell = cells[i]!;
    ctx.strokeStyle = "#9ca3af";
    ctx.setLineDash([1.2, 0.8]);
    ctx.lineWidth = 0.25;
    ctx.strokeRect(x, y, sizeMm, sizeMm);
    ctx.setLineDash([]);
    ctx.fillStyle = "#6b7280";
    ctx.font = "2.2px monospace";
    ctx.fillText(String(cell.key).padStart(2, "0"), x + 0.8, y + 2.4);
    if (cell.dot !== "none") {
      ctx.fillStyle = DOT_HEX[cell.dot];
      ctx.beginPath();
      ctx.arc(x + sizeMm - 1.4, y + 1.6, 0.7, 0, Math.PI * 2);
      ctx.fill();
    }
    ctx.fillStyle = "#111827";
    ctx.font = "2.6px sans-serif";
    const label = cell.label.slice(0, 12);
    ctx.fillText(label, x + 0.8, y + sizeMm / 2 + 1);
  }
}

export function downloadDataUrl(dataUrl: string, filename: string): void {
  const a = document.createElement("a");
  a.href = dataUrl;
  a.download = filename;
  a.click();
}
