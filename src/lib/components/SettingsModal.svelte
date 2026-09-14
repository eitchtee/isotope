<script lang="ts">
  import { api } from "$lib/api";
  import type { ShellStore } from "$lib/shell.svelte";
  import { DEFAULT_PROFILE_ID } from "$lib/types";
  import Modal from "./Modal.svelte";

  type Props = { store: ShellStore; onclose: () => void };

  let { store, onclose }: Props = $props();

  const config = $derived(store.snapshot?.config);
  let newProfile = $state("");
  let names = $state<Record<string, string>>({});
  let failure = $state<string | null>(null);

  function usage(profileId: string): number {
    return config?.apps.filter((a) => a.profileId === profileId).length ?? 0;
  }

  async function attempt(action: () => Promise<unknown>) {
    failure = null;
    try {
      await action();
    } catch (error) {
      failure = String(error);
    }
  }

  function addProfile() {
    const name = newProfile.trim();
    if (!name) return;
    attempt(async () => {
      await api.addProfile(name);
      newProfile = "";
    });
  }

  function rename(id: string) {
    const name = names[id]?.trim();
    if (name) attempt(() => api.renameProfile(id, name));
  }

  function setMinutes(value: number) {
    if (Number.isInteger(value) && value >= 1) attempt(() => api.setDefaultHibernationMinutes(value));
  }

  function deleteTitle(profileId: string, count: number): string {
    if (profileId === DEFAULT_PROFILE_ID) return "The Default profile can't be deleted";
    if (count > 0) return "Move its apps to another profile first";
    return "Delete this profile and its browsing data";
  }
</script>

<Modal title="Settings" {onclose}>
  {#if store.snapshot?.readOnly}
    <p class="notice" role="status">
      Settings are read-only because they were created by a newer Isotope version. Changes won't be saved.
    </p>
  {/if}
  {#if failure}
    <p class="failure" role="alert">{failure}</p>
  {/if}

  {#if config}
    <section>
      <h3>General</h3>
      <div class="field">
        <label for="default-hibernation">Default hibernation (minutes)</label>
        <input
          id="default-hibernation"
          type="number"
          min="1"
          step="1"
          value={config.settings.defaultHibernationMinutes}
          onchange={(e) => setMinutes(Number((e.currentTarget as HTMLInputElement).value))}
        />
      </div>
    </section>

    <section>
      <h3>Profiles</h3>
      <p class="hint">Apps on the same profile share cookies and logins.</p>
      <ul>
        {#each config.profiles as profile (profile.id)}
          {@const count = usage(profile.id)}
          <li>
            <input
              aria-label={`Profile name ${profile.name}`}
              value={names[profile.id] ?? profile.name}
              oninput={(e) => (names[profile.id] = (e.currentTarget as HTMLInputElement).value)}
              onchange={() => rename(profile.id)}
            />
            <span class="count">{count} {count === 1 ? "app" : "apps"}</span>
            <button
              class="danger"
              aria-label={`Delete profile ${profile.name}`}
              title={deleteTitle(profile.id, count)}
              disabled={profile.id === DEFAULT_PROFILE_ID || count > 0}
              onclick={() => attempt(() => api.removeProfile(profile.id))}
            >
              Delete
            </button>
          </li>
        {/each}
      </ul>
      <form
        onsubmit={(e) => {
          e.preventDefault();
          addProfile();
        }}
      >
        <input aria-label="New profile name" placeholder="New profile" bind:value={newProfile} />
        <button type="submit">Add profile</button>
      </form>
    </section>
  {/if}
</Modal>

<style>
  section + section {
    margin-top: 18px;
  }
  h3 {
    margin: 0 0 8px;
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: #a1a1aa;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 13px;
  }
  .hint {
    margin: 0 0 8px;
    font-size: 12px;
    color: #a1a1aa;
  }
  ul {
    list-style: none;
    margin: 0 0 10px;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  li,
  form {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  input {
    flex: 1;
    padding: 6px 9px;
    border: 1px solid #3a3a44;
    border-radius: 8px;
    background: #1b1b1f;
    color: #e4e4e7;
    font: inherit;
    font-size: 13px;
  }
  .count {
    width: 52px;
    font-size: 12px;
    color: #a1a1aa;
    font-variant-numeric: tabular-nums;
  }
  button {
    padding: 6px 12px;
    border: 0;
    border-radius: 8px;
    background: #303038;
    color: #e4e4e7;
    font: inherit;
    font-size: 13px;
    cursor: pointer;
  }
  button:disabled {
    opacity: 0.4;
    cursor: default;
  }
  .danger:not(:disabled):hover {
    background: rgb(248 113 113 / 0.2);
    color: #f87171;
  }
  .notice,
  .failure {
    margin: 0 0 12px;
    padding: 8px 10px;
    border-radius: 8px;
    font-size: 13px;
  }
  .notice {
    background: rgb(250 204 21 / 0.12);
    color: #facc15;
  }
  .failure {
    background: rgb(248 113 113 / 0.12);
    color: #f87171;
  }
</style>
