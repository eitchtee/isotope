<script lang="ts">
  import { api } from "$lib/api";
  import { planMove, sidebarDropTarget, type DropTarget } from "$lib/dnd";
  import { dragging } from "$lib/drag.svelte";
  import { icons } from "$lib/icons.svelte";
  import type { ShellStore } from "$lib/shell.svelte";
  import { folderBadge, resolveSidebar, type ResolvedItem } from "$lib/sidebar";
  import type { App, Folder, SidebarItem } from "$lib/types";
  import AppIcon from "./AppIcon.svelte";
  import FolderIcon from "./FolderIcon.svelte";

  type Props = {
    store: ShellStore;
    onaddapp: () => void;
    onsettings: () => void;
    onappmenu: (app: App) => void;
    onfoldermenu: (folder: Folder) => void;
  };

  let { store, onaddapp, onsettings, onappmenu, onfoldermenu }: Props = $props();

  let snapshot = $derived(store.snapshot);
  let items = $derived(snapshot ? resolveSidebar(snapshot) : []);

  function asSidebarItem(item: ResolvedItem): SidebarItem {
    return item.type === "app" ? { type: "app", id: item.id } : { type: "folder", id: item.id };
  }

  function dropTarget(event: DragEvent, item: ResolvedItem, index: number): DropTarget {
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    return sidebarDropTarget(index, item, event.clientY - rect.top, rect.height);
  }

  function onDragStart(event: DragEvent, item: ResolvedItem, index: number) {
    dragging.current = { item: asSidebarItem(item), from: { type: "sidebar" }, fromIndex: index };
    event.dataTransfer?.setData("text/plain", item.id);
  }

  function onDragOver(event: DragEvent, item: ResolvedItem, index: number) {
    if (dragging.current && snapshot && planMove(dragging.current, dropTarget(event, item, index), snapshot)) {
      event.preventDefault();
    }
  }

  function onDrop(event: DragEvent, item: ResolvedItem, index: number) {
    event.preventDefault();
    const drag = dragging.current;
    dragging.current = null;
    if (!drag || !snapshot) return;
    const move = planMove(drag, dropTarget(event, item, index), snapshot);
    if (move) api.moveItem(move.item, move.target, move.index).catch(store.reportError);
  }
</script>

{#if snapshot}
  <nav class="sidebar" aria-label="Apps">
    <ul class="items">
      {#each items as item, index (item.type + ":" + item.id)}
        <li
          draggable="true"
          ondragstart={(e) => onDragStart(e, item, index)}
          ondragover={(e) => onDragOver(e, item, index)}
          ondrop={(e) => onDrop(e, item, index)}
          ondragend={() => (dragging.current = null)}
        >
          {#if item.type === "app"}
            <AppIcon
              app={item.app}
              status={store.state(item.id)}
              badge={store.badge(item.id)}
              active={snapshot.activeAppId === item.id}
              iconUrl={icons.url(item.id, item.app.icon)}
              onactivate={() => api.activateApp(item.id).catch(store.reportError)}
              oncontextmenu={() => onappmenu(item.app)}
            />
          {:else}
            <FolderIcon
              folder={item.folder}
              apps={item.apps}
              badge={folderBadge(snapshot, item.folder)}
              open={snapshot.folderPanel === item.id}
              iconUrl={(app) => icons.url(app.id, app.icon)}
              ontoggle={() => api.toggleFolderPanel(item.id).catch(store.reportError)}
              oncontextmenu={() => onfoldermenu(item.folder)}
            />
          {/if}
        </li>
      {/each}
    </ul>

    <div class="actions">
      <button class="action" aria-label="Add app" title="Add app" onclick={onaddapp}>+</button>
      <button class="action" aria-label="Settings" title="Settings" onclick={onsettings}>⚙</button>
    </div>
  </nav>
{/if}

<style>
  .sidebar {
    position: fixed;
    inset: 0 auto 0 0;
    width: 64px;
    display: flex;
    flex-direction: column;
    background: #1b1b1f;
    border-right: 1px solid #26262c;
    box-sizing: border-box;
  }
  .items {
    flex: 1;
    list-style: none;
    margin: 0;
    padding: 10px 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    overflow-y: auto;
    scrollbar-width: none;
  }
  .actions {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    padding: 10px 0;
  }
  .action {
    width: 36px;
    height: 36px;
    border: 0;
    border-radius: 10px;
    background: transparent;
    color: #a1a1aa;
    font-size: 18px;
    cursor: pointer;
  }
  .action:hover {
    background: #26262c;
    color: #e4e4e7;
  }
</style>
