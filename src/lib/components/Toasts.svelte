<script lang="ts">
  import { api } from "$lib/api";
  import type { ShellStore } from "$lib/shell.svelte";

  type Props = { store: ShellStore };

  let { store }: Props = $props();

  const current = $derived(store.toasts[0]);
  const visible = $derived(store.toasts.length > 0);
  let reported: boolean | null = null;

  // Rust shrinks the app webview by the strip height while a toast is showing.
  $effect(() => {
    if (visible === reported || (reported === null && !visible)) return;
    reported = visible;
    api.setToastVisible(visible).catch(() => {});
  });

  $effect(() => {
    if (!current) return;
    const id = current.id;
    const timer = setTimeout(() => store.dismissToast(id), 5000);
    return () => clearTimeout(timer);
  });
</script>

{#if current}
  <div class="strip {current.level}" role="status">
    <span class="message">{current.message}</span>
    <button aria-label="Dismiss" onclick={() => store.dismissToast(current.id)}>×</button>
  </div>
{/if}

<style>
  .strip {
    position: absolute;
    left: 0;
    right: 0;
    bottom: 0;
    height: 32px;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 12px;
    box-sizing: border-box;
    font-size: 13px;
    background: #26262c;
    color: #e4e4e7;
    border-top: 2px solid #7c9cff;
  }
  .strip.warning {
    border-top-color: #facc15;
  }
  .strip.error {
    border-top-color: #f87171;
  }
  .message {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  button {
    border: 0;
    background: transparent;
    color: #a1a1aa;
    font-size: 18px;
    cursor: pointer;
  }
</style>
