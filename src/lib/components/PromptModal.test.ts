import { afterEach, expect, test, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/svelte";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import PromptModal from "./PromptModal.svelte";

afterEach(() => clearMocks());

test("confirms the trimmed value and ignores blanks", async () => {
  mockIPC(() => null);
  const onconfirm = vi.fn();
  const onclose = vi.fn();
  render(PromptModal, { title: "Rename folder", label: "Folder name", initial: "Work", confirmText: "Rename", onconfirm, onclose });

  const input = screen.getByLabelText("Folder name");
  expect(input).toHaveValue("Work");

  await fireEvent.input(input, { target: { value: "   " } });
  await fireEvent.click(screen.getByRole("button", { name: "Rename" }));
  expect(onconfirm).not.toHaveBeenCalled();

  await fireEvent.input(input, { target: { value: " Office " } });
  await fireEvent.click(screen.getByRole("button", { name: "Rename" }));
  expect(onconfirm).toHaveBeenCalledWith("Office");
  expect(onclose).toHaveBeenCalledOnce();
});
