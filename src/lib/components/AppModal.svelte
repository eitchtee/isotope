<script lang="ts">
  import { api } from "$lib/api";
  import { formFromApp, validateAppForm, type AppFormErrors } from "$lib/appForm";
  import type { ShellStore } from "$lib/shell.svelte";
  import { DEFAULT_PROFILE_ID, type App } from "$lib/types";
  import Modal from "./Modal.svelte";

  type Props = { store: ShellStore; app?: App; onclose: () => void };

  let { store, app, onclose }: Props = $props();

  const profiles = $derived(store.snapshot?.config.profiles ?? []);
  // A one-time copy of the app being edited; the modal is remounted per app.
  // svelte-ignore state_referenced_locally
  let form = $state(
    formFromApp(app, {
      profileId: DEFAULT_PROFILE_ID,
      timeoutMinutes: store.snapshot?.config.settings.defaultHibernationMinutes ?? 10,
    }),
  );
  let errors = $state<AppFormErrors>({});
  let failure = $state<string | null>(null);
  let iconFile = $state<File | null>(null);
  let saving = $state(false);

  async function save() {
    const result = validateAppForm(form, app?.icon ?? null);
    errors = result.errors;
    if (!result.input) return;
    saving = true;
    failure = null;
    try {
      let id: string;
      if (app) {
        await api.updateApp(app.id, result.input);
        id = app.id;
      } else {
        id = await api.addApp(result.input);
      }
      if (iconFile) {
        const bytes = Array.from(new Uint8Array(await iconFile.arrayBuffer()));
        await api.setAppIcon(id, bytes);
      }
      onclose();
    } catch (error) {
      failure = String(error);
    } finally {
      saving = false;
    }
  }
</script>

<Modal title={app ? `Edit ${app.name}` : "Add app"} {onclose}>
  <form
    id="app-form"
    onsubmit={(e) => {
      e.preventDefault();
      save();
    }}
  >
    {#if failure}
      <p class="failure" role="alert">{failure}</p>
    {/if}

    <!-- Labels contain only their text (help and errors sit beside them) so accessible names stay exact. -->
    <div class="field">
      <label for="app-name">Name</label>
      <input id="app-name" bind:value={form.name} autocomplete="off" />
      {#if errors.name}<small class="error">{errors.name}</small>{/if}
    </div>

    <div class="field">
      <label for="app-url">URL</label>
      <input id="app-url" bind:value={form.url} placeholder="https://mail.google.com" autocomplete="off" />
      {#if errors.url}<small class="error">{errors.url}</small>{/if}
    </div>

    <div class="field">
      <label for="app-icon">Icon</label>
      <input
        id="app-icon"
        type="file"
        accept="image/png,image/x-icon,image/jpeg,image/gif,image/webp"
        onchange={(e) => (iconFile = (e.currentTarget as HTMLInputElement).files?.[0] ?? null)}
      />
      <small>Leave empty to use the site's favicon.</small>
    </div>

    <div class="field">
      <label for="app-profile">Profile</label>
      <select id="app-profile" bind:value={form.profileId}>
        {#each profiles as profile (profile.id)}
          <option value={profile.id}>{profile.name}</option>
        {/each}
      </select>
      <small>Apps on the same profile share logins.</small>
    </div>

    <fieldset>
      <legend>Hibernation</legend>
      <label class="check"><input type="checkbox" bind:checked={form.hibernationEnabled} /> Hibernate when unused</label>
      <div class="field">
        <label for="app-timeout">After (minutes)</label>
        <input
          id="app-timeout"
          type="number"
          min="1"
          step="1"
          bind:value={form.timeoutMinutes}
          disabled={!form.hibernationEnabled}
        />
        {#if errors.timeoutMinutes}<small class="error">{errors.timeoutMinutes}</small>{/if}
      </div>
      <label class="check"><input type="checkbox" bind:checked={form.startHibernated} /> Start hibernated</label>
    </fieldset>

    <label class="check"><input type="checkbox" bind:checked={form.notifications} /> Show notifications</label>
    <label class="check"><input type="checkbox" bind:checked={form.badges} /> Show unread badge</label>

    <details>
      <summary>Advanced</summary>
      <div class="field">
        <label for="app-user-agent">User agent</label>
        <input id="app-user-agent" bind:value={form.userAgent} placeholder="Engine default" autocomplete="off" />
      </div>
      <div class="field">
        <label for="app-domains">Extra in-app domains</label>
        <textarea id="app-domains" bind:value={form.allowedDomains} rows="3" placeholder="login.microsoftonline.com"></textarea>
        <small>One per line. Links to other sites open in your browser.</small>
      </div>
    </details>
  </form>

  {#snippet footer()}
    <button type="button" class="secondary" onclick={onclose}>Cancel</button>
    <button type="submit" form="app-form" class="primary" disabled={saving}>{app ? "Save" : "Add app"}</button>
  {/snippet}
</Modal>

<style>
  form {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 13px;
  }
  label.check {
    font-size: 13px;
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: 8px;
  }
  fieldset {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin: 0;
    padding: 10px 12px;
    border: 1px solid #303038;
    border-radius: 10px;
  }
  legend {
    padding: 0 4px;
    font-size: 12px;
    color: #a1a1aa;
  }
  input:not([type="checkbox"]),
  select,
  textarea {
    padding: 7px 9px;
    border: 1px solid #3a3a44;
    border-radius: 8px;
    background: #1b1b1f;
    color: #e4e4e7;
    font: inherit;
  }
  small {
    color: #a1a1aa;
  }
  .error {
    color: #f87171;
  }
  .failure {
    margin: 0;
    padding: 8px 10px;
    border-radius: 8px;
    background: rgb(248 113 113 / 0.12);
    color: #f87171;
  }
  details summary {
    cursor: pointer;
    color: #a1a1aa;
    font-size: 13px;
    margin-bottom: 8px;
  }
  details[open] {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .primary,
  .secondary {
    padding: 7px 14px;
    border: 0;
    border-radius: 8px;
    font: inherit;
    cursor: pointer;
  }
  .primary {
    background: #7c9cff;
    color: #1b1b1f;
    font-weight: 600;
  }
  .secondary {
    background: #303038;
    color: #e4e4e7;
  }
</style>
