<script lang="ts">
  interface Props {
    delayMs: number;
    onChange: (ms: number) => void;
  }

  let { delayMs, onChange }: Props = $props();

  function clamp(ms: number): number {
    return Math.max(0, Math.min(6000, Math.round(ms / 10) * 10));
  }

  function onWheel(e: WheelEvent): void {
    e.preventDefault();
    const step = e.shiftKey ? 100 : 10;
    const delta = e.deltaY < 0 ? step : -step;
    onChange(clamp(delayMs + delta));
  }

  function onClick(): void {
    const next = delayMs >= 6000 ? 0 : delayMs + 50;
    onChange(clamp(next));
  }
</script>

<button
  type="button"
  class="shrink-0 rounded-full border border-border-subtle bg-bg-canvas px-2 py-0.5 font-mono text-[10px] text-accent-cyan hover:border-accent-cyan"
  title="클릭: +50ms · 휠: ±10ms (Shift ±100)"
  onclick={onClick}
  onwheel={onWheel}
>
  ⏱ {delayMs}ms
</button>
