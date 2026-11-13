<script lang="ts">
  import { i18n } from "$lib/i18n/i18n.svelte";
  import {
    buildLayerCells,
    downloadDataUrl,
    exportKeycapSheetPng,
    type CapSizeMm,
  } from "$lib/print/exportPrintPng";
  import "$lib/print/keycap-print.css";
  import type { SlotMode, SnippetSlot } from "$lib/types";

  interface Props {
    open: boolean;
    slots: SnippetSlot[];
    modeFor: (layer: number, key: number) => SlotMode;
    onClose: () => void;
  }

  let { open, slots, modeFor, onClose }: Props = $props();
  let sizeMm = $state<CapSizeMm>(12);

  const layers = $derived(
    [1, 2, 3].map((layer) => buildLayerCells(slots, layer, modeFor)),
  );

  function onPng(): void {
    const url = exportKeycapSheetPng(layers, sizeMm);
    downloadDataUrl(url, `easykeyboard-keycaps-${sizeMm}mm.png`);
  }
</script>

{#if open}
  <div class="ek-print-overlay" role="dialog" aria-modal="true">
    <div class="ek-print-toolbar ek-no-print">
      <h2>{i18n.t("printTitle")}</h2>
      <div class="ek-print-presets">
        <button
          type="button"
          class:active={sizeMm === 12}
          onclick={() => (sizeMm = 12)}>{i18n.t("printPreset12")}</button
        >
        <button
          type="button"
          class:active={sizeMm === 13.5}
          onclick={() => (sizeMm = 13.5)}>{i18n.t("printPreset135")}</button
        >
      </div>
      <p class="ek-print-hint">{i18n.t("printHint")}</p>
      <div class="ek-print-actions">
        <button type="button" class="primary" onclick={() => window.print()}
          >{i18n.t("printDo")}</button
        >
        <button type="button" onclick={onPng}>{i18n.t("printPng")}</button>
        <button type="button" onclick={onClose}>{i18n.t("printClose")}</button>
      </div>
    </div>

    <div class="ek-print-sheet" data-size={sizeMm} style="--cap: {sizeMm}mm">
      <header class="ek-print-sheet-head">
        EasyKeyboard · {sizeMm}×{sizeMm} mm · {i18n.t("printLayers")}
      </header>
      {#each layers as cells, li (li)}
        <section class="ek-print-layer">
          <h3>Layer {li + 1}</h3>
          <div class="ek-print-grid">
            {#each cells as cell (cell.key)}
              <div class="ek-print-cap" style="--cap: {sizeMm}mm">
                <span class="ek-print-num"
                  >{String(cell.key).padStart(2, "0")}</span
                >
                {#if cell.dot !== "none"}
                  <span class="ek-print-dot" data-dot={cell.dot}></span>
                {/if}
                <span class="ek-print-label">{cell.label}</span>
              </div>
            {/each}
          </div>
        </section>
      {/each}
    </div>
  </div>
{/if}
