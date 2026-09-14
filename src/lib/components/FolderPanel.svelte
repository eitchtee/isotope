<script lang="ts">
  import { api } from "$lib/api";
  import { folderPanelDropTarget, planMove, type DropTarget } from "$lib/dnd";
  import { dragging } from "$lib/drag.svelte";
  import { icons } from "$lib/icons.svelte";
  import type { ShellStore } from "$lib/shell.svelte";
  import { badgeLabel, initials } from "$lib/sidebar";
  import type { App } from "$lib/types";

  type Props = { store: ShellStore; onappmenu: (app: App) => void };

  let { store, onappmenu }: Props = $props();

  let snapshot = $derived(store.snapshot);
  let folder = $derived(snapshot?.folderPanel ? store.folder(snapshot.folderPanel) : undefined);
  let apps = $derived(folder ? folder.appIds.map((id) => store.app(id)).filter((a): a is App => !!a) : []);

  function drop(event: DragEvent, target: DropTarget) {
    event.preventDefault();
    event.stopPropagation();
    const drag = dragging.current;
    dragging.current = null;
    if (!drag || !snapshot) return;
    const move = planMove(drag, target, snapshot);
    if (move) api.moveItem(move.item, move.target, move.index).catch(store.reportError);
  }

  function rowTarget(event: DragEvent, folderId: string, index: number): DropTarget {
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    return folderPanelDropTarget(folderId, index, event.clientY - rect.top, rect.height);
  }

  function contextMenu(event: MouseEvent, app: App) {
    event.preventDefault();
    onappmenu(app);
  }
</script>

{#if folder && snapshot}
  {@const folderId = folder.id}
  <section
    class="panel"
    aria-label={folder.name}
    ondragover={(e) => dragging.current && e.preventDefault()}
    ondrop={(e) => drop(e, { kind: "folder-slot", folderId, index: apps.length })}
  >
    <h2>{folder.name}</h2>
    <ul>
      {#each apps as app, index (app.id)}
        {@const status = store.state(app.id)}
        {@const badge = store.badge(app.id)}
        {@const url = icons.url(app.id, app.icon)}
        <li
          draggable="true"
          ondragstart={(e) => {
            dragging.current = { item: { type: "app", id: app.id }, from: { type: "folder", id: folderId }, fromIndex: index };
            e.dataTransfer?.setData("text/plain", app.id);
          }}
          ondragover={(e) => {
            if (dragging.current && snapshot && planMove(dragging.current, rowTarget(e, folderId, index), snapshot)) {
              e.preventDefault();
            }
          }}
          ondrop={(e) => drop(e, rowTarget(e, folderId, index))}
          ondragend={() => (dragging.current = null)}
        >
          <button
            class="row"
            class:active={snapshot.activeAppId === app.id}
            class:hibernated={status.kind === "hibernated"}
            aria-label={app.name}
            onclick={() => api.activateApp(app.id).catch(store.reportError)}
            oncontextmenu={(e) => contextMenu(e, app)}
          >
            <span class="icon">
              {#if url}<img src={url} alt="" draggable="false" />{:else}{initials(app.name)}{/if}
            </span>
            <span class="name">{app.name}</span>
            {#if status.kind === "error"}
              <span class="warning">!</span>
            {:else if badge !== undefined}
              <span class="badge" class:dot={badge === "dot"}>{badgeLabel(badge)}</span>
            {/if}
          </button>
        </li>
      {/each}
    </ul>
  </section>
{/if}

<style>
  .panel {
    position: fixed;
    top: 0;
    bottom: 0;
    left: 64px;
    width: 200px;
    background: #1f1f25;
    border-right: 1px solid #26262c;
    box-sizing: border-box;
    padding: 12px 8px;
    overflow-y: auto;
  }
  h2 {
    margin: 0 8px 10px;
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: #a1a1aa;
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .row {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 6px 8px;
    border: 0;
    border-radius: 8px;
    background: transparent;
    color: #e4e4e7;
    font: inherit;
    font-size: 13px;
    text-align: left;
    cursor: pointer;
  }
  .row:hover {
    background: #26262c;
  }
  .row.active {
    background: #2c2c34;
  }
  .row.hibernated {
    opacity: 0.5;
  }
  .icon {
    width: 24px;
    height: 24px;
    flex: none;
    display: grid;
    place-items: center;
    border-radius: 6px;
    background: #2c2c34;
    font-size: 10px;
    font-weight: 700;
  }
  .icon img {
    width: 18px;
    height: 18px;
    object-fit: contain;
  }
  .name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .badge {
    min-width: 18px;
    height: 18px;
    padding: 0 5px;
    box-sizing: border-box;
    border-radius: 9px;
    background: #f87171;
    color: #1b1b1f;
    font-size: 11px;
    font-weight: 700;
    line-height: 18px;
    text-align: center;
    font-variant-numeric: tabular-nums;
  }
  .badge.dot {
    min-width: 8px;
    width: 8px;
    height: 8px;
    padding: 0;
  }
  .warning {
    color: #facc15;
    font-weight: 800;
  }
</style>
