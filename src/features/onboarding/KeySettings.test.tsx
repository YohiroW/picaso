// @vitest-environment jsdom
import { afterEach, expect, it, vi } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { KeySettings } from "./KeySettings";
import { desktop } from "../../lib/desktop";
afterEach(() => { cleanup(); vi.restoreAllMocks(); });
it("detects an environment key but requires an explicit choice", async () => {
  const state = { platform: "windows", selected: null, environment: true, keychain: false, keychainError: false };
  const select = vi.spyOn(desktop, "select").mockResolvedValue({ ...state, selected: "environment" });
  const close = vi.fn();
  render(<KeySettings status={state} onChange={vi.fn()} onClose={close}/>);
  expect(select).not.toHaveBeenCalled();
  await userEvent.click(screen.getByRole("button", {name: "Use environment key"}));
  expect(select).toHaveBeenCalledWith("environment");
  expect(close).toHaveBeenCalledOnce();
});
it("offers masked entry and optional browsing when no key exists", async () => {
  const close = vi.fn();
  render(<KeySettings status={{platform:"macos",selected:null,environment:false,keychain:false,keychainError:false}} onChange={vi.fn()} onClose={close}/>);
  expect(screen.getByLabelText("OpenAI API key").getAttribute("type")).toBe("password");
  await userEvent.click(screen.getByRole("button", { name: "Back to workspace" }));
  expect(close).toHaveBeenCalledOnce();
});

it("keeps saved-key removal behind a confirmation in the API key section", async () => {
  const status = { platform: "macos", selected: "keychain" as const, environment: false, keychain: true, keychainError: false };
  const remove = vi.spyOn(desktop, "remove").mockResolvedValue({ ...status, keychain: false });
  const onChange = vi.fn();
  const onClose = vi.fn();
  const user = userEvent.setup();
  render(<KeySettings status={status} onChange={onChange} onClose={onClose}/>);
  await user.click(screen.getByRole("button", { name: "Manage key" }));
  await user.click(screen.getByRole("button", { name: "Remove saved key" }));
  await user.click(screen.getByRole("button", { name: "Cancel" }));
  expect(remove).not.toHaveBeenCalled();
  await user.click(screen.getByRole("button", { name: "Remove saved key" }));
  await user.click(screen.getByRole("button", { name: "Confirm removal" }));
  expect(remove).toHaveBeenCalledOnce();
  expect(onChange).toHaveBeenCalledWith({ ...status, keychain: false });
  expect(onClose).not.toHaveBeenCalled();
});

it("shows environment setup on Windows and opens the portal on the first activation", async () => {
  const status = { platform: "windows", selected: null, environment: false, keychain: false, keychainError: false };
  const open = vi.spyOn(desktop, "openPortal").mockResolvedValue(undefined);
  const user = userEvent.setup();
  render(<KeySettings status={status} onChange={vi.fn()} onClose={vi.fn()}/>);
  expect(screen.getByRole("heading", {name: "Connection"})).toBeTruthy();
  expect(screen.queryByText(/keychain|add key|save and use key/i)).toBeNull();
  expect(screen.queryByRole("button", {name: "API key"})).toBeNull();
  expect(screen.getByText(/then restart Picaso/)).toBeTruthy();
  const link = screen.getByRole("link", {name: "Get an API key"});
  expect(link.getAttribute("href")).toBe("https://platform.openai.com/api-keys");
  link.focus();
  await user.keyboard("{Enter}");
  expect(open).toHaveBeenCalledOnce();
  expect(screen.queryByRole("alert")).toBeNull();

  open.mockRejectedValueOnce("Couldn’t open your browser.");
  await user.click(link);
  expect(screen.getByRole("alert").textContent).toContain("Couldn’t open your browser.");
  await user.click(link);
  expect(screen.queryByRole("alert")).toBeNull();
});
