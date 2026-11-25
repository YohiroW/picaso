// @vitest-environment jsdom
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Workspace } from "./Workspace";
import bundled from "../../../src-tauri/models.json";
import { desktop, type Job } from "../../lib/desktop";

const original: Job = {
  parameters: {size: "1024x1024", quality:"low",output_format:"png"},
  id: "A", images: ["a1","a2"], prompt: "Picaso icon", created: 1,
  provider: "fixture", model: "fixture", status: "succeeded", message: null, inputId: null,
};
beforeEach(() => {
  vi.stubGlobal("ResizeObserver", class { observe() {} disconnect() {} });
  // jsdom omits the native dialog methods used by WKWebView.
  HTMLDialogElement.prototype.showModal = function () { this.setAttribute("open", ""); };
  HTMLDialogElement.prototype.close = function () { this.removeAttribute("open"); };
});
afterEach(() => { cleanup(); vi.restoreAllMocks(); vi.unstubAllGlobals(); });

async function openWorkspace(canEdit = true) {
  vi.stubGlobal("__TAURI_INTERNALS__", {convertFileSrc: (id: string) => "/image/" + id});
  vi.spyOn(desktop,"history").mockResolvedValue([original]);
  vi.spyOn(desktop,"models").mockResolvedValue([{...bundled.models[0],provider:"fixture",id:"fixture",canEdit}]);
  const request = vi.spyOn(desktop,"submit").mockRejectedValue("The original image is missing.");
  render(<Workspace status={{selected:null,environment:false,keychain:false,keychainError:false}} openSettings={() => {}}/>);
  const user = userEvent.setup();
  await user.type(screen.getByLabelText("Prompt"), "Add a copper edge");
  await user.click(await screen.findByRole("button", {name:"View image 2: Picaso icon"}));
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
  expect(request).toHaveBeenCalledWith({requestId:expect.any(String),model:"fixture",prompt:input.value,inputId:"a2",options:bundled.models[0].defaults});
  expect(await screen.findByRole("alert")).toHaveProperty("textContent","The original image is missing.");
  expect(input.value).toBe("Add a copper edge");
});
it("holds a draft when the selected model cannot edit", async () => {
  const {user,request} = await openWorkspace(false);
  await user.click(screen.getByRole("button",{name:"Apply edit"}));
  expect(request).not.toHaveBeenCalled();
  expect((screen.getByLabelText("Edit instructions") as HTMLTextAreaElement).value).toBe("Add a copper edge");
});

it("does not submit an edit while an input method is composing text", async () => {
  const { request } = await openWorkspace();
  const input = screen.getByLabelText("Edit instructions");
  fireEvent.change(input, {target: {value:"保留折纸纹理"}});
  fireEvent.keyDown(input, {key:"Enter", metaKey:true, isComposing:true, keyCode:229});
  expect(request).not.toHaveBeenCalled();
  fireEvent.keyDown(input, {key:"Enter", metaKey:true, isComposing:false});
  await waitFor(() => expect(request).toHaveBeenCalledOnce());
  expect(request.mock.calls[0][0].prompt).toBe("保留折纸纹理");
});

