import { expect, test, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/svelte";
import AppIcon from "./AppIcon.svelte";
import { app } from "$lib/test/fixtures";

test("shows initials, activates on click and marks the active app", async () => {
  const onactivate = vi.fn();
  render(AppIcon, { app: app("g", { name: "Google Mail" }), status: { kind: "active" }, active: true, iconUrl: null, onactivate });

  const button = screen.getByRole("button", { name: "Google Mail" });
  expect(button).toHaveTextContent("GM");
  expect(button).toHaveAttribute("aria-current", "page");

  await fireEvent.click(button);
  expect(onactivate).toHaveBeenCalledOnce();
});

test("renders the icon image, count badges and dot badges", async () => {
  const { rerender } = render(AppIcon, {
    app: app("a"),
    status: { kind: "running" },
    badge: 120,
    active: false,
    iconUrl: "data:image/png;base64,x",
    onactivate: () => {},
  });
  expect(screen.getByRole("button", { name: "A" }).querySelector("img")).toHaveAttribute("src", "data:image/png;base64,x");
  expect(screen.getByTestId("badge")).toHaveTextContent("99+");

  await rerender({ badge: "dot" });
  expect(screen.getByTestId("badge")).toHaveClass("dot");
});

test("hibernated apps are dimmed and errors replace the badge", async () => {
  const { rerender } = render(AppIcon, {
    app: app("a"),
    status: { kind: "hibernated" },
    badge: 2,
    active: false,
    iconUrl: null,
    onactivate: () => {},
  });
  expect(screen.getByRole("button", { name: "A" })).toHaveClass("hibernated");

  await rerender({ status: { kind: "error", message: "boom" } });
  expect(screen.queryByTestId("badge")).toBeNull();
  expect(screen.getByLabelText("Error")).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "A" })).toHaveAttribute("title", "A: boom");
});

test("context menu handler receives the event", async () => {
  const oncontextmenu = vi.fn();
  render(AppIcon, { app: app("a"), status: { kind: "running" }, active: false, iconUrl: null, onactivate: () => {}, oncontextmenu });
  await fireEvent.contextMenu(screen.getByRole("button", { name: "A" }));
  expect(oncontextmenu).toHaveBeenCalledOnce();
});
