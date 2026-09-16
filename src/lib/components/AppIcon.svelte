<script lang="ts">
  import { badgeLabel, initials } from "$lib/sidebar";
  import type { App, AppState, Badge } from "$lib/types";

  type Props = {
    app: App;
    status: AppState;
    badge?: Badge;
    active: boolean;
    iconUrl: string | null;
    /** "small" is used inside the folder panel. */
    size?: "normal" | "small";
    onactivate: () => void;
    oncontextmenu?: (event: MouseEvent) => void;
  };

  let { app, status, badge, active, iconUrl, size = "normal", onactivate, oncontextmenu }: Props = $props();

  function handleContextMenu(event: MouseEvent) {
    if (!oncontextmenu) return;
    event.preventDefault();
    oncontextmenu(event);
  }
</script>

<button
  class="app-icon"
  class:active
  class:small={size === "small"}
  class:hibernated={status.kind === "hibernated"}
  aria-label={app.name}
  aria-current={active ? "page" : undefined}
  title={status.kind === "error" ? `${app.name}: ${status.message}` : app.name}
  onclick={onactivate}
  oncontextmenu={handleContextMenu}
>
  {#if iconUrl}
    <img src={iconUrl} alt="" draggable="false" />
  {:else}
    <span class="letters">{initials(app.name)}</span>
  {/if}

  {#if status.kind === "error"}
    <span class="warning" aria-label="Error">!</span>
  {:else if badge !== undefined}
    <span class="badge" class:dot={badge === "dot"} data-testid="badge">{badgeLabel(badge)}</span>
  {/if}
</button>

<style>
  .app-icon {
    position: relative;
    width: 44px;
    height: 44px;
    border: 0;
    border-radius: 12px;
    background: #26262c;
    color: #e4e4e7;
    display: grid;
    place-items: center;
    cursor: pointer;
    padding: 0;
    transition: background-color 120ms ease, opacity 120ms ease;
  }
  .app-icon:hover {
    background: #303038;
  }
  .app-icon.active::before {
    content: "";
    position: absolute;
    left: -10px;
    top: 10px;
    bottom: 10px;
    width: 3px;
    border-radius: 0 3px 3px 0;
    background: #7c9cff;
  }
  .app-icon.hibernated {
    opacity: 0.45;
  }
  .app-icon.small {
    width: 36px;
    height: 36px;
    border-radius: 10px;
  }
  .app-icon.small img {
    width: 22px;
    height: 22px;
  }
  .app-icon.small .letters {
    font-size: 12px;
  }
  .app-icon.small.active::before {
    top: 8px;
    bottom: 8px;
  }
  img {
    width: 28px;
    height: 28px;
    border-radius: 6px;
    object-fit: contain;
  }
  .letters {
    font-size: 14px;
    font-weight: 600;
    letter-spacing: 0.02em;
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
  .warning {
    position: absolute;
    top: -4px;
    right: -4px;
    width: 18px;
    height: 18px;
    border-radius: 9px;
    background: #facc15;
    color: #1b1b1f;
    font-size: 12px;
    font-weight: 800;
    line-height: 18px;
  }
</style>
