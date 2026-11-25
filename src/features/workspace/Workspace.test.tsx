// @vitest-environment jsdom
import { afterEach, expect, it, vi } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Workspace } from "./Workspace";
import { desktop, type Job } from "../../lib/desktop";

const original: Job = {
  id: "A", images: ["a1","a2"], prompt: "Picaso icon", created: 1,
  provider: "fixture", model: "fixture", status: "succeeded", message: null, inputId: null,
};
afterEach(() => { cleanup(); vi.restoreAllMocks(); vi.unstubAllGlobals(); });

async function openWorkspace(canEdit = true) {
  vi.stubGlobal("__TAURI_INTERNALS__", {convertFileSrc: (id: string) => "/image/" + id});
  vi.spyOn(desktop,"history").mockResolvedValue([original]);
  vi.spyOn(desktop,"model").mockResolvedValue({provider:"fixture", id:"fixture", canEdit});
  const request = vi.spyOn(desktop,"submit").mockRejectedValue("The original image is missing.");
  render(<Workspace status={{selected:null,environment:false,keychain:false,keychainError:false}} openSettings={() => {}}/>);
  const user = userEvent.setup();
  await user.click(await screen.findByRole("button", {name:"Select image 2"}));
  await user.type(screen.getByLabelText("Image prompt"), "Add a copper edge");
  await user.click(screen.getByRole("button", {name:"Continue editing"}));
  return {user, request};
}
it("selecting a single original preserves the draft and only an explicit submit sends it", async () => {
  const {user, request} = await openWorkspace();
  const input = screen.getByLabelText("Edit instructions") as HTMLTextAreaElement;
  expect(document.activeElement).toBe(input);
  expect(input.value).toBe("Add a copper edge");
  expect(request).not.toHaveBeenCalled();
  await user.click(screen.getByRole("button",{name:"Apply edit"}));
  expect(request).toHaveBeenCalledTimes(1);
  expect(request).toHaveBeenCalledWith({requestId:expect.any(String),prompt:input.value,inputId:"a2"});
  expect(await screen.findByRole("alert")).toHaveProperty("textContent","The original image is missing.");
  expect(input.value).toBe("Add a copper edge");
});
it("holds a draft when the selected model cannot edit", async () => {
  const {user,request} = await openWorkspace(false);
  await user.click(screen.getByRole("button",{name:"Apply edit"}));
  expect(request).not.toHaveBeenCalled();
  expect((screen.getByLabelText("Edit instructions") as HTMLTextAreaElement).value).toBe("Add a copper edge");
});
