import { afterEach, expect, test } from "vitest";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { emit } from "@tauri-apps/api/event";
import { ShellStore } from "./shell.svelte";
import { snapshot } from "./test/fixtures";

afterEach(() => clearMocks());

test("start loads state and pending toasts, then follows events", async () => {
  const initial = snapshot();
  mockIPC(
    (cmd) => {
      if (cmd === "get_state") return initial;
      if (cmd === "take_pending_toasts") return [{ level: "warning", message: "reset" }];
    },
    { shouldMockEvents: true },
  );
  const store = new ShellStore();

  const stop = await store.start();
  expect(store.snapshot?.activeAppId).toBe("a");
  expect(store.toasts.map((t) => t.message)).toEqual(["reset"]);

  await emit("state-changed", snapshot({ activeAppId: "b" }));
  expect(store.snapshot?.activeAppId).toBe("b");

  await emit("toast", { level: "error", message: "boom" });
  expect(store.toasts.map((t) => t.message)).toEqual(["reset", "boom"]);

  store.dismissToast(store.toasts[0].id);
  expect(store.toasts.map((t) => t.message)).toEqual(["boom"]);
  stop();
});

test("lookups read from the snapshot with safe defaults", () => {
  const store = new ShellStore();
  expect(store.state("a")).toEqual({ kind: "hibernated" });
  expect(store.app("a")).toBeUndefined();

  store.snapshot = snapshot();
  expect(store.app("b")?.name).toBe("B");
  expect(store.folder("f")?.appIds).toEqual(["c"]);
  expect(store.state("a")).toEqual({ kind: "active" });
  expect(store.badge("b")).toBe(3);
  expect(store.badge("a")).toBeUndefined();
});
