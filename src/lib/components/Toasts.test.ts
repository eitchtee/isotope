import { afterEach, expect, test, vi } from "vitest";
import { tick } from "svelte";
import { fireEvent, render, screen } from "@testing-library/svelte";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import Toasts from "./Toasts.svelte";
import { ShellStore } from "$lib/shell.svelte";
import { snapshot } from "$lib/test/fixtures";

afterEach(() => {
  clearMocks();
  vi.useRealTimers();
});

function setup() {
  const visibility: boolean[] = [];
  mockIPC((cmd, args) => {
    if (cmd === "set_toast_visible") visibility.push((args as { visible: boolean }).visible);
    return null;
  });
  const store = new ShellStore();
  store.snapshot = snapshot();
  return { store, visibility };
}

test("shows the oldest toast, dismisses it, and reports strip visibility", async () => {
  const { store, visibility } = setup();
  store.pushToast({ level: "error", message: "first" });
  store.pushToast({ level: "info", message: "second" });
  render(Toasts, { store });
  await tick();

  expect(screen.getByRole("status")).toHaveTextContent("first");
  await fireEvent.click(screen.getByRole("button", { name: "Dismiss" }));
  expect(screen.getByRole("status")).toHaveTextContent("second");
  await fireEvent.click(screen.getByRole("button", { name: "Dismiss" }));
  await tick();

  expect(screen.queryByRole("status")).toBeNull();
  expect(visibility).toEqual([true, false]);
});

test("toasts auto-dismiss after five seconds", async () => {
  vi.useFakeTimers();
  const { store } = setup();
  store.pushToast({ level: "warning", message: "soon gone" });
  render(Toasts, { store });
  await tick();

  vi.advanceTimersByTime(4999);
  await tick();
  expect(store.toasts).toHaveLength(1);

  vi.advanceTimersByTime(1);
  await tick();
  expect(store.toasts).toHaveLength(0);
});
