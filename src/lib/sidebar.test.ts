import { expect, test } from "vitest";
import { badgeLabel, folderBadge, initials, resolveSidebar } from "./sidebar";
import { snapshot } from "./test/fixtures";

test("resolveSidebar expands apps and folders in order and skips unknown ids", () => {
  const s = snapshot();
  s.config.sidebar.push({ type: "app", id: "ghost" });

  const items = resolveSidebar(s);

  expect(items.map((i) => `${i.type}:${i.id}`)).toEqual(["app:a", "app:b", "folder:f"]);
  const folder = items[2];
  expect(folder.type === "folder" && folder.apps.map((a) => a.id)).toEqual(["c"]);
});

test("folderBadge sums counts and falls back to a dot", () => {
  const s = snapshot();
  const folder = { id: "f", name: "Work", icon: null, appIds: ["a", "b", "c"] };

  expect(folderBadge(s, folder)).toBe(3);

  s.badges = { a: "dot", c: 2 };
  expect(folderBadge(s, folder)).toBe(2);

  s.badges = { a: "dot" };
  expect(folderBadge(s, folder)).toBe("dot");

  s.badges = {};
  expect(folderBadge(s, folder)).toBeUndefined();
});

test("badgeLabel caps large counts and hides dot text", () => {
  expect(badgeLabel(7)).toBe("7");
  expect(badgeLabel(99)).toBe("99");
  expect(badgeLabel(100)).toBe("99+");
  expect(badgeLabel("dot")).toBe("");
});

test("initials", () => {
  expect(initials("Gmail")).toBe("G");
  expect(initials("google calendar")).toBe("GC");
  expect(initials("  Microsoft Teams Work ")).toBe("MT");
  expect(initials("")).toBe("?");
});
