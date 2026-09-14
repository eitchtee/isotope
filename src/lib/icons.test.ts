import { afterEach, expect, test } from "vitest";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { IconCache } from "./icons.svelte";

afterEach(() => clearMocks());

const flush = () => new Promise((resolve) => setTimeout(resolve, 0));

test("loads each icon version once and caches the data URL", async () => {
  let calls = 0;
  mockIPC((cmd, args) => {
    if (cmd === "app_icon") {
      calls++;
      return `data:image/png;base64,${(args as { id: string }).id}`;
    }
  });
  const cache = new IconCache();

  expect(cache.url("a", "icons/a.png#1")).toBeNull();
  expect(cache.url("a", "icons/a.png#1")).toBeNull();
  await flush();

  expect(cache.url("a", "icons/a.png#1")).toBe("data:image/png;base64,a");
  expect(calls).toBe(1);

  cache.url("a", "icons/a.png#2");
  await flush();
  expect(calls).toBe(2);
});

test("apps without an icon never call Rust", () => {
  let calls = 0;
  mockIPC(() => {
    calls++;
  });
  expect(new IconCache().url("a", null)).toBeNull();
  expect(calls).toBe(0);
});
