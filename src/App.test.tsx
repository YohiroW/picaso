// @vitest-environment jsdom
import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";
import { App } from "./App";
import { desktop } from "./lib/desktop";
import catalog from "../core/models.json";

afterEach(() => { cleanup(); vi.restoreAllMocks(); vi.unstubAllGlobals(); });

it("returns from either settings section to the same history, draft and keyboard focus", async () => {
  vi.stubGlobal("ResizeObserver", class { observe() {} disconnect() {} });
  vi.spyOn(desktop, "status").mockResolvedValue({
    selected: "environment", environment: true, keychain: false, keychainError: false,
  });
  vi.spyOn(desktop, "models").mockResolvedValue(catalog.models);
  vi.spyOn(desktop, "history").mockResolvedValue([]);
  const user = userEvent.setup();
  render(<App/>);
  await user.type(await screen.findByLabelText("Prompt"), "Keep this draft");
  await user.click(screen.getByRole("button", { name: "History" }));
  const gear = screen.getByRole("button", { name: "Settings" });
  expect(gear.textContent).toBe("");
  for (const section of ["Connection", "API key"]) {
    await user.click(gear);
    await user.click(screen.getByRole("button", { name: section }));
    expect(screen.getByRole("heading", { name: section })).toBeTruthy();
    await user.click(screen.getByRole("button", { name: "Back to workspace" }));
    expect(document.activeElement).toBe(gear);
    expect(screen.getByLabelText("Prompt")).toHaveProperty("value", "Keep this draft");
    expect(screen.getByRole("heading", { name: "History" })).toBeTruthy();
  }
});
