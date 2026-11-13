<script lang="ts">
  import DelayPill from "./DelayPill.svelte";
  import { newStrokeId, type HardwareStroke } from "$lib/types";

  interface Props {
    strokes: HardwareStroke[];
    onChange: (strokes: HardwareStroke[]) => void;
  }

  let { strokes, onChange }: Props = $props();

  const MAX = 18;

  function addStroke(): void {
    if (strokes.length >= MAX) return;
    onChange([
      ...strokes,
      { id: newStrokeId(), mods: ["Ctrl"], key: "C", delayMs: strokes.length ? 50 : 0 },
    ]);
  }

  function removeLast(): void {
    if (!strokes.length) return;
    onChange(strokes.slice(0, -1));
  }

  function setDelay(index: number, ms: number): void {
    onChange(strokes.map((s, i) => (i === index ? { ...s, delayMs: ms } : s)));
  }

  function label(s: HardwareStroke): string {
    const mods = s.mods.length ? `${s.mods.join("+")}+` : "";
    return `${mods}${s.key}`;
  }
</script>

<div class="flex flex-col gap-2">
  <div class="flex items-center justify-between">
    <span class="text-xs text-text-secondary">키 시퀀스 ({strokes.length}/{MAX})</span>
    <div class="flex gap-1">
      <button
        type="button"
        class="rounded border border-border-subtle px-2 py-0.5 text-[10px] text-text-secondary hover:border-accent-cyan hover:text-accent-cyan disabled:opacity-40"
        disabled={strokes.length >= MAX}
        onclick={addStroke}
      >
        + 타
      </button>
      <button
        type="button"
        class="rounded border border-border-subtle px-2 py-0.5 text-[10px] text-text-secondary hover:border-status-rose disabled:opacity-40"
        disabled={!strokes.length}
        onclick={removeLast}
      >
        삭제
      </button>
    </div>
  </div>

  <div
    class="flex min-h-[48px] items-center gap-1 overflow-x-auto rounded-lg border border-border-subtle bg-bg-canvas px-2 py-2"
  >
    {#if strokes.length === 0}
      <span class="text-[10px] text-text-secondary">비어 있음 · + 타로 최대 18타 추가</span>
    {:else}
      {#each strokes as stroke, i (stroke.id)}
        {#if i > 0}
          <DelayPill delayMs={stroke.delayMs} onChange={(ms) => setDelay(i, ms)} />
          <span class="text-text-secondary">→</span>
        {/if}
        <span
          class="shrink-0 rounded border border-accent-cyan/40 bg-accent-cyan/10 px-2 py-1 font-mono text-[11px] text-accent-cyan"
        >
          {label(stroke)}
        </span>
      {/each}
    {/if}
  </div>
</div>
