import { afterEach, expect, test, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/svelte";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import ErrorPanel from "./ErrorPanel.svelte";
import { ShellStore } from "$lib/shell.svelte";
import { snapshot } from "$lib/test/fixtures";

afterEach(() => clearMocks());

function setup(configure: (store: ShellStore) => void) {
  const calls: [string, unknown][] = [];
  mockIPC((cmd, args) => {
    calls.push([cmd, args]);
    return null;
  });
  const store = new ShellStore();
  store.snapshot = snapshot();
  configure(store);
  const onedit = vi.fn();
  const onaddapp = vi.fn();
  render(ErrorPanel, { store, onedit, onaddapp });
  return { calls, onedit, onaddapp };
}

test("renders nothing while the active app is healthy", () => {
  setup(() => {});
  expect(screen.queryByRole("button")).toBeNull();
});

test("offers retry and edit when the active app failed", async () => {
  const { calls, onedit } = setup((store) => {
    store.snapshot!.states.a = { kind: "error", message: "DNS lookup failed" };
  });

  expect(screen.getByText("DNS lookup failed")).toBeInTheDocument();
  await fireEvent.click(screen.getByRole("button", { name: "Retry" }));
  await fireEvent.click(screen.getByRole("button", { name: "Edit" }));

  expect(calls).toEqual([["wake_app", { id: "a" }]]);
  expect(onedit).toHaveBeenCalledWith(expect.objectContaining({ id: "a" }));
});

test("shows an empty state when there are no apps", async () => {
  const { onaddapp } = setup((store) => {
    const s = store.snapshot!;
    s.config.apps = [];
    s.config.folders = [];
    s.config.sidebar = [];
    s.activeAppId = null;
  });

  await fireEvent.click(screen.getByRole("button", { name: "Add your first app" }));
  expect(onaddapp).toHaveBeenCalledOnce();
});
