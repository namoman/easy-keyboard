<script lang="ts">
  import { i18n } from "$lib/i18n/i18n.svelte";

  interface Props {
    message?: string;
    charCount?: number;
    byteHint?: string;
    elevatedForeground?: boolean;
    usbSyncPending?: boolean;
  }

  let {
    message = "",
    charCount = 0,
    byteHint = "",
    elevatedForeground = false,
    usbSyncPending = false,
  }: Props = $props();
</script>

<footer
  class="flex h-[28px] shrink-0 items-center justify-between gap-2 border-t border-border-subtle bg-bg-panel px-4 font-mono text-[10px] text-text-secondary"
>
  <div class="flex min-w-0 items-center gap-2 truncate">
    {#if elevatedForeground}
      <span
        class="shrink-0 rounded border border-status-rose/50 px-1.5 py-0.5 text-status-rose"
        title="UIPI"
      >
        {i18n.t("elevatedMismatch")}
      </span>
    {/if}
    {#if usbSyncPending}
      <span class="shrink-0 rounded border border-accent-amber/40 px-1.5 py-0.5 text-accent-amber">
        {i18n.t("usbPending")}
      </span>
    {/if}
    <span class="truncate">{message}</span>
  </div>
  <span class="shrink-0">
    {#if charCount > 0}
      {i18n.t("charCount")}: {charCount}{byteHint ? ` (${byteHint})` : ""}
    {:else}
      —
    {/if}
  </span>
</footer>
