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
  await userEvent.click(screen.getByText("Browse without a key"));
  expect(close).toHaveBeenCalledOnce();
});
