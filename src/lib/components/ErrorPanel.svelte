<script lang="ts">
  import { api } from "$lib/api";
  import type { ShellStore } from "$lib/shell.svelte";
  import type { App } from "$lib/types";

  type Props = { store: ShellStore; onedit: (app: App) => void; onaddapp: () => void };

  let { store, onedit, onaddapp }: Props = $props();

  const snapshot = $derived(store.snapshot);
  const active = $derived(snapshot?.activeAppId ? store.app(snapshot.activeAppId) : undefined);
  const status = $derived(active ? store.state(active.id) : undefined);
</script>

{#if snapshot && snapshot.config.apps.length === 0}
  <div class="panel">
    <h2>No apps yet</h2>
    <p>Add a web app to keep it one click away, with its own profile and hibernation.</p>
    <button class="primary" onclick={onaddapp}>Add your first app</button>
  </div>
{:else if active && status?.kind === "error"}
  <div class="panel">
    <h2>Couldn't load {active.name}</h2>
    <p class="message">{status.message}</p>
    <div class="actions">
      <button class="primary" onclick={() => api.wakeApp(active.id).catch(store.reportError)}>Retry</button>
      <button onclick={() => onedit(active)}>Edit</button>
    </div>
  </div>
{/if}

<style>
  .panel {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    padding: 24px;
    text-align: center;
    color: #e4e4e7;
  }
  h2 {
    margin: 0;
    font-size: 17px;
    font-weight: 600;
  }
  p {
    margin: 0;
    max-width: 420px;
    color: #a1a1aa;
    font-size: 13px;
  }
  .message {
    font-family: ui-monospace, monospace;
  }
  .actions {
    display: flex;
    gap: 8px;
  }
  button {
    padding: 7px 14px;
    border: 0;
    border-radius: 8px;
    background: #303038;
    color: #e4e4e7;
    font: inherit;
    cursor: pointer;
  }
  .primary {
    background: #7c9cff;
    color: #1b1b1f;
    font-weight: 600;
  }
</style>
