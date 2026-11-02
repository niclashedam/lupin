import "./style.css";
import init, { embed, extract } from "../pkg/lupin.js";

declare const __LUPIN_VERSION__: string;

const $ = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;
const show = (el: HTMLElement, on: boolean) => el.classList.toggle("hidden", !on);
const bytes = (n: number) =>
  n === 1 ? "1 byte" : n < 1024 ? `${n} bytes` : n < 1048576 ? `${(n / 1024).toFixed(1)} KB` : n < 1073741824 ? `${(n / 1048576).toFixed(1)} MB` : `${(n / 1073741824).toFixed(1)} GB`;
// A wasm trap surfaces as "unreachable"; with large files it is almost always memory
const reason = (err: unknown) =>
  (err instanceof Error ? err.message : String(err)).replace(/\.$/, "").replace(/^unreachable$/, "it ran out of memory or stopped unexpectedly");
// Cover plus payload, held several times over in JS and WebAssembly memory (4 GB at most)
const MAX_BYTES = 1024 ** 3;
const tooBig = (what: string) => `${what} is too large for the website, which handles up to 1 GB. Use the command-line tool instead.`;
/** Lets the browser paint a status message before blocking work starts */
const paint = () => new Promise((r) => requestAnimationFrame(() => setTimeout(r)));

let current: { name: string; data: Uint8Array } | null = null;
let hidden: Uint8Array | null = null;
// Incremented on every load so a slow earlier read cannot overwrite a later one
let seq = 0;

$("version").textContent = `v${__LUPIN_VERSION__}`;
const wasmFailed = "This page could not load Lupin's WebAssembly module. Reload the page to try again.";
const ready = init().then(
  () => true,
  () => (status(wasmFailed), false),
);

function status(text: string) {
  $("status").textContent = text;
}

function download(data: Uint8Array, name: string) {
  const url = URL.createObjectURL(new Blob([data as BlobPart]));
  const a = Object.assign(document.createElement("a"), { href: url, download: name });
  a.click();
  setTimeout(() => URL.revokeObjectURL(url), 60_000);
}

/** Decodes the payload as text if it looks like a message rather than a file */
function asText(data: Uint8Array): string | null {
  if (data.length > 10_000) return null;
  try {
    const text = new TextDecoder("utf-8", { fatal: true }).decode(data);
    return /[\x00-\x08\x0b\x0c\x0e-\x1f\x7f]/.test(text) ? null : text;
  } catch {
    return null;
  }
}

async function load(file: File) {
  const mine = ++seq;
  const name = file.name;
  // Stop a hide started now from embedding into the previous file
  current = null;
  show(form, false);
  let data: Uint8Array | null = null;
  let error = "";
  if (file.size > MAX_BYTES) {
    error = tooBig(`${name} (${bytes(file.size)})`);
  } else {
    status(`Reading ${name}…`);
    try {
      data = new Uint8Array(await file.arrayBuffer());
      if (mine === seq) status(`Looking for hidden data in ${name}…`);
      await paint();
    } catch (err) {
      error = `Lupin could not read ${name} (${reason(err)}). Try again.`;
    }
  }
  if (!(await ready)) error = wasmFailed;
  if (mine !== seq) return;

  current = data && !error ? { name, data } : null;
  hidden = null;
  form.reset();
  toggleKind();
  show(form, false);
  show($("reveal"), false);
  if (!current) {
    status(error);
  } else {
    try {
      hidden = extract(current.data) ?? null;
      if (hidden) {
        const text = asText(hidden);
        status(`${name} hides ${bytes(hidden.length)}${text === null ? "." : ":"}`);
        $("message").textContent = text ?? "";
        show($("message"), text !== null);
        show($("reveal"), true);
      } else {
        status(`Lupin found no hidden data in ${name} (${bytes(current.data.length)}).`);
        show(form, true);
      }
    } catch (err) {
      status(`Lupin cannot use ${name} (${reason(err)}). Choose a different file.`);
    }
  }
  // Focusing the status scrolls the result into view and keeps keyboard focus visible
  $("status").focus();
}

// Inputs

const dropZone = $("drop");

$<HTMLInputElement>("file").addEventListener("change", (e) => {
  const input = e.target as HTMLInputElement;
  const file = input.files?.[0];
  if (file) load(file);
  input.value = ""; // so choosing the same file again still fires change
});
// Drops anywhere load the cover file, except on the form's own inputs (file or text to hide)
const inForm = (e: Event) => (e.target as Element).closest?.("#hide");
window.addEventListener("dragover", (e) => inForm(e) || e.preventDefault());
window.addEventListener("drop", (e) => {
  if (inForm(e)) return;
  e.preventDefault();
  dropZone.removeAttribute("data-over");
  const file = e.dataTransfer?.files[0];
  if (file) load(file);
});
dropZone.addEventListener("dragover", () => dropZone.setAttribute("data-over", ""));
dropZone.addEventListener("dragleave", () => dropZone.removeAttribute("data-over"));

const form = $<HTMLFormElement>("hide");
const kind = () => (form.elements.namedItem("kind") as RadioNodeList).value;
const toggleKind = () => {
  show($("text"), kind() !== "file");
  show($("payload"), kind() === "file");
};
form.addEventListener("change", toggleKind);

form.addEventListener("submit", async (e) => {
  e.preventDefault();
  const target = current;
  const mine = seq;
  if (!target) return;
  const file = kind() === "file" ? $<HTMLInputElement>("payload").files?.[0] : undefined;
  if (file && target.data.length + file.size > MAX_BYTES) {
    status(tooBig(`${target.name} together with ${file.name}`));
    return;
  }
  try {
    status(`Hiding the data in ${target.name}…`);
    const payload = file ? new Uint8Array(await file.arrayBuffer()) : kind() === "file" ? new Uint8Array() : new TextEncoder().encode($<HTMLTextAreaElement>("text").value);
    await paint();
    if (mine !== seq) return; // another file was loaded while the payload was being read
    if (!payload.length) {
      status(kind() === "file" ? "Choose a file to hide first." : "Type a message to hide first.");
      return;
    }
    const output = embed(target.data, payload);
    const dot = target.name.lastIndexOf(".");
    const name = dot > 0 ? `${target.name.slice(0, dot)}-lupin${target.name.slice(dot)}` : `${target.name}-lupin`;
    download(output, name);
    show(form, false);
    status(`Hid ${bytes(payload.length)} in ${target.name} and downloaded the result as ${name}. Open that file above to check the hidden data.`);
    $("status").focus();
  } catch (err) {
    status(`Lupin could not hide the data in ${target.name} (${reason(err)}). Choose a different file.`);
  }
});

$("save").addEventListener("click", () => {
  if (!hidden || !current) return;
  const base = current.name.replace(/\.[^.]*$/, "");
  download(hidden, `${base}-hidden.${asText(hidden) === null ? "bin" : "txt"}`);
});
