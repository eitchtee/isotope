import { expect, test, vi } from "vitest";
import { appMenuEntries, folderMenuEntries, type MenuEntry } from "./menu";
import { app } from "./test/fixtures";

const texts = (entries: MenuEntry[]) =>
  entries.map((e) => (e === "separator" ? "---" : `${e.text}${e.enabled === false ? " (disabled)" : ""}`));

const actions = () => ({
  reload: vi.fn(),
  hibernate: vi.fn(),
  wake: vi.fn(),
  toggleMute: vi.fn(),
  edit: vi.fn(),
  newFolder: vi.fn(),
  remove: vi.fn(),
});

test("live apps can reload, hibernate and mute but not wake", () => {
  expect(texts(appMenuEntries(app("a"), { kind: "running" }, false, actions()))).toEqual([
    "Reload",
    "Hibernate now",
    "Wake without switching (disabled)",
    "---",
    "Mute",
    "---",
    "Edit…",
    "New folder with this app",
    "---",
    "Remove…",
  ]);
});

test("muted apps offer Unmute", () => {
  expect(texts(appMenuEntries(app("a"), { kind: "active" }, true, actions()))).toContain("Unmute");
});

test("hibernated and failed apps can only be woken", () => {
  for (const status of [{ kind: "hibernated" as const }, { kind: "error" as const, message: "x" }]) {
    expect(texts(appMenuEntries(app("a"), status, false, actions())).slice(0, 5)).toEqual([
      "Reload (disabled)",
      "Hibernate now (disabled)",
      "Wake without switching",
      "---",
      "Mute (disabled)",
    ]);
  }
});

test("entries call their actions", () => {
  const a = actions();
  const entries = appMenuEntries(app("a"), { kind: "active" }, false, a);
  for (const entry of entries) if (entry !== "separator") entry.action();
  for (const fn of Object.values(a)) expect(fn).toHaveBeenCalledOnce();

  const f = { rename: vi.fn(), remove: vi.fn() };
  expect(texts(folderMenuEntries(f))).toEqual(["Rename…", "---", "Delete folder"]);
});
