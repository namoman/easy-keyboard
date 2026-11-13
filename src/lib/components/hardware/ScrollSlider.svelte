<script lang="ts">
  interface Props {
    value: number;
    onChange: (v: number) => void;
  }

  let { value, onChange }: Props = $props();
</script>

<label class="flex flex-col gap-1 text-xs text-text-secondary">
  스크롤 배속
  <div class="flex items-center gap-3">
    <input
      type="range"
      min="1"
      max="5"
      step="1"
      class="h-1 flex-1 accent-cyan-500"
      value={value}
      oninput={(e) => onChange(Number(e.currentTarget.value))}
    />
    <span class="w-10 font-mono text-[11px] text-accent-cyan">{value}x</span>
  </div>
</label>
