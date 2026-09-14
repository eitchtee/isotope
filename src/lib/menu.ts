import { Menu, PredefinedMenuItem } from "@tauri-apps/api/menu";
import type { App, AppState } from "./types";

export type MenuEntry = { text: string; action: () => void; enabled?: boolean } | "separator";

/** Native context menus draw above the app webviews, so they never need the overlay. */
export async function showContextMenu(entries: MenuEntry[]): Promise<void> {
  const items = await Promise.all(
    entries.map((entry) =>
      entry === "separator"
        ? PredefinedMenuItem.new({ item: "Separator" })
        : Promise.resolve({ text: entry.text, enabled: entry.enabled ?? true, action: () => entry.action() }),
    ),
  );
  const menu = await Menu.new({ items });
  await menu.popup();
}

export function appMenuEntries(
  _app: App,
  status: AppState,
  actions: {
    reload: () => void;
    hibernate: () => void;
    wake: () => void;
    edit: () => void;
    newFolder: () => void;
    remove: () => void;
  },
): MenuEntry[] {
  const live = status.kind === "running" || status.kind === "active";
  return [
    { text: "Reload", action: actions.reload, enabled: live },
    { text: "Hibernate now", action: actions.hibernate, enabled: live },
    { text: "Wake without switching", action: actions.wake, enabled: !live },
    "separator",
    { text: "Edit…", action: actions.edit },
    { text: "New folder with this app", action: actions.newFolder },
    "separator",
    { text: "Remove…", action: actions.remove },
  ];
}

export function folderMenuEntries(actions: { rename: () => void; remove: () => void }): MenuEntry[] {
  return [{ text: "Rename…", action: actions.rename }, "separator", { text: "Delete folder", action: actions.remove }];
}
