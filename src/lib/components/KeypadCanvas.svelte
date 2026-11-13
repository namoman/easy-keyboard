<script lang="ts">
  import KnobUnit from "./keypad/KnobUnit.svelte";
  import KeycapTile from "./keypad/KeycapTile.svelte";
  import { session } from "$lib/stores/session.svelte";
  import type {
    CanvasSelection,
    KnobAction,
    SnippetSlot,
  } from "$lib/types";

  interface Props {
    layer: number;
    slots: SnippetSlot[];
    selection: CanvasSelection;
    onSelect: (sel: CanvasSelection) => void;
    onSwapKeys: (a: number, b: number) => void;
  }

  let { layer, slots, selection, onSelect, onSwapKeys }: Props = $props();

  let dragFrom = $state<number | null>(null);

  function slotFor(key: number): SnippetSlot | undefined {
    return slots.find((s) => s.layer === layer && s.key === key);
  }

  function selectedKnobAction(knob: 1 | 2 | 3): KnobAction | null {
    if (selection.kind !== "knob" || selection.knob !== knob) return null;
    return selection.action;
  }

  function accentFor(key: number): "amber" | "cyan" {
    return session.modeFor(layer, key) === "B" ? "cyan" : "amber";
  }

  function onKeyDragStart(key: number, e: DragEvent): void {
    dragFrom = key;
    e.dataTransfer?.setData("text/plain", String(key));
    if (e.dataTransfer) e.dataTransfer.effectAllowed = "move";
  }

  function onKeyDrop(target: number, e: DragEvent): void {
    e.preventDefault();
    const raw = e.dataTransfer?.getData("text/plain") ?? String(dragFrom ?? "");
    const from = Number.parseInt(raw, 10);
    dragFrom = null;
    if (!Number.isFinite(from) || from < 1 || from > 16 || from === target) return;
    onSwapKeys(from, target);
  }
</script>

<div class="flex h-full w-full flex-col gap-3 overflow-hidden p-4">
  <div class="flex items-center justify-between">
    <p class="text-[10px] uppercase tracking-wider text-text-secondary">
      Keypad · Layer {layer} · {session.canvasRotation}°
    </p>
  </div>

  <div
    class="flex min-h-0 flex-1 flex-col items-center justify-center gap-4 transition-transform duration-200"
    style="transform: rotate({session.canvasRotation}deg);"
  >
    <div class="flex justify-center gap-6">
      {#each [1, 2, 3] as knob (knob)}
        <KnobUnit
          knob={knob as 1 | 2 | 3}
          selectedAction={selectedKnobAction(knob as 1 | 2 | 3)}
          onSelect={(action) =>
            onSelect({ kind: "knob", knob: knob as 1 | 2 | 3, action })}
        />
      {/each}
    </div>

    <div class="grid grid-cols-4 gap-2">
      {#each Array.from({ length: 16 }, (_, i) => i + 1) as key (key)}
        {@const slot = slotFor(key)}
        {@const hw = session.hardwareFor(layer, key)}
        <KeycapTile
          keyNum={key}
          {slot}
          mode={session.modeFor(layer, key)}
          led={hw.led}
          selected={selection.kind === "key" && selection.key === key}
          pressed={session.pressedKey === key}
          accent={accentFor(key)}
          onSelect={() => onSelect({ kind: "key", key })}
          onDragStart={(e) => onKeyDragStart(key, e)}
          onDrop={(e) => onKeyDrop(key, e)}
        />
      {/each}
    </div>
  </div>
</div>
