import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Container, NewApp, SidebarItem, Snapshot, Toast } from "./types";

export const api = {
  getState: () => invoke<Snapshot>("get_state"),
  takePendingToasts: () => invoke<Toast[]>("take_pending_toasts"),
  activateApp: (id: string) => invoke<void>("activate_app", { id }),
  toggleFolderPanel: (folderId: string) => invoke<void>("toggle_folder_panel", { folderId }),
  addApp: (input: NewApp) => invoke<string>("add_app", { input }),
  updateApp: (id: string, input: NewApp) => invoke<void>("update_app", { id, input }),
  removeApp: (id: string) => invoke<void>("remove_app", { id }),
  reloadApp: (id: string) => invoke<void>("reload_app", { id }),
  hibernateApp: (id: string) => invoke<void>("hibernate_app", { id }),
  wakeApp: (id: string) => invoke<void>("wake_app", { id }),
  addFolder: (name: string, appIds: string[]) => invoke<string>("add_folder", { name, appIds }),
  renameFolder: (id: string, name: string) => invoke<void>("rename_folder", { id, name }),
  removeFolder: (id: string) => invoke<void>("remove_folder", { id }),
  moveItem: (item: SidebarItem, target: Container, index: number) =>
    invoke<void>("move_item", { item, target, index }),
  addProfile: (name: string) => invoke<string>("add_profile", { name }),
  renameProfile: (id: string, name: string) => invoke<void>("rename_profile", { id, name }),
  removeProfile: (id: string) => invoke<void>("remove_profile", { id }),
  setDefaultHibernationMinutes: (minutes: number) =>
    invoke<void>("set_default_hibernation_minutes", { minutes }),
  setOverlayOpen: (open: boolean) => invoke<void>("set_overlay_open", { open }),
  setToastVisible: (visible: boolean) => invoke<void>("set_toast_visible", { visible }),
  appIcon: (id: string) => invoke<string | null>("app_icon", { id }),
  setAppIcon: (id: string, bytes: number[]) => invoke<void>("set_app_icon", { id, bytes }),
};

export function onStateChanged(callback: (snapshot: Snapshot) => void): Promise<UnlistenFn> {
  return listen<Snapshot>("state-changed", (event) => callback(event.payload));
}

export function onToast(callback: (toast: Toast) => void): Promise<UnlistenFn> {
  return listen<Toast>("toast", (event) => callback(event.payload));
}
