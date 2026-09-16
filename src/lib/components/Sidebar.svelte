<script lang="ts">
  import { api } from "$lib/api";
  import { planMove, sidebarDropTarget, type DropTarget } from "$lib/dnd";
  import { dragging } from "$lib/drag.svelte";
  import { icons } from "$lib/icons.svelte";
  import type { ShellStore } from "$lib/shell.svelte";
  import { appAudio, folderAudio, folderBadge, resolveSidebar, type ResolvedItem } from "$lib/sidebar";
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
  /** Where the dragged icon would land, for the drop indicator. */
  let hint = $state<{ index: number; where: "before" | "after" | "into" } | null>(null);

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
    if (!dragging.current || !snapshot) return;
    const target = dropTarget(event, item, index);
    if (!planMove(dragging.current, target, snapshot)) {
      hint = null;
      return;
    }
    event.preventDefault();
    if (target.kind === "into-folder") hint = { index, where: "into" };
    else if (target.kind === "sidebar-slot") hint = { index, where: target.index === index ? "before" : "after" };
  }

  function onDragEnd() {
    dragging.current = null;
    hint = null;
  }

  function onDrop(event: DragEvent, item: ResolvedItem, index: number) {
    event.preventDefault();
    hint = null;
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
          class:drop-before={hint?.index === index && hint.where === "before"}
          class:drop-after={hint?.index === index && hint.where === "after"}
          class:drop-into={hint?.index === index && hint.where === "into"}
          ondragstart={(e) => onDragStart(e, item, index)}
          ondragover={(e) => onDragOver(e, item, index)}
          ondrop={(e) => onDrop(e, item, index)}
          ondragend={onDragEnd}
        >
          {#if item.type === "app"}
            <AppIcon
              app={item.app}
              status={store.state(item.id)}
              badge={store.badge(item.id)}
              active={snapshot.activeAppId === item.id}
              iconUrl={icons.url(item.id, item.app.icon)}
              audio={appAudio(snapshot, item.id)}
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
              audio={folderAudio(snapshot, item.folder)}
              ontoggle={() => api.openFolder(item.id).catch(store.reportError)}
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
  li {
    position: relative;
  }
  li.drop-before::before,
  li.drop-after::after {
    content: "";
    position: absolute;
    left: 2px;
    right: 2px;
    height: 2px;
    border-radius: 1px;
    background: #7c9cff;
  }
  li.drop-before::before {
    top: -6px;
  }
  li.drop-after::after {
    bottom: -6px;
  }
  li.drop-into :global(.folder-icon) {
    outline: 2px solid #7c9cff;
    outline-offset: 2px;
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
