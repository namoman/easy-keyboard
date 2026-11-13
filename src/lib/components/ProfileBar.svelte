<script lang="ts">
  import { i18n } from "$lib/i18n/i18n.svelte";
  import type { ProfileRule, ProfileStatus } from "$lib/types";

  interface Props {
    status: ProfileStatus | null;
    onPin: (locked: boolean) => void;
    onSwitch: (id: string) => void;
    onUpsert: (rule: ProfileRule) => void;
    onDelete: (id: string) => void;
    onExportEkb: (path: string, password: string) => void;
    onImportEkb: (path: string, password: string) => void;
    onExportClean: (path: string) => void;
    onImportClean: (path: string) => void;
  }

  let {
    status,
    onPin,
    onSwitch,
    onUpsert,
    onDelete,
    onExportEkb,
    onImportEkb,
    onExportClean,
    onImportClean,
  }: Props = $props();

  let showEditor = $state(false);
  let showBundle = $state(false);
  let editId = $state("");
  let editName = $state("");
  let editPriority = $state(10);
  let editExe = $state("");
  let editTitle = $state("");
  let bundlePath = $state("");
  let bundlePassword = $state("");

  function openNew(): void {
    editId = "";
    editName = "";
    editPriority = 10;
    editExe = status?.foreground.exe ?? "";
    editTitle = "";
    showEditor = true;
  }

  function openEdit(rule: ProfileRule): void {
    editId = rule.profileId;
    editName = rule.name;
    editPriority = rule.priority;
    editExe = rule.matchRules.executables.join(", ");
    editTitle = rule.matchRules.titleContains.join(", ");
    showEditor = true;
  }

  function commitRule(): void {
    const id =
      editId.trim() ||
      editName
        .trim()
        .toLowerCase()
        .replace(/\s+/g, "-")
        .replace(/[^a-z0-9-_]/g, "") ||
      `p-${Date.now()}`;
    onUpsert({
      profileId: id,
      name: editName.trim() || id,
      priority: editPriority,
      matchRules: {
        executables: editExe
          .split(",")
          .map((s) => s.trim())
          .filter(Boolean),
        titleContains: editTitle
          .split(",")
          .map((s) => s.trim())
          .filter(Boolean),
      },
    });
    showEditor = false;
  }
</script>

<div
  class="flex h-8 shrink-0 items-center gap-2 border-b border-border-subtle bg-bg-panel px-3 text-[11px]"
>
  <span class="font-mono text-text-secondary">{i18n.t("profile")}</span>
  <select
    class="max-w-[140px] rounded border border-border-subtle bg-bg-canvas px-1.5 py-0.5 font-mono text-[10px] outline-none"
    value={status?.activeProfileId ?? "default"}
    onchange={(e) => onSwitch(e.currentTarget.value)}
  >
    {#each status?.profiles ?? [] as p (p.profileId)}
      <option value={p.profileId}>{p.name}</option>
    {/each}
  </select>
  <button
    type="button"
    class="rounded border px-1.5 py-0.5 {status?.pinLocked
      ? 'border-accent-amber text-accent-amber'
      : 'border-border-subtle text-text-secondary'}"
    title={i18n.t("pinTitle")}
    onclick={() => onPin(!(status?.pinLocked ?? false))}
  >
    {status?.pinLocked ? i18n.t("pinOn") : i18n.t("pinOff")}
  </button>
  <span class="truncate font-mono text-[10px] text-text-secondary" title={status?.foreground.title}>
    {status?.foreground.exe || "—"}
  </span>
  <div class="ml-auto flex items-center gap-1">
    <button
      type="button"
      class="rounded border border-border-subtle px-1.5 py-0.5 text-text-secondary hover:text-accent-cyan"
      onclick={openNew}
    >
      {i18n.t("addRule")}
    </button>
    <button
      type="button"
      class="rounded border border-border-subtle px-1.5 py-0.5 text-text-secondary hover:text-status-emerald"
      onclick={() => (showBundle = !showBundle)}
    >
      {i18n.t("ekb")}
    </button>
  </div>
</div>

{#if showEditor}
  <div class="border-b border-border-subtle bg-bg-canvas px-3 py-2 text-[11px]">
    <div class="mb-1 flex gap-2">
      <input
        class="flex-1 rounded border border-border-subtle bg-bg-panel px-2 py-1 outline-none focus:border-accent-amber"
        placeholder={i18n.t("namePlaceholder")}
        bind:value={editName}
      />
      <input
        class="w-16 rounded border border-border-subtle bg-bg-panel px-2 py-1 font-mono outline-none"
        type="number"
        title="priority"
        bind:value={editPriority}
      />
    </div>
    <input
      class="mb-1 w-full rounded border border-border-subtle bg-bg-panel px-2 py-1 font-mono outline-none"
      placeholder={i18n.t("exePlaceholder")}
      bind:value={editExe}
    />
    <input
      class="mb-1 w-full rounded border border-border-subtle bg-bg-panel px-2 py-1 outline-none"
      placeholder={i18n.t("titlePlaceholder")}
      bind:value={editTitle}
    />
    <div class="flex flex-wrap gap-1">
      {#each (status?.profiles ?? []).filter((p) => p.profileId !== "default") as p (p.profileId)}
        <button
          type="button"
          class="rounded border border-border-subtle px-1.5 py-0.5 hover:border-accent-cyan"
          onclick={() => openEdit(p)}>{p.name}</button
        >
        <button
          type="button"
          class="rounded border border-status-rose/40 px-1 text-status-rose"
          onclick={() => onDelete(p.profileId)}>×</button
        >
      {/each}
      <div class="ml-auto flex gap-1">
        <button
          type="button"
          class="rounded border border-border-subtle px-2 py-0.5"
          onclick={() => (showEditor = false)}>{i18n.t("cancel")}</button
        >
        <button
          type="button"
          class="rounded border border-accent-amber bg-accent-amber px-2 py-0.5 text-[#0B0C0E]"
          onclick={commitRule}>{i18n.t("save")}</button
        >
      </div>
    </div>
  </div>
{/if}

{#if showBundle}
  <div class="border-b border-border-subtle bg-bg-canvas px-3 py-2 text-[11px]">
    <input
      class="mb-1 w-full rounded border border-border-subtle bg-bg-panel px-2 py-1 font-mono outline-none"
      placeholder={i18n.t("pathPlaceholder")}
      bind:value={bundlePath}
    />
    <input
      class="mb-1 w-full rounded border border-border-subtle bg-bg-panel px-2 py-1 outline-none"
      type="password"
      placeholder={i18n.t("passwordPlaceholder")}
      bind:value={bundlePassword}
    />
    <div class="flex flex-wrap gap-1">
      <button
        type="button"
        class="rounded border border-status-emerald/50 px-2 py-0.5 text-status-emerald"
        onclick={() => onExportEkb(bundlePath, bundlePassword)}
        >{i18n.t("exportEkb")}</button
      >
      <button
        type="button"
        class="rounded border border-accent-cyan/50 px-2 py-0.5 text-accent-cyan"
        onclick={() => onImportEkb(bundlePath, bundlePassword)}
        >{i18n.t("importEkb")}</button
      >
      <button
        type="button"
        class="rounded border border-border-subtle px-2 py-0.5"
        onclick={() =>
          onExportClean(bundlePath.replace(/\.ekb$/i, "") + ".json")}
        >{i18n.t("cleanJson")}</button
      >
      <button
        type="button"
        class="rounded border border-border-subtle px-2 py-0.5"
        onclick={() => onImportClean(bundlePath)}
        >{i18n.t("importJson")}</button
      >
    </div>
  </div>
{/if}
