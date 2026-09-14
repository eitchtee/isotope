<script lang="ts">
  import Modal from "./Modal.svelte";

  type Props = {
    title: string;
    message: string;
    confirmText: string;
    onconfirm: () => void | Promise<void>;
    onclose: () => void;
  };

  let { title, message, confirmText, onconfirm, onclose }: Props = $props();

  async function confirm() {
    await onconfirm();
    onclose();
  }
</script>

<Modal {title} {onclose}>
  <p>{message}</p>
  {#snippet footer()}
    <button type="button" onclick={onclose}>Cancel</button>
    <button type="button" class="danger" onclick={confirm}>{confirmText}</button>
  {/snippet}
</Modal>

<style>
  p {
    margin: 0;
    font-size: 13px;
    color: #a1a1aa;
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
  .danger {
    background: #f87171;
    color: #1b1b1f;
    font-weight: 600;
  }
</style>
