<script lang="ts">
  import type { KnobAction } from "$lib/types";

  interface Props {
    knob: 1 | 2 | 3;
    selectedAction: KnobAction | null;
    onSelect: (action: KnobAction) => void;
  }

  let { knob, selectedAction, onSelect }: Props = $props();

  const actions: { id: KnobAction; label: string }[] = [
    { id: "ccw", label: "↺" },
    { id: "press", label: "●" },
    { id: "cw", label: "↻" },
  ];
</script>

<div class="flex flex-col items-center gap-2">
  <div
    class="flex h-16 w-16 items-center justify-center rounded-full border border-border-subtle bg-bg-keycap font-mono text-[10px] text-text-secondary"
  >
    N{knob}
  </div>
  <div class="flex gap-1">
    {#each actions as a (a.id)}
      <button
        type="button"
        class="rounded-full border px-2 py-0.5 font-mono text-[10px] transition-colors {selectedAction ===
        a.id
          ? 'border-accent-cyan text-accent-cyan ring-1 ring-accent-cyan/30'
          : 'border-border-subtle text-text-secondary hover:bg-bg-keycap-hover'}"
        onclick={() => onSelect(a.id)}
        title={a.id}
      >
        {a.label}
      </button>
    {/each}
  </div>
</div>
