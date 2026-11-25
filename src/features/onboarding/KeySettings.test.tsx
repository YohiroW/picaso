// @vitest-environment jsdom
import { afterEach, expect, it, vi } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { KeySettings } from "./KeySettings";
import { desktop } from "../../lib/desktop";
afterEach(() => { cleanup(); vi.restoreAllMocks(); });
it("detects an environment key but requires an explicit choice", async () => {
  const state = { selected: null, environment: true, keychain: false, keychainError: false };
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
  render(<KeySettings status={{selected:null,environment:false,keychain:false,keychainError:false}} onChange={vi.fn()} onClose={close}/>);
  expect(screen.getByLabelText("OpenAI API key").getAttribute("type")).toBe("password");
  await userEvent.click(screen.getByRole("button", { name: "Back to workspace" }));
  expect(close).toHaveBeenCalledOnce();
});

it("keeps saved-key removal behind a confirmation in the API key section", async () => {
  const status = { selected: "keychain" as const, environment: false, keychain: true, keychainError: false };
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
