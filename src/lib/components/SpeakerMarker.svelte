<script lang="ts">
  type Props = { state: "playing" | "muted" };

  let { state }: Props = $props();
</script>

<span class="speaker" class:muted={state === "muted"} data-testid="audio" data-state={state}>
  <svg viewBox="0 0 16 16" aria-hidden="true">
    <path d="M2 6h2.5L8 3v10L4.5 10H2z" fill="currentColor" />
    {#if state === "muted"}
      <path d="M10.5 6l4 4m0-4l-4 4" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
    {:else}
      <path
        d="M10.5 5.5a3.5 3.5 0 0 1 0 5M12.5 3.5a6.3 6.3 0 0 1 0 9"
        fill="none"
        stroke="currentColor"
        stroke-width="1.4"
        stroke-linecap="round"
      />
    {/if}
  </svg>
</span>

<style>
  .speaker {
    position: absolute;
    right: -3px;
    bottom: -3px;
    width: 16px;
    height: 16px;
    border-radius: 8px;
    display: grid;
    place-items: center;
    background: #1b1b1f;
    color: #7c9cff;
    box-shadow: 0 0 0 1px #303038;
  }
  .speaker.muted {
    color: #a1a1aa;
  }
  svg {
    width: 11px;
    height: 11px;
  }
</style>
