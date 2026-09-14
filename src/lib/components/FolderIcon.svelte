<script lang="ts">
  import { badgeLabel, initials } from "$lib/sidebar";
  import type { App, Badge, Folder } from "$lib/types";

  type Props = {
    folder: Folder;
    apps: App[];
    badge?: Badge;
    open: boolean;
    iconUrl: (app: App) => string | null;
    ontoggle: () => void;
    oncontextmenu?: (event: MouseEvent) => void;
  };

  let { folder, apps, badge, open, iconUrl, ontoggle, oncontextmenu }: Props = $props();

  function handleContextMenu(event: MouseEvent) {
    if (!oncontextmenu) return;
    event.preventDefault();
    oncontextmenu(event);
  }
</script>

<button
  class="folder-icon"
  class:open
  aria-label={`Folder ${folder.name}`}
  aria-expanded={open}
  title={folder.name}
  onclick={ontoggle}
  oncontextmenu={handleContextMenu}
>
  <span class="grid">
    {#each apps.slice(0, 4) as app (app.id)}
      {@const url = iconUrl(app)}
      <span class="mini">
        {#if url}
          <img src={url} alt="" draggable="false" />
        {:else}
          {initials(app.name).slice(0, 1)}
        {/if}
      </span>
    {/each}
  </span>

  {#if badge !== undefined}
    <span class="badge" class:dot={badge === "dot"} data-testid="badge">{badgeLabel(badge)}</span>
  {/if}
</button>

<style>
  .folder-icon {
    position: relative;
    width: 44px;
    height: 44px;
    border: 1px solid #303038;
    border-radius: 12px;
    background: #1f1f25;
    padding: 5px;
    box-sizing: border-box;
    cursor: pointer;
  }
  .folder-icon.open {
    border-color: #7c9cff;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 2px;
    width: 100%;
    height: 100%;
  }
  .mini {
    display: grid;
    place-items: center;
    border-radius: 4px;
    background: #2c2c34;
    color: #a1a1aa;
    font-size: 8px;
    font-weight: 700;
    overflow: hidden;
  }
  .mini img {
    width: 12px;
    height: 12px;
    object-fit: contain;
  }
  .badge {
    position: absolute;
    top: -4px;
    right: -4px;
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
    font-variant-numeric: tabular-nums;
  }
  .badge.dot {
    min-width: 10px;
    width: 10px;
    height: 10px;
    top: -2px;
    right: -2px;
    padding: 0;
  }
</style>
