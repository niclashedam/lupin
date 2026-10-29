// Walks the top-level structure of the formats Lupin supports, for display only.
// Mirrors where each engine in ../src/engines writes its payload.

export type Region = { name: string; start: number; end: number; lupin: boolean };

export type Structure = {
  format: "PDF" | "PNG" | "JPEG" | "MKV";
  regions: Region[];
  /** Where Lupin inserts its data when embedding */
  insertAt: number;
  /** False when the engine will refuse the file, e.g. a PDF without %%EOF; insertNote says why */
  usable: boolean;
  insertNote: string;
};

const LUPIN_SIG = [0x4c, 0x75, 0x70, 0x69, 0x6e, 0x00]; // "Lupin\0"
// Walkers stop here and the rest shows as "other", so hostile files with millions of tiny
// chunks or segments cannot freeze the page
const MAX_REGIONS = 10_000;

const startsWith = (b: Uint8Array, at: number, seq: number[]) => seq.every((v, i) => b[at + i] === v);
const ascii = (b: Uint8Array, start: number, end: number) => String.fromCharCode(...b.subarray(start, end));

export function inspect(b: Uint8Array): Structure | null {
  const s = startsWith(b, 0, [0x25, 0x50, 0x44, 0x46])
    ? pdf(b)
    : startsWith(b, 0, [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a])
      ? png(b)
      : startsWith(b, 0, [0xff, 0xd8, 0xff])
        ? jpeg(b)
        : startsWith(b, 0, [0x1a, 0x45, 0xdf, 0xa3])
          ? mkv(b)
          : null;
  if (!s) return null;
  // Cover bytes the walkers skipped (truncated chunks, stray markers) so the strip always spans the file
  const regions: Region[] = [];
  let pos = 0;
  for (const r of [...s.regions, { name: "", start: b.length, end: b.length, lupin: false }]) {
    if (r.start > pos) regions.push({ name: "other", start: pos, end: r.start, lupin: false });
    if (r.end > r.start) regions.push(r);
    pos = Math.max(pos, r.end);
  }
  return { ...s, regions };
}

function pdf(b: Uint8Array): Structure {
  const eof = [0x25, 0x25, 0x45, 0x4f, 0x46]; // "%%EOF"
  // Jump between '%' bytes rather than testing every offset
  let pos = b.length < eof.length ? -1 : b.lastIndexOf(0x25, b.length - eof.length);
  while (pos >= 0 && !startsWith(b, pos, eof)) pos = pos > 0 ? b.lastIndexOf(0x25, pos - 1) : -1;
  const regions: Region[] = [];
  if (pos < 0) {
    regions.push({ name: "body (no %%EOF)", start: 0, end: b.length, lupin: false });
    return { format: "PDF", regions, insertAt: b.length, usable: false, insertNote: "This PDF has no %%EOF marker, so Lupin cannot use it." };
  }
  const end = pos + eof.length;
  regions.push({ name: "body", start: 0, end: pos, lupin: false }, { name: "%%EOF", start: pos, end, lupin: false });
  if (end < b.length) {
    // Rust's is_ascii_whitespace, which the PDF engine uses
    const hidden = b.subarray(end).some((c) => ![0x20, 0x09, 0x0a, 0x0c, 0x0d].includes(c));
    regions.push({ name: hidden ? "after %%EOF" : "whitespace", start: end, end: b.length, lupin: hidden });
  }
  return { format: "PDF", regions, insertAt: end, usable: true, insertNote: "Lupin appends Base64 after the last %%EOF. PDF readers ignore anything there." };
}

function png(b: Uint8Array): Structure {
  const v = new DataView(b.buffer, b.byteOffset, b.byteLength);
  const regions: Region[] = [{ name: "signature", start: 0, end: 8, lupin: false }];
  let pos = 8;
  let insertAt = b.length;
  let capped = false;
  while (pos + 12 <= b.length) {
    const type = ascii(b, pos + 4, pos + 8);
    // Chunk types are four ASCII letters; anything else (e.g. a zero-filled tail) is not a chunk
    if (!/^[A-Za-z]{4}$/.test(type)) break;
    if (regions.length >= MAX_REGIONS) {
      capped = true;
      break;
    }
    const end = Math.min(pos + 12 + v.getUint32(pos), b.length);
    if (type === "IEND") insertAt = pos;
    regions.push({ name: type, start: pos, end, lupin: type === "lpNg" });
    pos = end;
  }
  const usable = capped || insertAt < b.length;
  return {
    format: "PNG",
    regions,
    insertAt,
    usable,
    insertNote: usable
      ? "Lupin adds a private lpNg chunk just before IEND and stores the data in it as Base64. Viewers skip chunks they do not know."
      : "This PNG has no IEND chunk, so Lupin cannot use it.",
  };
}

function jpeg(b: Uint8Array): Structure {
  const regions: Region[] = [{ name: "SOI", start: 0, end: 2, lupin: false }];
  let pos = 2;
  let insertAt = -1;
  while (pos + 4 <= b.length && b[pos] === 0xff && regions.length < MAX_REGIONS) {
    const m = b[pos + 1];
    if (insertAt < 0 && !(m >= 0xe0 && m <= 0xef)) insertAt = pos;
    if (m === 0xda) break;
    if (m === 0x01 || (m >= 0xd0 && m <= 0xd8)) {
      pos += 2;
      continue;
    }
    const end = Math.min(pos + 2 + ((b[pos + 2] << 8) | b[pos + 3]), b.length);
    const lupin = m === 0xed && startsWith(b, pos + 4, LUPIN_SIG);
    regions.push({ name: lupin ? "APP13 Lupin" : jpegName(m), start: pos, end, lupin });
    pos = end;
  }
  if (insertAt < 0) insertAt = pos;
  if (pos < b.length) regions.push({ name: "image data", start: pos, end: b.length, lupin: false });
  return {
    format: "JPEG",
    regions,
    insertAt,
    usable: true,
    insertNote: "Lupin adds APP13 segments signed Lupin\\0 after the leading APPn segments. Decoders skip them.",
  };
}

