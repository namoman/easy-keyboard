<script lang="ts">
  import { i18n } from "$lib/i18n/i18n.svelte";
  import type { LocaleId } from "$lib/i18n/messages";
  import type { TriggerProfileId } from "$lib/types";

  interface Props {
    layer: number;
    connected: boolean;
    daemonActive: boolean;
    dirty: boolean;
    syncing?: boolean;
    triggerProfile: TriggerProfileId;
    osdEnabled: boolean;
    onLayerChange: (layer: number) => void;
    onProfileChange: (p: TriggerProfileId) => void;
    onRotate: () => void;
    onOsdToggle: () => void;
    onSync: () => void;
    onConnectToggle: () => void;
    onPrintLegend: () => void;
  }

  let {
    layer,
    connected,
    daemonActive,
    dirty,
    syncing = false,
    triggerProfile,
    osdEnabled,
    onLayerChange,
    onProfileChange,
    onRotate,
    onOsdToggle,
    onSync,
    onConnectToggle,
    onPrintLegend,
  }: Props = $props();

  function setLang(id: LocaleId): void {
    i18n.setLocale(id);
  }
</script>

<header
  class="flex h-[52px] shrink-0 items-center justify-between gap-2 border-b border-border-subtle bg-bg-panel px-4"
>
  <div class="flex min-w-0 items-center gap-2">
    <button
      type="button"
      class="flex items-center gap-1.5 rounded border border-border-subtle px-2 py-1 text-xs text-text-secondary hover:border-status-emerald"
      onclick={onConnectToggle}
    >
      <span
        class="inline-block h-2 w-2 rounded-full {connected
          ? 'bg-status-emerald'
          : daemonActive
            ? 'bg-accent-cyan'
            : 'bg-status-rose'}"
      ></span>
      {#if connected}
        {i18n.t("usbConnected")}
      {:else if daemonActive}
        {i18n.t("daemonActive")}
      {:else}
        {i18n.t("idle")}
      {/if}
    </button>

    <select
      class="rounded border border-border-subtle bg-bg-canvas px-2 py-1 font-mono text-[10px] text-text-secondary outline-none"
      value={triggerProfile}
      onchange={(e) =>
        onProfileChange(e.currentTarget.value as TriggerProfileId)}
    >
      <option value="wiredDongle">{i18n.t("profileWired")}</option>
      <option value="bluetooth">{i18n.t("profileBt")}</option>
    </select>
  </div>

  <div class="flex items-center gap-1 rounded border border-border-subtle p-0.5">
    {#each [1, 2, 3] as l (l)}
      <button
        type="button"
        class="rounded px-3 py-1 text-xs transition-colors {layer === l
          ? 'bg-bg-keycap text-accent-amber'
          : 'text-text-secondary hover:text-text-primary'}"
        onclick={() => onLayerChange(l)}
        title="Alt+{l}"
      >
        {i18n.t("layer")} {l}
      </button>
    {/each}
  </div>

  <div class="flex items-center gap-2">
    <div class="flex rounded border border-border-subtle p-0.5 text-[10px]">
      <button
        type="button"
        class="rounded px-1.5 py-0.5 {i18n.locale === 'ko'
          ? 'bg-bg-keycap text-accent-amber'
          : 'text-text-secondary'}"
        onclick={() => setLang("ko")}>{i18n.t("langKo")}</button
      >
      <button
        type="button"
        class="rounded px-1.5 py-0.5 {i18n.locale === 'en'
          ? 'bg-bg-keycap text-accent-amber'
          : 'text-text-secondary'}"
        onclick={() => setLang("en")}>{i18n.t("langEn")}</button
      >
    </div>
    <button
      type="button"
      class="rounded border border-border-subtle px-2 py-1 text-xs text-text-secondary hover:text-accent-cyan"
      onclick={onPrintLegend}
      title={i18n.t("printLegend")}
    >
      {i18n.t("printLegend")}
    </button>
    <button
      type="button"
      class="rounded border border-border-subtle px-2 py-1 text-xs text-text-secondary hover:text-text-primary"
      onclick={onRotate}
    >
      {i18n.t("rotate")}
    </button>
    <button
      type="button"
      class="rounded border px-2 py-1 text-xs {osdEnabled
        ? 'border-accent-amber/40 text-accent-amber'
        : 'border-border-subtle text-text-secondary opacity-70'}"
      onclick={onOsdToggle}
    >
      {i18n.t("osd")}
    </button>
    <button
      type="button"
      class="rounded border px-3 py-1 text-xs transition-colors {dirty
        ? 'border-accent-amber bg-accent-amber text-[#0B0C0E]'
        : 'border-border-subtle text-text-secondary hover:border-status-emerald hover:text-status-emerald'} {syncing
        ? 'animate-pulse'
        : ''}"
      onclick={onSync}
    >
      {syncing ? i18n.t("syncing") : i18n.t("sync")}
    </button>
  </div>
</header>
