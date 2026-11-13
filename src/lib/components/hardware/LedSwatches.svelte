<script lang="ts">
  import { LED_SWATCHES, type LedColor } from "$lib/types";

  interface Props {
    value: LedColor;
    onChange: (led: LedColor) => void;
  }

  let { value, onChange }: Props = $props();
</script>

<div class="flex flex-col gap-1.5">
  <span class="text-xs text-text-secondary">키별 LED</span>
  <div class="flex flex-wrap gap-2">
    {#each LED_SWATCHES as sw (sw.id)}
      <button
        type="button"
        class="h-5 w-5 rounded-full border transition-transform hover:scale-110 {value === sw.id
          ? 'border-text-primary ring-2 ring-white/20'
          : 'border-border-subtle'}"
        style="background-color: {sw.hex};"
        title={sw.label}
        onclick={() => onChange(sw.id)}
      ></button>
    {/each}
  </div>
</div>
