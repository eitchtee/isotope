import { afterEach, expect, test, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/svelte";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import ConfirmModal from "./ConfirmModal.svelte";

afterEach(() => clearMocks());

test("confirm runs the action and closes; cancel only closes", async () => {
  mockIPC(() => null);
  const onconfirm = vi.fn();
  const onclose = vi.fn();
  render(ConfirmModal, { title: "Remove Gmail?", message: "Its settings will be deleted.", confirmText: "Remove", onconfirm, onclose });

  expect(screen.getByText("Its settings will be deleted.")).toBeInTheDocument();
  await fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
  expect(onconfirm).not.toHaveBeenCalled();
  expect(onclose).toHaveBeenCalledOnce();

  await fireEvent.click(screen.getByRole("button", { name: "Remove" }));
  expect(onconfirm).toHaveBeenCalledOnce();
  expect(onclose).toHaveBeenCalledTimes(2);
});
