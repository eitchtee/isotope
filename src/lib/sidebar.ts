import type { App, Badge, Folder, Snapshot } from "./types";

export type ResolvedItem =
  | { type: "app"; id: string; app: App }
  | { type: "folder"; id: string; folder: Folder; apps: App[] };

export function resolveSidebar(snapshot: Snapshot): ResolvedItem[] {
  const apps = new Map(snapshot.config.apps.map((a) => [a.id, a]));
  const folders = new Map(snapshot.config.folders.map((f) => [f.id, f]));
  const items: ResolvedItem[] = [];
  for (const entry of snapshot.config.sidebar) {
    if (entry.type === "app") {
      const app = apps.get(entry.id);
      if (app) items.push({ type: "app", id: app.id, app });
    } else {
      const folder = folders.get(entry.id);
      if (folder) {
        const folderApps = folder.appIds.map((id) => apps.get(id)).filter((a): a is App => a !== undefined);
        items.push({ type: "folder", id: folder.id, folder, apps: folderApps });
      }
    }
  }
  return items;
}

/** Spec §4.1: sum of counts, or a dot when nothing is counted but some app has a dot. */
export function folderBadge(snapshot: Snapshot, folder: Folder): Badge | undefined {
  let sum = 0;
  let dot = false;
  for (const id of folder.appIds) {
    const badge = snapshot.badges[id];
    if (badge === "dot") dot = true;
    else if (typeof badge === "number") sum += badge;
  }
  if (sum > 0) return sum;
  return dot ? "dot" : undefined;
}

export function badgeLabel(badge: Badge): string {
  if (badge === "dot") return "";
  return badge > 99 ? "99+" : String(badge);
}

export function initials(name: string): string {
  const words = name.trim().split(/\s+/).filter(Boolean);
  if (words.length === 0) return "?";
  return words
    .slice(0, 2)
    .map((w) => w[0].toUpperCase())
    .join("");
}