it("limits the studio by image count and preserves drafts through full task history", async () => {
  vi.stubGlobal("__TAURI_INTERNALS__", { convertFileSrc: (id: string) => "/image/" + id });
  const jobs = Array.from({length: 6}, (_, i) => ({
    ...original, id: String(i), prompt: "Study " + i, images: [i + "-first", i + "-second"]
  }));
  const failed = {...original, id:"failed", prompt:"Another study", status:"unconfirmed", images:[], message:"Result unconfirmed."};
  const history = vi.spyOn(desktop, "history").mockImplementation(async recent => recent ? jobs : [failed, ...jobs]);
  vi.spyOn(desktop,"models").mockResolvedValue([{...bundled.models[0],provider:"fixture",id:"test"}]);
  const request = vi.spyOn(desktop, "submit").mockResolvedValue("B");
  render(<Workspace status={{selected:null,environment:false,keychain:false,keychainError:false}} openSettings={() => {}}/>);
  const user = userEvent.setup();
  const images = await screen.findAllByRole("button", { name: /^View image/ });
  expect(images).toHaveLength(8);
  expect(images[0].getAttribute("aria-label")).toContain("Study 3");
  expect(images[0].textContent).toBe("Study 3");
  expect(images[7].nextElementSibling).toBe(screen.getByRole("button",{name:"New image"}));
  const prompt = screen.getByLabelText("Prompt");
  await user.type(prompt, "New landscape");
  await user.selectOptions(screen.getByLabelText("Size & orientation"), "1536x1024");
  await user.selectOptions(screen.getByLabelText("Quality"), "medium");
  await user.click(screen.getByRole("button", { name: "History" }));
  expect(await screen.findByText(/Result unconfirmed\./)).toBeTruthy();
  expect(await screen.findAllByRole("button",{name:/^View image/})).toHaveLength(12);
  expect(history).toHaveBeenCalledWith(false);
  await user.click(screen.getByRole("button",{name:"View image 2: Study 5"}));
  await user.click(screen.getByRole("button",{name:"Continue editing"}));
  await user.type(screen.getByLabelText("Edit instructions"), " with clouds");
  await user.click(screen.getByRole("button",{name:"Back to studio"}));
  await user.click(await screen.findByRole("button",{name:"New image"}));
  expect((prompt as HTMLTextAreaElement).value).toBe("New landscape");
  expect(document.activeElement).toBe(prompt);
  expect(request).not.toHaveBeenCalled();
  await user.click(screen.getByRole("button",{name:"Generate image"}));
  expect(request).toHaveBeenCalledOnce();
  expect(request.mock.calls[0][0].options).toEqual({...bundled.models[0].defaults,size:"1536x1024",quality:"medium"});
});

it("submits the selected catalog model, custom dimensions, count, format and background together", async () => {
  vi.stubGlobal("__TAURI_INTERNALS__", { convertFileSrc: (id: string) => "/image/" + id });
  vi.spyOn(desktop,"models").mockResolvedValue(bundled.models);
  vi.spyOn(desktop,"history").mockResolvedValue([]);
  const submit=vi.spyOn(desktop,"submit").mockResolvedValue("B");
  render(<Workspace status={{selected:"environment",environment:true,keychain:false,keychainError:false}} openSettings={()=>{}}/>);
  const user=userEvent.setup();
  await user.type(screen.getByLabelText("Prompt"),"Paper icon");
  const model = await screen.findByRole("combobox", {name: "Model"}) as HTMLSelectElement;
  expect(model.value).toBe("gpt-image-2.6-flare");
  expect(model.selectedOptions[0].textContent).toBe("GPT-Image-2.6 Flare");
  await user.selectOptions(model, "GPT-Image-2.6 Sunburst");
  await user.selectOptions(screen.getByLabelText("Size & orientation"),"custom");
  const width=screen.getByLabelText("Width (px)");
  fireEvent.change(width,{target:{value:"1599"}});
  expect(screen.getByRole("button",{name:"Generate image"})).toHaveProperty("disabled",true);
  fireEvent.change(width,{target:{value:"1600"}});
  await user.selectOptions(screen.getByLabelText("Quality"),"xhigh");
  fireEvent.change(screen.getByRole("slider"),{target:{value:"3"}});
  expect(screen.getByRole("slider", {name: "Number of images"})).toHaveProperty("value", "3");
  expect(screen.getByText("3", {selector: ".range-ticks .selected"})).toBeTruthy();
  await user.click(screen.getByText("Advanced"));
  const fmt = screen.getByLabelText("Output format") as HTMLSelectElement;
  expect(Array.from(fmt.options).find(o=>o.value==="svg")?.disabled).toBe(true);
  await user.selectOptions(fmt,"webp");
  await user.selectOptions(screen.getByLabelText("Background"),"transparent");
  expect(submit).not.toHaveBeenCalled();
  await user.click(screen.getByRole("button",{name:"Generate 3 images"}));
  expect(submit).toHaveBeenCalledOnce();
  expect(submit.mock.calls[0][0]).toMatchObject({model:"gpt-image-2.6-sunburst",prompt:"Paper icon",options:{
    size:"1600x1024",quality:"xhigh",n:3,output_format:"webp",background:"transparent"
  }});
});
