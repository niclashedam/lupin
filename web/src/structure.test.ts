import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { inspect, merge } from "./structure.ts";

const read = (name: string) => new Uint8Array(readFileSync(new URL(`../../examples/${name}`, import.meta.url)));

for (const ext of ["pdf", "png", "jpg", "mkv"]) {
  test(`${ext}: regions tile the file and only out.${ext} has a Lupin region`, () => {
    for (const [name, hidden] of [[`cat.${ext}`, false], [`out.${ext}`, true]] as const) {
      const bytes = read(name);
      const s = inspect(bytes)!;
      let pos = 0;
      for (const r of s.regions) {
        assert.equal(r.start, pos, `${name}: gap before ${r.name}`);
        pos = r.end;
      }
      assert.equal(pos, bytes.length, `${name}: regions stop early`);
      assert.equal(merge(s.regions).filter((r) => r.lupin).length, hidden ? 1 : 0, name);
    }
  });
}

const bytes = (...parts: (number[] | string)[]) =>
  new Uint8Array(parts.flatMap((p) => (typeof p === "string" ? [...p].map((c) => c.charCodeAt(0)) : p)));
const lupinRegions = (b: Uint8Array) => inspect(b)!.regions.filter((r) => r.lupin);

test("mkv: finds the Lupin Void after an unknown-size Cluster", () => {
  const ebml = [0x1a, 0x45, 0xdf, 0xa3, 0x84, 0x42, 0x82, 0x01, 0x02];
  const b = bytes(ebml, [0x18, 0x53, 0x80, 0x67, 0xff], [0x1f, 0x43, 0xb6, 0x75, 0xff, 1, 2, 3], [0xec, 0x8b], "Lupin\0hello");
  assert.deepEqual(lupinRegions(b), [{ name: "Void Lupin", start: 22, end: b.length, lupin: true }]);
});

test("mkv: inserts into the first Segment only", () => {
  const ebml = [0x1a, 0x45, 0xdf, 0xa3, 0x84, 0x42, 0x82, 0x01, 0x02];
  const segment = [0x18, 0x53, 0x80, 0x67, 0x82, 0x11, 0x11];
  assert.equal(inspect(bytes(ebml, segment, segment))!.insertAt, 16);
});

test("pdf: a vertical tab after %%EOF is data, as in Rust's is_ascii_whitespace", () => {
  assert.equal(lupinRegions(bytes("%PDF-1.4\n%%EOF\n")).length, 0);
  assert.equal(lupinRegions(bytes("%PDF-1.4\n%%EOF\v")).length, 1);
});

test("random and truncated input terminates and tiles the file", () => {
  const seeds = [read("cat.png"), read("out.jpg"), read("out.mkv"), read("out.pdf")].map((b) => b.subarray(0, 4096));
  let x = 1;
  const rand = () => ((x = (x * 1103515245 + 12345) & 0x7fffffff) / 0x7fffffff);
  for (let n = 0; n < 3000; n++) {
    const b = seeds[n % 4].slice(0, Math.floor(rand() * 4096));
    for (let k = 0; k < 4; k++) b[Math.floor(rand() * b.length)] = Math.floor(rand() * 256);
    const s = inspect(b);
    if (!s) continue;
    let pos = 0;
    for (const r of s.regions) {
      assert.equal(r.start, pos);
      pos = r.end;
    }
    assert.equal(pos, b.length);
  }
});

test("hostile and huge files are walked quickly with a bounded number of regions", () => {
  const cat = read("cat.png");
  const zeroTail = new Uint8Array(cat.length + 20_000_000);
  zeroTail.set(cat.subarray(0, cat.length - 12)); // drop IEND, then 20 MB of zeros
  const tinyChunks = new Uint8Array(8 + 12 * 200_000);
  tinyChunks.set(cat.subarray(0, 8));
  for (let pos = 8; pos < tinyChunks.length; pos += 12) tinyChunks.set([0x74, 0x45, 0x58, 0x74], pos + 4); // empty tEXt
  const noEof = new Uint8Array(100_000_000);
  noEof.set([0x25, 0x50, 0x44, 0x46]);
  for (const b of [zeroTail, tinyChunks, noEof]) {
    const started = performance.now();
    const s = inspect(b)!;
    assert.ok(performance.now() - started < 1000, "took over a second");
    assert.ok(s.regions.length <= 10_002, `${s.regions.length} regions`);
    assert.equal(s.regions.at(-1)!.end, b.length);
  }
});
