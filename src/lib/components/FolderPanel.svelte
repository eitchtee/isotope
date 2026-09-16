<script lang="ts">
  import { api } from "$lib/api";
  import { folderPanelDropTarget, planMove, type DropTarget } from "$lib/dnd";
  import { dragging } from "$lib/drag.svelte";
  import { icons } from "$lib/icons.svelte";
  import type { ShellStore } from "$lib/shell.svelte";
  import type { App } from "$lib/types";
  import AppIcon from "./AppIcon.svelte";

  type Props = { store: ShellStore; onappmenu: (app: App) => void };

  let { store, onappmenu }: Props = $props();

  let snapshot = $derived(store.snapshot);
  let folder = $derived(snapshot?.folderPanel ? store.folder(snapshot.folderPanel) : undefined);
  let apps = $derived(folder ? folder.appIds.map((id) => store.app(id)).filter((a): a is App => !!a) : []);
  /** Where the dragged icon would land, for the drop indicator. */
  let hint = $state<{ index: number; where: "before" | "after" } | null>(null);

  function rowTarget(event: DragEvent, folderId: string, index: number): DropTarget {
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    return folderPanelDropTarget(folderId, index, event.clientY - rect.top, rect.height);
  }

  function over(event: DragEvent, target: DropTarget, index: number) {
    if (!dragging.current || !snapshot) return;
    if (!planMove(dragging.current, target, snapshot)) {
      hint = null;
      return;
    }
    event.preventDefault();
    if (target.kind === "folder-slot") hint = { index, where: target.index === index ? "before" : "after" };
  }

  function drop(event: DragEvent, target: DropTarget) {
    event.preventDefault();
    event.stopPropagation();
    hint = null;
    const drag = dragging.current;
    dragging.current = null;
    if (!drag || !snapshot) return;
    const move = planMove(drag, target, snapshot);
    if (move) api.moveItem(move.item, move.target, move.index).catch(store.reportError);
  }
</script>

{#if folder && snapshot}
  {@const folderId = folder.id}
  {@const endSlot = { kind: "folder-slot" as const, folderId, index: apps.length }}
  <section
    class="panel"
    aria-label={folder.name}
    ondragover={(e) => {
      if (dragging.current && snapshot && planMove(dragging.current, endSlot, snapshot)) e.preventDefault();
    }}
    ondrop={(e) => drop(e, endSlot)}
  >
    <ul>
      {#each apps as app, index (app.id)}
        <li
          draggable="true"
          class:drop-before={hint?.index === index && hint.where === "before"}
          class:drop-after={hint?.index === index && hint.where === "after"}
          ondragstart={(e) => {
            dragging.current = { item: { type: "app", id: app.id }, from: { type: "folder", id: folderId }, fromIndex: index };
            e.dataTransfer?.setData("text/plain", app.id);
          }}
          ondragover={(e) => over(e, rowTarget(e, folderId, index), index)}
          ondrop={(e) => drop(e, rowTarget(e, folderId, index))}
          ondragend={() => {
            dragging.current = null;
            hint = null;
          }}
        >
          <AppIcon
            {app}
            status={store.state(app.id)}
            badge={store.badge(app.id)}
            active={snapshot.activeAppId === app.id}
            iconUrl={icons.url(app.id, app.icon)}
            size="small"
            onactivate={() => api.activateApp(app.id).catch(store.reportError)}
            oncontextmenu={() => onappmenu(app)}
          />
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
    width: 56px;
    background: #1f1f25;
    border-right: 1px solid #26262c;
    box-sizing: border-box;
    overflow-y: auto;
    scrollbar-width: none;
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 10px 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
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
    top: -5px;
  }
  li.drop-after::after {
    bottom: -5px;
  }
</style>
