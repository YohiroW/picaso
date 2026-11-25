// @vitest-environment jsdom
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
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
  const tile = await screen.findByRole("button", {name:"View image 2: Picaso icon"});
  fireEvent.load(tile.querySelector("img")!);
  await user.click(tile);
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

it("copies the full prompt and returns from image-only view to the same original", async () => {
  const { user, request } = await openWorkspace();
  const job = { ...original, prompt: "保留纹理\n  Second line — 完整文本" };
  vi.mocked(desktop.history).mockResolvedValue([job]);
  await user.click(screen.getByRole("button", { name: "History" }));
  await user.click(await screen.findByRole("button", { name: /^View image 2: 保留纹理/ }));
  const details = screen.getByRole("dialog", { name: "Image preview" });
  const copyButton = within(details).getByRole("button", { name: "Copy prompt" });
  expect(copyButton.textContent).toBe("");
  expect(copyButton.title).toBe("Copy prompt");
  await user.click(copyButton);
  expect(copyButton.title).toBe("Prompt copied");
  expect(await navigator.clipboard.readText()).toBe(job.prompt);
  expect(within(details).getByRole("status").textContent).toBe("Prompt copied.");
  await user.click(within(details).getByRole("button", { name: "View image only" }));
  const viewer = screen.getByRole("dialog", { name: "Image only" });
  expect(within(viewer).getByRole("img").getAttribute("src")).toBe("/image/a2");
  expect(within(viewer).queryByRole("button", { name: "Copy prompt" })).toBeNull();
  expect(within(viewer).queryByText("Continue editing")).toBeNull();
  fireEvent(viewer, new Event("cancel", { cancelable: true }));
  expect(within(details).getByRole("button", { name: "View image only" })).toBe(document.activeElement);
  await user.click(within(details).getByRole("button", { name: "Close" }));
  expect(screen.queryByRole("dialog")).toBeNull();
  expect(request).not.toHaveBeenCalled();
});

