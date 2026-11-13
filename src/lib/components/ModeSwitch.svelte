<script lang="ts">
  import { i18n } from "$lib/i18n/i18n.svelte";
  import type { InspectorTab } from "$lib/types";

  interface Props {
    tab: InspectorTab;
    onChange: (tab: InspectorTab) => void;
  }

  let { tab, onChange }: Props = $props();
</script>

<div class="flex shrink-0 gap-1 border-b border-border-subtle px-4 pt-3 text-xs">
  <button
    type="button"
    class="rounded-t border px-3 py-1.5 transition-colors {tab === 'snippet'
      ? 'border-border-subtle border-b-0 bg-bg-canvas text-accent-amber'
      : 'border-transparent text-text-secondary hover:text-accent-amber'}"
    onclick={() => onChange("snippet")}
  >
    {i18n.t("tabSnippet")}
  </button>
  <button
    type="button"
    class="rounded-t border px-3 py-1.5 transition-colors {tab === 'hardware'
      ? 'border-border-subtle border-b-0 bg-bg-canvas text-accent-cyan'
      : 'border-transparent text-text-secondary hover:text-accent-cyan'}"
    onclick={() => onChange("hardware")}
  >
    {i18n.t("tabHardware")}
  </button>
</div>
