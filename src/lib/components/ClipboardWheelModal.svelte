<script lang="ts">
  import type { WheelView } from "$lib/types";

  interface Props {
    view: WheelView | null;
    visible: boolean;
  }

  let { view, visible }: Props = $props();

  function linePreview(text: string, max = 3): string[] {
    return text.split(/\r?\n/).filter((l) => l.length > 0).slice(0, max);
  }

  const current = $derived(view?.preview ?? "");
  const lines = $derived(linePreview(current));
  const idx = $derived(view?.cursor ?? 0);
  const total = $derived(view?.items.length ?? 0);
</script>

{#if visible && view && total > 0}
  <div
    class="pointer-events-none absolute inset-x-2 bottom-2 z-30 rounded-lg border border-accent-cyan/40 bg-[#0B0C0E]/95 p-2 shadow-xl"
  >
    <p class="mb-1 font-mono text-[10px] text-accent-cyan">
      클립보드 휠 · {idx + 1}/{total}
    </p>
    <div class="space-y-0.5 font-mono text-[11px] text-text-primary">
      {#each lines as line, i (i)}
        <p class="truncate {i === 0 ? 'text-accent-amber' : 'text-text-secondary'}">
          {line || "⏎"}
        </p>
      {/each}
      {#if !lines.length}
        <p class="text-text-secondary">(빈 항목)</p>
      {/if}
    </div>
    <p class="mt-1 text-[9px] text-text-secondary">
      Ctrl+Alt+Shift+PgUp/Dn · Enter 붙여넣기
    </p>
  </div>
{/if}
