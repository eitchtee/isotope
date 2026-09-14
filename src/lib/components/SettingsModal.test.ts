import { afterEach, expect, test, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/svelte";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import SettingsModal from "./SettingsModal.svelte";
import { ShellStore } from "$lib/shell.svelte";
import { snapshot } from "$lib/test/fixtures";

afterEach(() => clearMocks());

const flush = () => new Promise((resolve) => setTimeout(resolve, 0));

function setup(options: { failWith?: string; readOnly?: boolean } = {}) {
  const calls: [string, unknown][] = [];
  mockIPC((cmd, args) => {
    if (cmd !== "set_overlay_open") calls.push([cmd, args]);
    if (options.failWith && cmd === "add_profile") throw options.failWith;
    if (cmd === "add_profile") return "p-new";
    return null;
  });
  const store = new ShellStore();
  const s = snapshot({ readOnly: options.readOnly ?? false });
  s.config.profiles.push({ id: "work", name: "Work" }, { id: "old", name: "Old" });
  s.config.apps[1].profileId = "work";
  store.snapshot = s;
  render(SettingsModal, { store, onclose: vi.fn() });
  return { calls };
}

test("profiles show usage and deletion rules", () => {
  setup();
  expect(screen.getByRole("button", { name: "Delete profile Default" })).toBeDisabled();
  expect(screen.getByRole("button", { name: "Delete profile Work" })).toBeDisabled();
  expect(screen.getByRole("button", { name: "Delete profile Old" })).toBeEnabled();
  expect(screen.getByText("2 apps")).toBeInTheDocument();
  expect(screen.getByText("1 app")).toBeInTheDocument();
  expect(screen.getByText("0 apps")).toBeInTheDocument();
});

test("add, rename and delete call Rust", async () => {
  const { calls } = setup();

  await fireEvent.input(screen.getByLabelText("New profile name"), { target: { value: " Personal " } });
  await fireEvent.click(screen.getByRole("button", { name: "Add profile" }));
  await flush();
  expect(screen.getByLabelText("New profile name")).toHaveValue("");

  const workName = screen.getByLabelText("Profile name Work");
  await fireEvent.input(workName, { target: { value: "Office" } });
  await fireEvent.change(workName);
  await fireEvent.click(screen.getByRole("button", { name: "Delete profile Old" }));
  await flush();

  expect(calls).toEqual([
    ["add_profile", { name: "Personal" }],
    ["rename_profile", { id: "work", name: "Office" }],
    ["remove_profile", { id: "old" }],
  ]);
});

test("default hibernation minutes are validated before saving", async () => {
  const { calls } = setup();
  const input = screen.getByLabelText("Default hibernation (minutes)");

  await fireEvent.change(input, { target: { value: "0" } });
  await fireEvent.change(input, { target: { value: "20" } });
  await flush();

  expect(calls).toEqual([["set_default_hibernation_minutes", { minutes: 20 }]]);
});

test("failures and read-only mode are shown", async () => {
  setup({ failWith: "disk full", readOnly: true });
  expect(screen.getByRole("status")).toHaveTextContent("read-only");

  await fireEvent.input(screen.getByLabelText("New profile name"), { target: { value: "X" } });
  await fireEvent.click(screen.getByRole("button", { name: "Add profile" }));
  await flush();

  expect(screen.getByRole("alert")).toHaveTextContent("disk full");
});
