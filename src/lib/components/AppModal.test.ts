import { afterEach, expect, test, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/svelte";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import AppModal from "./AppModal.svelte";
import { ShellStore } from "$lib/shell.svelte";
import { app, snapshot } from "$lib/test/fixtures";

afterEach(() => clearMocks());

const flush = () => new Promise((resolve) => setTimeout(resolve, 0));

function setup(options: { edit?: boolean; failWith?: string } = {}) {
  const calls: [string, any][] = [];
  mockIPC((cmd, args) => {
    calls.push([cmd, args]);
    if (options.failWith && (cmd === "add_app" || cmd === "update_app")) throw options.failWith;
    if (cmd === "add_app") return "new-id";
    return null;
  });
  const store = new ShellStore();
  const s = snapshot();
  s.config.profiles.push({ id: "work", name: "Work" });
  s.config.apps[0] = app("a", { icon: "icons/a.png#7" });
  store.snapshot = s;
  const onclose = vi.fn();
  const view = render(AppModal, { store, app: options.edit ? s.config.apps[0] : undefined, onclose });
  return { calls, onclose, view };
}

test("hides the app webview while open", async () => {
  const { calls, view } = setup();
  await flush();
  expect(calls).toContainEqual(["set_overlay_open", { open: true }]);
  view.unmount();
  await flush();
  expect(calls.at(-1)).toEqual(["set_overlay_open", { open: false }]);
});

test("adding an app sends a NewApp and closes", async () => {
  const { calls, onclose } = setup();

  await fireEvent.input(screen.getByLabelText("Name"), { target: { value: "Gmail" } });
  await fireEvent.input(screen.getByLabelText("URL"), { target: { value: "mail.google.com" } });
  await fireEvent.change(screen.getByLabelText("Profile"), { target: { value: "work" } });
  await fireEvent.click(screen.getByRole("button", { name: "Add app" }));
  await flush();

  const add = calls.find(([cmd]) => cmd === "add_app");
  expect(add?.[1].input).toMatchObject({
    name: "Gmail",
    url: "https://mail.google.com/",
    profileId: "work",
    icon: null,
    hibernation: { enabled: true, timeoutMinutes: 10, startHibernated: false },
  });
  expect(onclose).toHaveBeenCalledOnce();
});

test("editing keeps the existing icon value", async () => {
  const { calls, onclose } = setup({ edit: true });

  expect(screen.getByLabelText("Name")).toHaveValue("A");
  await fireEvent.input(screen.getByLabelText("Name"), { target: { value: "Renamed" } });
  await fireEvent.click(screen.getByRole("button", { name: "Save" }));
  await flush();

  const update = calls.find(([cmd]) => cmd === "update_app");
  expect(update?.[1]).toMatchObject({ id: "a", input: { name: "Renamed", icon: "icons/a.png#7" } });
  expect(onclose).toHaveBeenCalledOnce();
});

test("invalid input is reported inline without calling Rust", async () => {
  const { calls, onclose } = setup();
  await fireEvent.click(screen.getByRole("button", { name: "Add app" }));
  await flush();

  expect(screen.getByText("Name is required")).toBeInTheDocument();
  expect(screen.getByText("Enter a web address (http or https)")).toBeInTheDocument();
  expect(calls.some(([cmd]) => cmd === "add_app")).toBe(false);
  expect(onclose).not.toHaveBeenCalled();
});

test("uploads a chosen icon after saving", async () => {
  const { calls } = setup();
  await fireEvent.input(screen.getByLabelText("Name"), { target: { value: "Gmail" } });
  await fireEvent.input(screen.getByLabelText("URL"), { target: { value: "https://mail.google.com" } });
  const file = new File([new Uint8Array([0x89, 0x50, 0x4e, 0x47])], "icon.png", { type: "image/png" });
  await fireEvent.change(screen.getByLabelText("Icon"), { target: { files: [file] } });
  await fireEvent.click(screen.getByRole("button", { name: "Add app" }));
  await flush();
  await flush();

  expect(calls.find(([cmd]) => cmd === "set_app_icon")?.[1]).toEqual({ id: "new-id", bytes: [0x89, 0x50, 0x4e, 0x47] });
});

test("Rust errors are shown in the form", async () => {
  setup({ edit: true, failWith: "profile not found: work" });
  await fireEvent.click(screen.getByRole("button", { name: "Save" }));
  await flush();
  expect(screen.getByRole("alert")).toHaveTextContent("profile not found: work");
});
