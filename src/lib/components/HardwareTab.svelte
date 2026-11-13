<script lang="ts">
  import KeyTimeline from "./hardware/KeyTimeline.svelte";
  import ScrollSlider from "./hardware/ScrollSlider.svelte";
  import JoystickPad from "./hardware/JoystickPad.svelte";
  import LedSwatches from "./hardware/LedSwatches.svelte";
  import type { HardwareMapping } from "$lib/types";

  interface Props {
    mapping: HardwareMapping;
    title: string;
    onChange: (m: HardwareMapping) => void;
  }

  let { mapping, title, onChange }: Props = $props();

  function patch(partial: Partial<HardwareMapping>): void {
    onChange({ ...mapping, ...partial });
  }
</script>

<div class="flex h-full flex-col gap-3 overflow-y-auto p-4">
  <div class="flex items-center justify-between">
    <h2 class="text-sm font-medium text-text-primary">{title}</h2>
    <span class="font-mono text-[10px] text-accent-cyan">하드웨어 · 온보드</span>
  </div>

  <KeyTimeline
    strokes={mapping.strokes}
    onChange={(strokes) => patch({ strokes })}
  />

  <div class="grid grid-cols-2 gap-3">
    <ScrollSlider
      value={mapping.scrollSpeed}
      onChange={(scrollSpeed) => patch({ scrollSpeed })}
    />
    <JoystickPad
      dx={mapping.mouseDx}
      dy={mapping.mouseDy}
      onChange={(mouseDx, mouseDy) => patch({ mouseDx, mouseDy })}
    />
  </div>

  <div class="mt-auto border-t border-border-subtle pt-3">
    <LedSwatches value={mapping.led} onChange={(led) => patch({ led })} />
  </div>
</div>
