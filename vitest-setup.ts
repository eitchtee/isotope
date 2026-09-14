import "@testing-library/jest-dom/vitest";
import { randomFillSync } from "node:crypto";

// @tauri-apps/api/mocks needs crypto.getRandomValues.
if (!globalThis.crypto?.getRandomValues) {
  Object.defineProperty(globalThis, "crypto", {
    value: { getRandomValues: (buffer: Uint8Array) => randomFillSync(buffer) },
  });
}
