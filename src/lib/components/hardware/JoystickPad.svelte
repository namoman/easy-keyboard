<script lang="ts">
  interface Props {
    dx: number;
    dy: number;
    onChange: (dx: number, dy: number) => void;
  }

  let { dx, dy, onChange }: Props = $props();

  let padEl = $state<HTMLDivElement | null>(null);
  let dragging = $state(false);

  const MAX = 127;

  function clamp(n: number): number {
    return Math.max(-MAX, Math.min(MAX, Math.round(n)));
  }

  function fromPointer(clientX: number, clientY: number): void {
    const el = padEl;
    if (!el) return;
    const r = el.getBoundingClientRect();
    const nx = ((clientX - r.left) / r.width) * 2 - 1;
    const ny = ((clientY - r.top) / r.height) * 2 - 1;
    onChange(clamp(nx * MAX), clamp(ny * MAX));
  }

  function onDown(e: PointerEvent): void {
    dragging = true;
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    fromPointer(e.clientX, e.clientY);
  }

  function onMove(e: PointerEvent): void {
    if (!dragging) return;
    fromPointer(e.clientX, e.clientY);
  }

  function onUp(): void {
    dragging = false;
  }

  const knobX = $derived(50 + (dx / MAX) * 50);
  const knobY = $derived(50 + (dy / MAX) * 50);
</script>

<div class="flex flex-col gap-1 text-xs text-text-secondary">
  <span>커서 이동 (2D)</span>
  <div class="flex items-center gap-3">
    <div
      bind:this={padEl}
      class="relative h-20 w-20 shrink-0 cursor-crosshair rounded-lg border border-border-subtle bg-bg-canvas"
      role="application"
      aria-label="마우스 이동 조이스틱"
      onpointerdown={onDown}
      onpointermove={onMove}
      onpointerup={onUp}
      onpointercancel={onUp}
    >
      <div class="pointer-events-none absolute inset-0 flex items-center justify-center">
        <div class="h-px w-full bg-border-subtle"></div>
      </div>
      <div class="pointer-events-none absolute inset-0 flex items-center justify-center">
        <div class="h-full w-px bg-border-subtle"></div>
      </div>
      <div
        class="pointer-events-none absolute h-3 w-3 -translate-x-1/2 -translate-y-1/2 rounded-full bg-accent-cyan"
        style="left: {knobX}%; top: {knobY}%;"
      ></div>
    </div>
    <div class="flex flex-col gap-1 font-mono text-[11px]">
      <label class="flex items-center gap-1">
        X
        <input
          type="number"
          min={-MAX}
          max={MAX}
          class="w-14 rounded border border-border-subtle bg-bg-canvas px-1 py-0.5 text-text-primary"
          value={dx}
          oninput={(e) => onChange(clamp(Number(e.currentTarget.value)), dy)}
        />
      </label>
      <label class="flex items-center gap-1">
        Y
        <input
          type="number"
          min={-MAX}
          max={MAX}
          class="w-14 rounded border border-border-subtle bg-bg-canvas px-1 py-0.5 text-text-primary"
          value={dy}
          oninput={(e) => onChange(dx, clamp(Number(e.currentTarget.value)))}
        />
      </label>
    </div>
  </div>
</div>
