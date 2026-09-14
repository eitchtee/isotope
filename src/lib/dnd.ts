import type { Container, SidebarItem, Snapshot } from "./types";

export type DragItem = { item: SidebarItem; from: Container; fromIndex: number };

export type DropTarget =
  | { kind: "sidebar-slot"; index: number }
  | { kind: "into-folder"; folderId: string }
  | { kind: "folder-slot"; folderId: string; index: number };

export type Move = { item: SidebarItem; target: Container; index: number };

/**
 * Converts a drop into `move_item` arguments. Gap indexes are counted before the
 * dragged item is removed; Rust expects the index after removal, so moving an
 * item down within the same list shifts the index by one.
 */
export function planMove(drag: DragItem, drop: DropTarget, snapshot: Snapshot): Move | null {
  switch (drop.kind) {
    case "sidebar-slot": {
      const same = drag.from.type === "sidebar";
      const index = same && drag.fromIndex < drop.index ? drop.index - 1 : drop.index;
      if (same && index === drag.fromIndex) return null;
      return { item: drag.item, target: { type: "sidebar" }, index };
    }
    case "into-folder": {
      if (drag.item.type === "folder") return null;
      if (drag.from.type === "folder" && drag.from.id === drop.folderId) return null;
      const folder = snapshot.config.folders.find((f) => f.id === drop.folderId);
      if (!folder) return null;
      return { item: drag.item, target: { type: "folder", id: folder.id }, index: folder.appIds.length };
    }
    case "folder-slot": {
      if (drag.item.type === "folder") return null;
      const same = drag.from.type === "folder" && drag.from.id === drop.folderId;
      const index = same && drag.fromIndex < drop.index ? drop.index - 1 : drop.index;
      if (same && index === drag.fromIndex) return null;
      return { item: drag.item, target: { type: "folder", id: drop.folderId }, index };
    }
  }
}

/** Middle half of a folder icon drops into the folder; otherwise the nearer gap. */
export function sidebarDropTarget(
  index: number,
  item: { type: "app" | "folder"; id: string },
  offsetY: number,
  height: number,
): DropTarget {
  const ratio = height > 0 ? offsetY / height : 0;
  if (item.type === "folder" && ratio > 0.25 && ratio < 0.75) {
    return { kind: "into-folder", folderId: item.id };
  }
  return { kind: "sidebar-slot", index: ratio < 0.5 ? index : index + 1 };
}

export function folderPanelDropTarget(folderId: string, index: number, offsetY: number, height: number): DropTarget {
  const ratio = height > 0 ? offsetY / height : 0;
  return { kind: "folder-slot", folderId, index: ratio < 0.5 ? index : index + 1 };
}