function jpegName(m: number): string {
  if (m >= 0xe0 && m <= 0xef) return `APP${m - 0xe0}`;
  if (m >= 0xc0 && m <= 0xcf && m !== 0xc4 && m !== 0xc8 && m !== 0xcc) return "SOF";
  return { 0xc4: "DHT", 0xdb: "DQT", 0xdd: "DRI", 0xfe: "COM" }[m] ?? `FF${m.toString(16).toUpperCase()}`;
}

const MKV_IDS: Record<string, string> = {
  "1a45dfa3": "EBML header",
  "18538067": "Segment",
  "114d9b74": "SeekHead",
  "1549a966": "Info",
  "1654ae6b": "Tracks",
  "1f43b675": "Cluster",
  "1c53bb6b": "Cues",
  "1941a469": "Attachments",
  "1043a770": "Chapters",
  "1254c367": "Tags",
  ec: "Void",
  bf: "CRC-32",
};

/** Reads an EBML variable-length integer; ids keep their marker bit, sizes do not. */
function vint(b: Uint8Array, pos: number, keepMarker: boolean) {
  const width = Math.clz32(b[pos] ?? 0) - 23;
  if (width < 1 || width > 8 || pos + width > b.length) return null;
  let value = keepMarker ? b[pos] : b[pos] & (0xff >> width);
  let allOnes = value === 0xff >> width;
  for (let i = 1; i < width; i++) {
    value = value * 256 + b[pos + i];
    allOnes &&= b[pos + i] === 0xff;
  }
  return { value, width, unknown: !keepMarker && allOnes };
}

/** Finds a Lupin Void (0xEC, size, "Lupin\0", payload) in b[start..end], like the engine's find_carrier */
function findCarrier(b: Uint8Array, start: number, end: number): Region | null {
  for (let i = b.indexOf(LUPIN_SIG[0], start); i >= 0 && i + LUPIN_SIG.length <= end; i = b.indexOf(LUPIN_SIG[0], i + 1)) {
    if (!startsWith(b, i, LUPIN_SIG)) continue;
    for (let width = 1; width <= 8; width++) {
      const voidStart = i - width - 1;
      const size = voidStart >= start && b[voidStart] === 0xec ? vint(b, i - width, false) : null;
      if (size && size.width === width && !size.unknown && size.value >= LUPIN_SIG.length && i + size.value <= end) {
        return { name: "Void Lupin", start: voidStart, end: i + size.value, lupin: true };
      }
    }
  }
  return null;
}

function mkv(b: Uint8Array): Structure {
  const regions: Region[] = [];
  let insertAt = -1;
  // ponytail: walks the top level and the first Segment's direct children only. An unknown-size
  // child (a live-recorded Cluster) cannot be skipped without parsing it, so the rest of the
  // Segment is shown as one region with any Lupin Void found by signature.
  const walk = (pos: number, limit: number, inSegment: boolean) => {
    while (pos < limit && regions.length < MAX_REGIONS) {
      const id = vint(b, pos, true);
      const size = id && vint(b, pos + id.width, false);
      if (!id || !size) {
        regions.push({ name: "unparsed", start: pos, end: limit, lupin: false });
        return;
      }
      const name = MKV_IDS[id.value.toString(16)] ?? "element";
      const dataStart = pos + id.width + size.width;
      const end = size.unknown ? limit : Math.min(dataStart + size.value, limit);
      if (!inSegment && name === "Segment" && insertAt < 0) {
        regions.push({ name: "Segment header", start: pos, end: dataStart, lupin: false });
        walk(dataStart, end, true);
        insertAt = end;
      } else if (inSegment && size.unknown) {
        const carrier = findCarrier(b, pos, limit);
        if (carrier) regions.push({ name: "Segment data", start: pos, end: carrier.start, lupin: false }, carrier);
        regions.push({ name: "Segment data", start: carrier?.end ?? pos, end: limit, lupin: false });
        return;
      } else {
        const lupin = name === "Void" && startsWith(b, dataStart, LUPIN_SIG);
        regions.push({ name: lupin ? "Void Lupin" : name, start: pos, end, lupin });
      }
      pos = end;
    }
  };
  walk(0, b.length, false);
  return {
    format: "MKV",
    regions,
    insertAt: insertAt < 0 ? b.length : insertAt,
    usable: true,
    insertNote:
      "Lupin appends a Void element signed Lupin\\0 as the Segment's last child and, if the Segment declares a size, updates it. Players skip Void elements.",
  };
}

/** Merges runs of same-named regions, e.g. 57 IDAT chunks into "IDAT ×57" */
export function merge(regions: Region[]): Region[] {
  const out: (Region & { count: number })[] = [];
  for (const r of regions) {
    const last = out.at(-1);
    if (last && last.name === r.name && last.end === r.start) {
      last.end = r.end;
      last.count++;
    } else out.push({ ...r, count: 1 });
  }
  return out.map(({ count, ...r }) => (count > 1 ? { ...r, name: `${r.name} ×${count}` } : r));
}
