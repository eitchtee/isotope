<script lang="ts">
  import Modal from "./Modal.svelte";

  type Props = {
    title: string;
    label: string;
    initial: string;
    confirmText: string;
    onconfirm: (value: string) => void | Promise<void>;
    onclose: () => void;
  };

  let { title, label, initial, confirmText, onconfirm, onclose }: Props = $props();
  // svelte-ignore state_referenced_locally
  let value = $state(initial);

  async function confirm() {
    const trimmed = value.trim();
    if (!trimmed) return;
    await onconfirm(trimmed);
    onclose();
  }
</script>

<Modal {title} {onclose}>
  <form
    id="prompt-form"
    onsubmit={(e) => {
      e.preventDefault();
      confirm();
    }}
  >
    <label for="prompt-input">{label}</label>
    <!-- svelte-ignore a11y_autofocus -->
    <input id="prompt-input" bind:value autocomplete="off" autofocus />
  </form>
  {#snippet footer()}
    <button type="button" onclick={onclose}>Cancel</button>
    <button type="submit" form="prompt-form" class="primary">{confirmText}</button>
  {/snippet}
</Modal>

<style>
  form {
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 13px;
  }
  input {
    padding: 7px 9px;
    border: 1px solid #3a3a44;
    border-radius: 8px;
    background: #1b1b1f;
    color: #e4e4e7;
    font: inherit;
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