it("confirms deletion of the chosen original, keeps its sibling and refreshes History", async () => {
  const { user, request } = await openWorkspace();
  const remove = vi.spyOn(desktop, "deleteImage").mockRejectedValueOnce("Couldn’t erase the file.")
    .mockImplementation(async id => {
      vi.mocked(desktop.history).mockResolvedValue([{ ...original, images: original.images.filter(image => image !== id) }]);
    });
  const tile = screen.getByRole("button", { name: "View image 2: Picaso icon" });
  fireEvent.contextMenu(tile, { clientX: 80, clientY: 100 });
  await user.click(await screen.findByRole("menuitem", { name: "Delete image…" }));
  await user.click(screen.getByRole("button", { name: "Cancel" }));
  expect(remove).not.toHaveBeenCalled();

  tile.focus();
  await user.keyboard("{Shift>}{F10}{/Shift}");
  await user.click(await screen.findByRole("menuitem", { name: "Delete image…" }));
  const confirm = screen.getByRole("dialog", { name: "Delete this image?" });
  await user.click(within(confirm).getByRole("button", { name: "Delete image" }));
  expect(within(confirm).getByRole("alert").textContent).toContain("Couldn’t erase");
  await user.click(within(confirm).getByRole("button", { name: "Delete image" }));
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  expect(remove.mock.calls).toEqual([["a2"], ["a2"]]);
  expect(screen.getByText("Reference image unavailable")).toBeTruthy();
  expect(screen.getByLabelText("Edit instructions")).toHaveProperty("value", "Add a copper edge");
  expect(screen.getByRole("button", { name: "Apply edit" })).toHaveProperty("disabled", true);
  for (const view of ["Recent creations", "Task history"]) {
    if (view === "Task history") await user.click(screen.getByRole("button", { name: "History" }));
    const region = within(screen.getByRole("region", { name: view }));
    const remaining = await region.findByRole("button", { name: "View image 1: Picaso icon" });
    expect(remaining.querySelector("img")?.getAttribute("src")).toBe("/image/a1");
    expect(region.queryByRole("button", { name: /^View image 2/ })).toBeNull();
  }
  expect(request).not.toHaveBeenCalled();
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
  fireEvent.load(images[0].querySelector("img")!);
  expect(images[0].textContent).toBe("Study 3");
  expect(images[7].parentElement?.nextElementSibling).toBe(screen.getByRole("button",{name:"New image"}));
  const prompt = screen.getByLabelText("Prompt");
  await user.type(prompt, "New landscape");
  await user.selectOptions(screen.getByLabelText("Size & orientation"), "1536x1024");
  await user.selectOptions(screen.getByLabelText("Quality"), "medium");
  const historyButton = screen.getByRole("button", { name: "History", pressed: false });
  expect(historyButton.textContent).toBe("");
  await user.click(historyButton);
  expect(await screen.findByText(/Result unconfirmed\./)).toBeTruthy();
  expect(await screen.findAllByRole("button",{name:/^View image/})).toHaveLength(12);
  expect(history).toHaveBeenCalledWith(false);
  await user.click(screen.getByRole("button",{name:"View image 2: Study 5"}));
  await user.click(screen.getByRole("button",{name:"Continue editing"}));
  await user.type(screen.getByLabelText("Edit instructions"), " with clouds");
  expect(screen.getByRole("button", { name: "History", pressed: true }).title).toBe("Back to studio");
  await user.click(historyButton);
  expect(historyButton.getAttribute("aria-pressed")).toBe("false");
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

it.each([false, true])("replaces three landscape placeholders in an eight-image gallery (visit History: %s)", async visitHistory => {
  vi.stubGlobal("__TAURI_INTERNALS__", { convertFileSrc: (id: string) => "/image/" + id });
  const jobs = Array.from({ length: 8 }, (_, i) => ({
    ...original, id: "old-" + i, images: ["image-" + i], prompt: "Study " + i,
  }));
  vi.spyOn(desktop, "history").mockImplementation(async recent => jobs.slice(0, recent ? 8 : undefined));
  vi.spyOn(desktop, "models").mockResolvedValue(bundled.models);
  let finish: () => void;
  vi.spyOn(desktop, "submit").mockImplementation(draft => {
    const job = { ...original, id: draft.requestId, parameters: draft.options, prompt: draft.prompt, images: [] as string[], status: "running" };
    jobs.unshift(job);
    return new Promise(resolve => {
      finish = () => {
        job.images = ["hill-1", "hill-2", "hill-3"];
        job.status = "succeeded";
        resolve(job.id);
      };
    });
  });
  render(<Workspace status={{selected:"environment",environment:true,keychain:false,keychainError:false}} openSettings={() => {}}/>);
  const user = userEvent.setup();
  expect(await screen.findAllByRole("button", { name: /^View image/ })).toHaveLength(8);
  await user.type(screen.getByLabelText("Prompt"), "Wooded hills");
  await user.selectOptions(screen.getByLabelText("Size & orientation"), "1536x1024");
  fireEvent.change(screen.getByRole("slider"), { target: { value: "3" } });
  await user.click(screen.getByRole("button", { name: "Generate 3 images" }));
  const placeholders = screen.getAllByText("Generating…").map(label => label.parentElement!);
  expect(placeholders).toHaveLength(3);
  expect(screen.getByRole("status").textContent).toBe("Generating 3 images…");
  expect(screen.getAllByRole("button", { name: /^View image/ })).toHaveLength(5);
  const positions = placeholders.map(tile => {
    expect(tile.querySelector("button")).toBeNull();
    expect(parseFloat(tile.style.width) / parseFloat(tile.style.height)).toBeCloseTo(1.5);
    return tile.style.cssText;
  });
  const history = screen.getByRole("button", { name: "History" });
  if (visitHistory) {
    await user.click(history);
    await screen.findByText("running");
    expect(within(screen.getByRole("list")).getAllByText("Wooded hills")).toHaveLength(1);
  }
  await act(async () => finish());
  if (visitHistory) {
    expect(screen.getByRole("heading", { name: "History" })).toBeTruthy();
    expect(screen.getAllByRole("button", { name: /^View image/ })).toHaveLength(11);
    await user.click(history);
  }
  const results = await screen.findAllByRole("button", { name: /View image .*: Wooded hills/ });
  expect(results).toHaveLength(3);
  results.forEach((button, i) => {
    const tile = button.parentElement!;
    if (!visitHistory) expect(tile).toBe(placeholders[i]);
    expect(tile.style.cssText).toBe(positions[i]);
    expect(button).toHaveProperty("disabled", true);
    expect(within(tile).getByText("Loading image…")).toBeTruthy();
    fireEvent.load(button.querySelector("img")!);
    expect(button).toHaveProperty("disabled", false);
    expect(within(tile).queryByText("Loading image…")).toBeNull();
  });
  expect(screen.getAllByRole("button", { name: /^View image/ })).toHaveLength(8);
  expect(screen.queryByText("Generating…")).toBeNull();
});

it.each(["failed", "unconfirmed"])("restores a %s edit as a draft for explicit resubmission", async status => {
  vi.stubGlobal("__TAURI_INTERNALS__", { convertFileSrc: (id: string) => "/image/" + id });
  const job = { ...original, status, id: "edit-1", inputId: "a1", prompt: "Wider sky",
    parameters: { ...original.parameters, size: "1536x1024" }, images: [],
    message: status === "failed" ? "Check your connection." : "Result unconfirmed. Submitting again may incur another charge." };
  vi.spyOn(desktop, "history").mockResolvedValue([job, original]);
  vi.spyOn(desktop, "models").mockResolvedValue([{ ...bundled.models[0], provider: "fixture", id: "fixture" }]);
  const submit = vi.spyOn(desktop, "submit").mockResolvedValue("retry");
  render(<Workspace status={{selected:null,environment:false,keychain:false,keychainError:false}} openSettings={() => {}}/>);
  const user = userEvent.setup();
  await screen.findByText(status === "failed" ? "Generation failed" : "Result unconfirmed");
  expect(screen.queryByText("Generating…")).toBeNull();
  await user.click(screen.getByRole("button", { name: "Edit prompt: Wider sky" }));
  expect(screen.getByLabelText("Edit instructions")).toHaveProperty("value", job.prompt);
  expect(screen.getByLabelText("Size & orientation")).toHaveProperty("value", job.parameters.size);
  expect(screen.getByRole("alert").textContent).toContain(status === "failed" ? "Check your connection" : "another charge");
  expect(submit).not.toHaveBeenCalled();
  await user.click(screen.getByRole("button", { name: "Apply edit" }));
  expect(submit.mock.calls[0][0]).toMatchObject({ prompt: job.prompt, inputId: "a1" });
});
