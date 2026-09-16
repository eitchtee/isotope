import { afterEach, expect, test, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/svelte";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import FolderPanel from "./FolderPanel.svelte";
import { ShellStore } from "$lib/shell.svelte";
import { app, snapshot } from "$lib/test/fixtures";

afterEach(() => clearMocks());

function setup(folderPanel: string | null) {
  const calls: [string, unknown][] = [];
  mockIPC((cmd, args) => {
    calls.push([cmd, args]);
    return null;
  });
  const store = new ShellStore();
  const s = snapshot({ folderPanel });
  s.config.apps.push(app("d"), app("e"));
  s.config.folders[0].appIds = ["c", "d", "e"];
  store.snapshot = s;
  const onappmenu = vi.fn();
  render(FolderPanel, { store, onappmenu });
  return { calls, onappmenu };
}

test("hidden when no folder panel is open", () => {
  setup(null);
  expect(screen.queryByRole("region")).toBeNull();
});

test("shows the folder's apps as small icons with tooltips", async () => {
  const { calls, onappmenu } = setup("f");
  expect(screen.getByRole("region", { name: "Work" })).toBeInTheDocument();

  const buttons = screen.getAllByRole("button");
  expect(buttons.map((b) => b.getAttribute("aria-label"))).toEqual(["C", "D", "E"]);
  for (const button of buttons) {
    expect(button).toHaveClass("small");
    expect(button).toHaveAttribute("title", button.getAttribute("aria-label"));
  }

  await fireEvent.click(screen.getByRole("button", { name: "D" }));
  await fireEvent.contextMenu(screen.getByRole("button", { name: "E" }));

  expect(calls).toEqual([["activate_app", { id: "d" }]]);
  expect(onappmenu).toHaveBeenCalledWith(expect.objectContaining({ id: "e" }));
});

test("dragging within the panel reorders the folder and shows a drop indicator", async () => {
  const { calls } = setup("f");
  const items = screen.getAllByRole("listitem");

  await fireEvent.dragStart(items[0]);
  await fireEvent.dragOver(items[2]);
  expect(items[2]).toHaveClass("drop-before");

  await fireEvent.drop(items[2]);

  expect(items[2]).not.toHaveClass("drop-before");
  expect(calls).toEqual([["move_item", { item: { type: "app", id: "c" }, target: { type: "folder", id: "f" }, index: 1 }]]);
});
