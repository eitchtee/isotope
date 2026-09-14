import { afterEach, expect, test, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/svelte";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import Sidebar from "./Sidebar.svelte";
import { ShellStore } from "$lib/shell.svelte";
import { snapshot } from "$lib/test/fixtures";

afterEach(() => clearMocks());

const flush = () => new Promise((resolve) => setTimeout(resolve, 0));

function setup() {
  const calls: [string, unknown][] = [];
  mockIPC((cmd, args) => {
    calls.push([cmd, args]);
    if (cmd === "activate_app" && (args as { id: string }).id === "b") throw "app not found: b";
    return null;
  });
  const store = new ShellStore();
  store.snapshot = snapshot();
  const props = { store, onaddapp: vi.fn(), onsettings: vi.fn(), onappmenu: vi.fn(), onfoldermenu: vi.fn() };
  render(Sidebar, props);
  return { calls, store, props };
}

test("renders items in sidebar order with badges and active state", () => {
  setup();
  const buttons = screen.getAllByRole("button").map((b) => b.getAttribute("aria-label"));
  expect(buttons).toEqual(["A", "B", "Folder Work", "Add app", "Settings"]);
  expect(screen.getByRole("button", { name: "A" })).toHaveAttribute("aria-current", "page");
  expect(screen.getByRole("button", { name: "B" }).querySelector("[data-testid=badge]")).toHaveTextContent("3");
});

test("clicks call activate, toggle panel and the action callbacks", async () => {
  const { calls, props } = setup();
  await fireEvent.click(screen.getByRole("button", { name: "A" }));
  await fireEvent.click(screen.getByRole("button", { name: "Folder Work" }));
  await fireEvent.click(screen.getByRole("button", { name: "Add app" }));
  await fireEvent.click(screen.getByRole("button", { name: "Settings" }));
  await fireEvent.contextMenu(screen.getByRole("button", { name: "A" }));
  await fireEvent.contextMenu(screen.getByRole("button", { name: "Folder Work" }));

  expect(calls).toEqual([
    ["activate_app", { id: "a" }],
    ["toggle_folder_panel", { folderId: "f" }],
  ]);
  expect(props.onaddapp).toHaveBeenCalledOnce();
  expect(props.onsettings).toHaveBeenCalledOnce();
  expect(props.onappmenu).toHaveBeenCalledWith(expect.objectContaining({ id: "a" }));
  expect(props.onfoldermenu).toHaveBeenCalledWith(expect.objectContaining({ id: "f" }));
});

test("command errors become toasts", async () => {
  const { store } = setup();
  await fireEvent.click(screen.getByRole("button", { name: "B" }));
  await flush();
  expect(store.toasts.map((t) => t.message)).toEqual(["app not found: b"]);
});

test("dropping an app on another item's top half moves it there", async () => {
  const { calls } = setup();
  const items = screen.getAllByRole("listitem");

  await fireEvent.dragStart(items[0]);
  await fireEvent.dragOver(items[2]);
  await fireEvent.drop(items[2]);

  expect(calls).toEqual([["move_item", { item: { type: "app", id: "a" }, target: { type: "sidebar" }, index: 1 }]]);
});
