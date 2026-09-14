import { expect, test, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/svelte";
import FolderIcon from "./FolderIcon.svelte";
import { app } from "$lib/test/fixtures";

test("shows up to four mini icons, the badge, and toggles on click", async () => {
  const ontoggle = vi.fn();
  const apps = ["a", "b", "c", "d", "e"].map((id) => app(id));
  render(FolderIcon, {
    folder: { id: "f", name: "Work", icon: null, appIds: apps.map((a) => a.id) },
    apps,
    badge: 5,
    open: true,
    iconUrl: (a: { id: string }) => (a.id === "a" ? "data:image/png;base64,a" : null),
    ontoggle,
  });

  const button = screen.getByRole("button", { name: "Folder Work" });
  expect(button.querySelectorAll(".mini")).toHaveLength(4);
  expect(button.querySelector("img")).toHaveAttribute("src", "data:image/png;base64,a");
  expect(button).toHaveAttribute("aria-expanded", "true");
  expect(screen.getByTestId("badge")).toHaveTextContent("5");

  await fireEvent.click(button);
  expect(ontoggle).toHaveBeenCalledOnce();
});
