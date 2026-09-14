import { expect, test } from "vitest";
import { folderPanelDropTarget, planMove, sidebarDropTarget, type DragItem } from "./dnd";
import { snapshot } from "./test/fixtures";

const s = snapshot(); // sidebar [a, b, f]; folder f = [c]

const fromSidebar = (id: string, fromIndex: number, type: "app" | "folder" = "app"): DragItem => ({
  item: { type, id },
  from: { type: "sidebar" },
  fromIndex,
});

test("reordering within the sidebar adjusts for removal", () => {
  // Drag a (index 0) to the gap before f (index 2) -> lands at 1 after removal.
  expect(planMove(fromSidebar("a", 0), { kind: "sidebar-slot", index: 2 }, s)).toEqual({
    item: { type: "app", id: "a" },
    target: { type: "sidebar" },
    index: 1,
  });
  // Drag f (index 2) to the top.
  expect(planMove(fromSidebar("f", 2, "folder"), { kind: "sidebar-slot", index: 0 }, s)).toEqual({
    item: { type: "folder", id: "f" },
    target: { type: "sidebar" },
    index: 0,
  });
});

test("dropping in the gaps around the item itself is a no-op", () => {
  expect(planMove(fromSidebar("b", 1), { kind: "sidebar-slot", index: 1 }, s)).toBeNull();
  expect(planMove(fromSidebar("b", 1), { kind: "sidebar-slot", index: 2 }, s)).toBeNull();
});

test("dropping an app onto a folder appends it", () => {
  expect(planMove(fromSidebar("a", 0), { kind: "into-folder", folderId: "f" }, s)).toEqual({
    item: { type: "app", id: "a" },
    target: { type: "folder", id: "f" },
    index: 1,
  });
});

test("folders cannot go into folders, and an app can't be dropped on its own folder", () => {
  expect(planMove(fromSidebar("f", 2, "folder"), { kind: "into-folder", folderId: "f" }, s)).toBeNull();
  expect(planMove(fromSidebar("f", 2, "folder"), { kind: "folder-slot", folderId: "f", index: 0 }, s)).toBeNull();
  const fromFolder: DragItem = { item: { type: "app", id: "c" }, from: { type: "folder", id: "f" }, fromIndex: 0 };
  expect(planMove(fromFolder, { kind: "into-folder", folderId: "f" }, s)).toBeNull();
  expect(planMove(fromSidebar("a", 0), { kind: "into-folder", folderId: "missing" }, s)).toBeNull();
});

test("dragging out of a folder panel onto the sidebar", () => {
  const fromFolder: DragItem = { item: { type: "app", id: "c" }, from: { type: "folder", id: "f" }, fromIndex: 0 };
  expect(planMove(fromFolder, { kind: "sidebar-slot", index: 1 }, s)).toEqual({
    item: { type: "app", id: "c" },
    target: { type: "sidebar" },
    index: 1,
  });
});

test("reordering inside a folder panel", () => {
  const big = snapshot();
  big.config.folders[0].appIds = ["c", "a", "b"];
  const drag: DragItem = { item: { type: "app", id: "c" }, from: { type: "folder", id: "f" }, fromIndex: 0 };

  expect(planMove(drag, { kind: "folder-slot", folderId: "f", index: 3 }, big)).toEqual({
    item: { type: "app", id: "c" },
    target: { type: "folder", id: "f" },
    index: 2,
  });
  expect(planMove(drag, { kind: "folder-slot", folderId: "f", index: 1 }, big)).toBeNull();
});

test("pointer position picks the gap or the folder", () => {
  const app = { type: "app" as const, id: "a" };
  const folder = { type: "folder" as const, id: "f" };

  expect(sidebarDropTarget(3, app, 10, 44)).toEqual({ kind: "sidebar-slot", index: 3 });
  expect(sidebarDropTarget(3, app, 30, 44)).toEqual({ kind: "sidebar-slot", index: 4 });
  expect(sidebarDropTarget(3, folder, 22, 44)).toEqual({ kind: "into-folder", folderId: "f" });
  expect(sidebarDropTarget(3, folder, 5, 44)).toEqual({ kind: "sidebar-slot", index: 3 });
  expect(sidebarDropTarget(3, folder, 40, 44)).toEqual({ kind: "sidebar-slot", index: 4 });
  expect(sidebarDropTarget(0, app, 0, 0)).toEqual({ kind: "sidebar-slot", index: 0 });

  expect(folderPanelDropTarget("f", 1, 5, 36)).toEqual({ kind: "folder-slot", folderId: "f", index: 1 });
  expect(folderPanelDropTarget("f", 1, 30, 36)).toEqual({ kind: "folder-slot", folderId: "f", index: 2 });
});
