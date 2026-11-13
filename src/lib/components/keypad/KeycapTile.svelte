<script lang="ts">
  import { ledHex, type LedColor, type SlotMode, type SnippetSlot } from "$lib/types";

  interface Props {
    keyNum: number;
    slot: SnippetSlot | undefined;
    mode: SlotMode;
    led: LedColor;
    selected: boolean;
    pressed: boolean;
    accent: "amber" | "cyan";
    onSelect: () => void;
    onDragStart: (e: DragEvent) => void;
    onDrop: (e: DragEvent) => void;
  }

  let {
    keyNum,
    slot,
    mode,
    led,
    selected,
    pressed,
    accent,
    onSelect,
    onDragStart,
    onDrop,
  }: Props = $props();

  const hasBody = $derived(Boolean(slot?.body?.trim()));
  const hasHw = $derived(mode === "B");
  const ring = $derived(
    accent === "cyan"
      ? "border-accent-cyan ring-2 ring-accent-cyan/20"
      : "border-accent-amber ring-2 ring-accent-amber/20",
  );
</script>

<button
  type="button"
  draggable="true"
  class="flex h-[78px] w-[78px] flex-col justify-between rounded-xl border bg-bg-keycap p-2 text-left transition-transform duration-75 hover:bg-bg-keycap-hover {selected
    ? ring
    : 'border-border-subtle'} {pressed ? 'scale-[0.96] brightness-110' : ''}"
  style="border-bottom-width: 2px; border-bottom-color: {ledHex(led)};"
  onclick={onSelect}
  ondragstart={onDragStart}
  ondragover={(e) => e.preventDefault()}
  ondrop={onDrop}
>
  <div class="flex w-full items-start justify-between">
    <span class="font-mono text-[10px] font-semibold text-text-secondary">
      {String(keyNum).padStart(2, "0")}
    </span>
    {#if slot?.secret}
      <span class="h-1.5 w-1.5 rounded-full bg-status-rose" title="Secret"></span>
    {:else if hasHw}
      <span class="h-1.5 w-1.5 rounded-full bg-accent-cyan" title="Hardware"></span>
    {:else if hasBody}
      <span class="h-1.5 w-1.5 rounded-full bg-accent-amber" title="Snippet"></span>
    {:else}
      <span class="h-1.5 w-1.5 rounded-full bg-transparent"></span>
    {/if}
  </div>
  <span class="line-clamp-2 text-center text-xs leading-tight text-text-primary">
    {slot?.label || (hasBody ? "…" : "")}
  </span>
  <span class="truncate font-mono text-[10px] text-text-secondary">
    {slot?.trigger || "—"}
  </span>
</button>
