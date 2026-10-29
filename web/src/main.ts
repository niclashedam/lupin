import "./style.css";
import init, { embed, extract } from "../pkg/lupin.js";
import { inspect, merge, type Region, type Structure } from "./structure.ts";

declare const __LUPIN_VERSION__: string;

const samples = import.meta.glob("../../examples/{cat.*,out.png}", { query: "?url", import: "default", eager: true }) as Record<string, string>;

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
const wide = matchMedia("(min-width: 640px)");

let current: { name: string; data: Uint8Array } | null = null;
let hidden: Uint8Array | null = null;
// What the byte view shows, so it can be redrawn when the hex row width changes
let shown: { name: string; structure: Structure; usable: boolean } | null = null;
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

/** Loads a file once its bytes are read; `read` is awaited here so a newer load wins */
async function load(name: string, read: () => Promise<ArrayBuffer>, size = 0) {
  const mine = ++seq;
  let data: Uint8Array | null = null;
  let error = "";
  if (size > MAX_BYTES) {
    error = tooBig(`${name} (${bytes(size)})`);
  } else {
    status(`Reading ${name}…`);
    try {
      data = new Uint8Array(await read());
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
  shown = null;
  form.reset();
  toggleKind();
  show(form, false);
  show($("reveal"), false);
  const structure = current && inspect(current.data);
  show($("result"), !!structure);
  if (error || !data) {
    status(error);
  } else if (!structure) {
    status(`Lupin cannot use ${name} because it is not a PDF, PNG, JPEG, MKV or WebM file. Choose a file in one of those formats.`);
  } else {
    let usable = structure.usable;
    try {
      hidden = extract(data) ?? null;
      if (hidden) {
        const text = asText(hidden);
        status(`${name} hides ${bytes(hidden.length)}${text === null ? "." : ":"}`);
        $("message").textContent = text ?? "";
        show($("message"), text !== null);
        show($("reveal"), true);
      } else if (!usable) {
        status(`${structure.insertNote} Choose a different file.`);
      } else {
        status(`Lupin found no hidden data in ${name} (${structure.format}, ${bytes(data.length)}).`);
        show(form, true);
      }
    } catch (err) {
      usable = false;
      status(`Lupin cannot read ${name} (${reason(err)}). Choose a different file.`);
    }
    render(name, structure, usable, false);
  }
  // Focusing the status scrolls the result into view and keeps keyboard focus visible
  $("status").focus();
}

function render(name: string, s: Structure, usable: boolean, animate: boolean) {
  shown = { name, structure: s, usable };
  // Only highlight what the engine actually extracted, so the view never contradicts the status
  const regions = merge(s.regions.map((r) => ({ ...r, lupin: r.lupin && hidden !== null })));
  const lupin = regions.find((r) => r.lupin);
  const total = s.regions.at(-1)?.end ?? 0;
  const unit = { PDF: "parts", PNG: "chunks", JPEG: "segments", MKV: "elements" }[s.format];
  $("inside").textContent = `Inside ${name}`;
  $("legend").textContent = `${bytes(total)}, ${s.regions.length} ${unit}`;
  $("note").textContent = lupin
    ? `Highlighted: the ${bytes(hidden!.length)} of hidden data take up ${bytes(lupin.end - lupin.start)} in the file. ${s.insertNote}`
    : usable
      ? `${s.insertNote} The yellow marker shows where.`
      : "";
  $("part").textContent = "Select a part of the strip to see its size and offset.";
  strip(regions, total, lupin || !usable ? null : s.insertAt);
  hex(current!.data, lupin, animate);
}

function strip(regions: Region[], total: number, insertAt: number | null) {
  const el = $("strip");
  const caret = () => {
    const c = document.createElement("div");
    c.setAttribute("role", "listitem");
    c.setAttribute("aria-label", "Lupin inserts its data here");
    c.title = "Lupin inserts its data here";
    c.className = "w-1.5 shrink-0 border-x border-ink bg-mark";
    return c;
  };
  el.replaceChildren();
  // ponytail: with hundreds of parts, runs of parts too thin to see become one cell, so the
  // strip stays cheap to draw; files with a normal number of parts keep every label
  if (regions.length > 200) {
    const thin = (r: Region) => !r.lupin && r.end - r.start < total / 300 && r.start !== insertAt;
    regions = regions.reduce<Region[]>((cells, r) => {
      const last = cells.at(-1);
      if (last && thin(last) && thin(r)) cells[cells.length - 1] = { ...last, name: "small parts", end: r.end };
      else cells.push(r);
      return cells;
    }, []);
  }
  for (const r of regions) {
    const size = r.end - r.start;
    if (insertAt === r.start && insertAt > 0) el.append(caret());
    const cell = document.createElement("div");
    const info = `${r.name}: ${bytes(size)} from offset 0x${r.start.toString(16)}`;
    cell.setAttribute("role", "listitem");
    cell.setAttribute("aria-label", info);
    cell.tabIndex = 0;
    cell.title = info;
    // Small regions get a minimum width so headers stay visible next to megabytes of image data
    cell.style.flex = `${Math.max(size, total * 0.06)} 1 0`;
    cell.className = `truncate border-r border-rule px-2 text-xs leading-10 last:border-r-0 focus:outline-2 focus:-outline-offset-2 focus:outline-ink ${
      r.lupin ? "min-w-max bg-mark font-medium text-mark-ink" : "min-w-0 text-muted"
    }`;
    cell.textContent = r.name;
    for (const type of ["focus", "mouseenter"]) cell.addEventListener(type, () => ($("part").textContent = info));
    el.append(cell);
  }
  if (insertAt === total) el.append(caret());
}

function hex(data: Uint8Array, lupin: Region | undefined, animate: boolean) {
  const el = $("hex");
  show(el, !!lupin);
  el.classList.toggle("sweep", animate);
  if (!lupin) return;
  const width = wide.matches ? 16 : 8;
  const first = Math.max(0, Math.floor(lupin.start / width) - 2);
  const lastLupinRow = Math.floor((lupin.end - 1) / width);
  const last = Math.min(Math.floor((data.length - 1) / width), lastLupinRow + 2, first + 13);
  const rows: HTMLElement[] = [];
  for (let row = first; row <= last; row++) {
    const line = document.createElement("div");
    line.className = "whitespace-pre";
    const offset = document.createElement("span");
    offset.className = "text-muted";
    offset.textContent = (row * width).toString(16).padStart(8, "0") + "  ";
    line.append(offset);
    const hexCol: Node[] = [];
    const asciiCol: Node[] = [];
    for (let i = row * width; i < (row + 1) * width; i++) {
      const sep = i % width === width - 1 ? "" : i % 8 === 7 ? "  " : " ";
      if (i >= data.length) {
        hexCol.push(document.createTextNode("  " + sep));
        continue;
      }
      const inside = i >= lupin.start && i < lupin.end;
      const h = data[i].toString(16).padStart(2, "0");
      const c = data[i] >= 0x20 && data[i] < 0x7f ? String.fromCharCode(data[i]) : ".";
      for (const [col, text] of [
        [hexCol, h],
        [asciiCol, c],
      ] as const) {
        const span = document.createElement("span");
        span.textContent = text;
        if (inside) {
          span.className = "mark bg-mark text-mark-ink";
          // Sweeps left to right, row by row
          span.style.setProperty("--i", String((row - first) * width + (i % width)));
        }
        col.push(span);
        // Keep the highlight continuous across the gap between hex bytes
        if (col === hexCol && sep) {
          const gap = document.createElement("span");
          gap.textContent = sep;
          if (inside && i + 1 < lupin.end) gap.className = span.className;
          gap.style.cssText = span.style.cssText;
          col.push(gap);
        }
      }
    }
    line.append(...hexCol, document.createTextNode("  "), ...asciiCol);
    rows.push(line);
  }
  if (lastLupinRow > last) {
    const more = document.createElement("div");
    more.className = "mt-1 text-muted";
    more.textContent = `… ${bytes(lupin.end - (last + 1) * width)} more`;
    rows.push(more);
  }
  el.replaceChildren(...rows);
}

// Inputs

const dropZone = $("drop");
const openFile = (file: File) => load(file.name, () => file.arrayBuffer(), file.size);

$<HTMLInputElement>("file").addEventListener("change", (e) => {
  const input = e.target as HTMLInputElement;
  const file = input.files?.[0];
  if (file) openFile(file);
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
  if (file) openFile(file);
});
dropZone.addEventListener("dragover", () => dropZone.setAttribute("data-over", ""));
dropZone.addEventListener("dragleave", () => dropZone.removeAttribute("data-over"));

for (const button of document.querySelectorAll<HTMLButtonElement>(".sample")) {
  button.addEventListener("click", () => {
    const name = button.dataset.src!;
    load(name, () =>
      fetch(samples[`../../examples/${name}`]).then((res) => {
        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        return res.arrayBuffer();
      }),
    );
  });
}

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
    status(`Hid ${bytes(payload.length)} in ${target.name} and downloaded the result as ${name}. Drop that file here to see the hidden data.`);
    $("status").focus();
    current = { name, data: output };
    hidden = payload;
    render(name, inspect(output)!, true, true);
  } catch (err) {
    status(`Lupin could not hide the data in ${target.name} (${reason(err)}). Choose a different file.`);
  }
});

$("save").addEventListener("click", () => {
  if (!hidden || !current) return;
  const base = current.name.replace(/\.[^.]*$/, "");
  download(hidden, `${base}-hidden.${asText(hidden) === null ? "bin" : "txt"}`);
});

wide.addEventListener("change", () => shown && render(shown.name, shown.structure, shown.usable, false));
