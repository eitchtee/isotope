<script lang="ts">
  import type { Snippet } from "svelte";
  import { api } from "$lib/api";

  type Props = { title: string; onclose: () => void; children: Snippet; footer?: Snippet };

  let { title, onclose, children, footer }: Props = $props();

  // Native app webviews draw above the shell, so hide the active one while a modal is open.
  $effect(() => {
    api.setOverlayOpen(true).catch(() => {});
    return () => {
      api.setOverlayOpen(false).catch(() => {});
    };
  });

  function onKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") onclose();
  }
</script>

<svelte:window onkeydown={onKeydown} />

<div class="backdrop" role="presentation" onclick={(e) => e.target === e.currentTarget && onclose()}>
  <div class="modal" role="dialog" aria-modal="true" aria-label={title}>
    <header>
      <h2>{title}</h2>
      <button class="close" aria-label="Close" onclick={onclose}>×</button>
    </header>
    <div class="body">{@render children()}</div>
    {#if footer}
      <footer>{@render footer()}</footer>
    {/if}
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    display: grid;
    place-items: center;
    background: rgb(0 0 0 / 0.55);
    z-index: 10;
  }
  .modal {
    width: min(520px, calc(100vw - 32px));
    max-height: calc(100vh - 48px);
    display: flex;
    flex-direction: column;
    background: #26262c;
    color: #e4e4e7;
    border-radius: 14px;
    box-shadow: 0 20px 60px rgb(0 0 0 / 0.5);
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 18px;
    border-bottom: 1px solid #303038;
  }
  h2 {
    margin: 0;
    font-size: 15px;
    font-weight: 600;
  }
  .close {
    border: 0;
    background: transparent;
    color: #a1a1aa;
    font-size: 20px;
    cursor: pointer;
  }
  .body {
    padding: 16px 18px;
    overflow-y: auto;
  }
  footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 12px 18px;
    border-top: 1px solid #303038;
  }
</style>
