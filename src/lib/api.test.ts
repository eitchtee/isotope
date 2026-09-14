import { afterEach, expect, test } from "vitest";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { api } from "./api";

afterEach(() => clearMocks());

test("commands use the Rust names and camelCase argument keys", async () => {
  const calls: [string, unknown][] = [];
  mockIPC((cmd, args) => {
    calls.push([cmd, args]);
    return cmd === "add_folder" ? "f1" : null;
  });

  await api.toggleFolderPanel("f1");
  await api.moveItem({ type: "app", id: "a" }, { type: "folder", id: "f1" }, 2);
  expect(await api.addFolder("Work", ["a"])).toBe("f1");
  await api.setAppIcon("a", [1, 2, 3]);
  await api.setDefaultHibernationMinutes(15);

  expect(calls).toEqual([
    ["toggle_folder_panel", { folderId: "f1" }],
    ["move_item", { item: { type: "app", id: "a" }, target: { type: "folder", id: "f1" }, index: 2 }],
    ["add_folder", { name: "Work", appIds: ["a"] }],
    ["set_app_icon", { id: "a", bytes: [1, 2, 3] }],
    ["set_default_hibernation_minutes", { minutes: 15 }],
  ]);
});
