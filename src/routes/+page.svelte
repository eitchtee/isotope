<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/api";
  import AppModal from "$lib/components/AppModal.svelte";
  import ConfirmModal from "$lib/components/ConfirmModal.svelte";
  import ErrorPanel from "$lib/components/ErrorPanel.svelte";
  import FolderPanel from "$lib/components/FolderPanel.svelte";
  import PromptModal from "$lib/components/PromptModal.svelte";
  import SettingsModal from "$lib/components/SettingsModal.svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import Toasts from "$lib/components/Toasts.svelte";
  import { appMenuEntries, folderMenuEntries, showContextMenu } from "$lib/menu";
  import { shell } from "$lib/shell.svelte";
  import type { App, Folder } from "$lib/types";

  type Dialog =
    | { kind: "add" }
    | { kind: "edit"; app: App }
    | { kind: "settings" }
    | { kind: "remove-app"; app: App }
    | { kind: "rename-folder"; folder: Folder }
    | { kind: "delete-empty-folder"; folder: Folder }
    | null;

  let dialog = $state<Dialog>(null);
  const close = () => (dialog = null);

  onMount(() => {
    let stop: (() => void) | undefined;
    shell
      .start()
      .then((unsubscribe) => (stop = unsubscribe))
      .catch(shell.reportError);
    return () => stop?.();
  });

  // Spec §3 invariant 7: offer to delete a folder whose last app was dragged out.
  let folderSizes = new Map<string, number>();
  $effect(() => {
    const folders = shell.snapshot?.config.folders ?? [];
    for (const folder of folders) {
      if ((folderSizes.get(folder.id) ?? 0) > 0 && folder.appIds.length === 0 && dialog === null) {
        dialog = { kind: "delete-empty-folder", folder };
      }
    }
    folderSizes = new Map(folders.map((f) => [f.id, f.appIds.length]));
  });

  function openAppMenu(app: App) {
    const entries = appMenuEntries(app, shell.state(app.id), {
      reload: () => api.reloadApp(app.id).catch(shell.reportError),
      hibernate: () => api.hibernateApp(app.id).catch(shell.reportError),
      wake: () => api.wakeApp(app.id).catch(shell.reportError),
      edit: () => (dialog = { kind: "edit", app }),
      newFolder: () =>
        api
          .addFolder("New folder", [app.id])
          .then((id) => (dialog = { kind: "rename-folder", folder: { id, name: "New folder", icon: null, appIds: [app.id] } }))
          .catch(shell.reportError),
      remove: () => (dialog = { kind: "remove-app", app }),
    });
    showContextMenu(entries).catch(shell.reportError);
  }

  function openFolderMenu(folder: Folder) {
    const entries = folderMenuEntries({
      rename: () => (dialog = { kind: "rename-folder", folder }),
      remove: () => api.removeFolder(folder.id).catch(shell.reportError),
    });
    showContextMenu(entries).catch(shell.reportError);
  }

  const contentLeft = $derived(shell.snapshot?.folderPanel ? 264 : 64);
</script>

<Sidebar
  store={shell}
  onaddapp={() => (dialog = { kind: "add" })}
  onsettings={() => (dialog = { kind: "settings" })}
  onappmenu={openAppMenu}
  onfoldermenu={openFolderMenu}
/>
<FolderPanel store={shell} onappmenu={openAppMenu} />

<main class="content" style:left="{contentLeft}px">
  <ErrorPanel store={shell} onedit={(app) => (dialog = { kind: "edit", app })} onaddapp={() => (dialog = { kind: "add" })} />
  <Toasts store={shell} />
</main>

{#if dialog?.kind === "add"}
  <AppModal store={shell} onclose={close} />
{:else if dialog?.kind === "edit"}
  {#key dialog.app.id}
    <AppModal store={shell} app={dialog.app} onclose={close} />
  {/key}
{:else if dialog?.kind === "settings"}
  <SettingsModal store={shell} onclose={close} />
{:else if dialog?.kind === "remove-app"}
  {@const app = dialog.app}
  <ConfirmModal
    title={`Remove ${app.name}?`}
    message="Its settings and icon will be deleted. Browsing data stays with its profile."
    confirmText="Remove"
    onconfirm={() => api.removeApp(app.id).catch(shell.reportError)}
    onclose={close}
  />
{:else if dialog?.kind === "rename-folder"}
  {@const folder = dialog.folder}
  <PromptModal
    title="Rename folder"
    label="Folder name"
    initial={folder.name}
    confirmText="Rename"
    onconfirm={(name) => api.renameFolder(folder.id, name).catch(shell.reportError)}
    onclose={close}
  />
{:else if dialog?.kind === "delete-empty-folder"}
  {@const folder = dialog.folder}
  <ConfirmModal
    title={`Delete empty folder "${folder.name}"?`}
    message="The folder has no apps left."
    confirmText="Delete"
    onconfirm={() => api.removeFolder(folder.id).catch(shell.reportError)}
    onclose={close}
  />
{/if}

<style>
  :global(html, body) {
    margin: 0;
    height: 100%;
    overflow: hidden;
    background: #1b1b1f;
    color: #e4e4e7;
    font-family: system-ui, -apple-system, "Segoe UI", sans-serif;
    -webkit-font-smoothing: antialiased;
  }
  .content {
    position: fixed;
    top: 0;
    right: 0;
    bottom: 0;
  }
</style>
