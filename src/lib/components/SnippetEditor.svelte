<script lang="ts">
  import { getClipboardText } from "$lib/ipc";
  import type { InjectionMode, PostPasteAction, SnippetSlot } from "$lib/types";

  interface Props {
    slot: SnippetSlot;
    onChange: (slot: SnippetSlot) => void;
  }

  let { slot, onChange }: Props = $props();

  let bodyEl = $state<HTMLTextAreaElement | null>(null);
  let secretHover = $state(false);

  const isEmpty = $derived(slot.body.trim().length === 0);
  const showTypingDelay = $derived(slot.injectionMode === "simulatedTyping");

  const chips = [
    "{{date}}",
    "{{time}}",
    "{{clipboard}}",
    "{{cursor}}",
    "{{tab}}",
    "{{enter}}",
    "{{wait:500}}",
  ];

  const modes: { id: InjectionMode; label: string }[] = [
    { id: "clipboardPlain", label: "Ctrl+V" },
    { id: "clipboardTerminal", label: "터미널" },
    { id: "simulatedTyping", label: "타이핑" },
  ];

  function patch(partial: Partial<SnippetSlot>): void {
    onChange({ ...slot, ...partial });
  }

  async function registerFromClipboard(): Promise<void> {
    const text = await getClipboardText();
    if (!text) return;
    const label =
      slot.label.trim() ||
      text
        .split("\n")[0]
        .replace(/\s+/g, "")
        .slice(0, 8);
    patch({ body: text, label });
  }

  function insertChip(token: string): void {
    const el = bodyEl;
    if (!el) {
      patch({ body: `${slot.body}${token}` });
      return;
    }
    const start = el.selectionStart ?? slot.body.length;
    const end = el.selectionEnd ?? start;
    const next = slot.body.slice(0, start) + token + slot.body.slice(end);
    patch({ body: next });
    queueMicrotask(() => {
      el.focus();
      const caret = start + token.length;
      el.setSelectionRange(caret, caret);
    });
  }

  function setPostPaste(action: PostPasteAction): void {
    patch({ postPaste: action });
  }
</script>

<div class="flex h-full flex-col gap-3 p-4">
  <div class="flex items-center justify-between">
    <h2 class="text-sm font-medium text-text-primary">
      슬롯: Key {String(slot.key).padStart(2, "0")} (Layer {slot.layer})
    </h2>
    <span class="font-mono text-[10px] text-accent-amber">
      트리거: {slot.trigger || "미등록"}
    </span>
  </div>

  <label class="flex flex-col gap-1 text-xs text-text-secondary">
    버튼 라벨
    <input
      class="rounded border border-border-subtle bg-bg-canvas px-2 py-1.5 text-sm text-text-primary outline-none focus:border-accent-amber"
      value={slot.label}
      oninput={(e) => patch({ label: e.currentTarget.value })}
      placeholder="본문 첫 8자 자동"
    />
  </label>

  <div class="flex flex-wrap gap-1.5">
    {#each chips as token (token)}
      <button
        type="button"
        class="rounded border border-border-subtle px-2 py-1 font-mono text-[10px] text-accent-amber hover:bg-bg-keycap"
        onclick={() => insertChip(token)}>+ {token}</button
      >
    {/each}
  </div>

  <div class="flex flex-wrap items-center gap-2 text-xs text-text-secondary">
    <span>주입</span>
    {#each modes as m (m.id)}
      <button
        type="button"
        class="rounded border px-2 py-0.5 {slot.injectionMode === m.id
          ? 'border-accent-cyan text-accent-cyan'
          : 'border-border-subtle'}"
        onclick={() => patch({ injectionMode: m.id })}
      >
        {m.label}
      </button>
    {/each}
    {#if showTypingDelay}
      <label class="ml-1 flex items-center gap-1 font-mono text-[10px]">
        delay
        <input
          type="number"
          min="5"
          max="20"
          class="w-12 rounded border border-border-subtle bg-bg-canvas px-1 py-0.5"
          value={slot.typingDelayMs || 10}
          oninput={(e) =>
            patch({
              typingDelayMs: Math.min(
                20,
                Math.max(5, Number(e.currentTarget.value) || 10),
              ),
            })}
        />
        ms
      </label>
    {/if}
  </div>

  {#if isEmpty}
    <button
      type="button"
      class="flex min-h-[160px] flex-1 flex-col items-center justify-center gap-2 rounded-lg border border-dashed border-border-subtle bg-bg-canvas/50 text-sm text-text-secondary hover:border-accent-amber hover:text-accent-amber"
      onclick={registerFromClipboard}
    >
      <span class="text-2xl opacity-60">📋</span>
      <span>클릭하여 현재 클립보드 텍스트 바로 등록</span>
      <span class="font-mono text-[10px]">Ctrl+V 한 방에 저장 흐름</span>
    </button>
  {:else}
    <textarea
      bind:this={bodyEl}
      class="min-h-[160px] flex-1 resize-none rounded-lg border border-border-subtle bg-bg-canvas p-3 font-mono text-[13px] text-text-primary outline-none focus:border-accent-amber {slot.secret &&
      !secretHover
        ? 'blur-sm'
        : ''}"
      value={slot.body}
      oninput={(e) => patch({ body: e.currentTarget.value })}
      onmouseenter={() => (secretHover = true)}
      onmouseleave={() => (secretHover = false)}
    ></textarea>
  {/if}

  <div class="flex flex-wrap items-center gap-3">
    <label class="flex items-center gap-2 text-xs text-text-secondary">
      <input
        type="checkbox"
        checked={slot.secret}
        onchange={(e) => patch({ secret: e.currentTarget.checked })}
      />
      민감 텍스트 (Secret)
    </label>

    <div class="flex items-center gap-1 text-xs text-text-secondary">
      <span>붙여넣기 후</span>
      {#each [
        { id: "none" as const, label: "없음" },
        { id: "enter" as const, label: "Enter ↵" },
        { id: "tab" as const, label: "Tab ⇥" },
      ] as opt (opt.id)}
        <button
          type="button"
          class="rounded border px-2 py-0.5 {slot.postPaste === opt.id
            ? 'border-accent-amber text-accent-amber'
            : 'border-border-subtle'}"
          onclick={() => setPostPaste(opt.id)}
        >
          {opt.label}
        </button>
      {/each}
    </div>
  </div>
</div>
