import { readFileSync } from "node:fs";
import { expect, test, type Page } from "@playwright/test";

const errors = new WeakMap<Page, string[]>();

test.beforeEach(async ({ page, baseURL }) => {
  const seen: string[] = [];
  errors.set(page, seen);
  page.on("console", (m) => m.type() === "error" && seen.push(m.text()));
  page.on("pageerror", (e) => seen.push(e.message));
  // Files must never leave the browser: fail on any request to another origin
  page.on("request", (r) => !r.url().startsWith(baseURL!) && !r.url().startsWith("blob:") && !r.url().startsWith("data:") && seen.push(`request to ${r.url()}`));
  await page.goto("/");
});

test.afterEach(async ({ page }) => expect(errors.get(page)).toEqual([]));

/** Loads a file from examples/ as the cover file */
const open = (page: Page, name: string) =>
  page.setInputFiles("#file", { name, mimeType: "application/octet-stream", buffer: readFileSync(new URL(`../../examples/${name}`, import.meta.url)) });

/** Loads cat.pdf as slow.pdf, taking a second to read, so a file chosen next finishes first */
async function openSlowly(page: Page) {
  await page.evaluate(() => {
    const read = File.prototype.arrayBuffer;
    File.prototype.arrayBuffer = function () {
      return this.name === "slow.pdf" ? new Promise((r) => setTimeout(r, 1000)).then(() => read.call(this)) : read.call(this);
    };
  });
  await page.setInputFiles("#file", { name: "slow.pdf", mimeType: "application/pdf", buffer: readFileSync(new URL("../../examples/cat.pdf", import.meta.url)) });
}

/** Clicks the button and returns the downloaded file */
async function download(page: Page, button: string) {
  const [dl] = await Promise.all([page.waitForEvent("download"), page.click(button)]);
  return { name: dl.suggestedFilename(), buffer: readFileSync((await dl.path())!) };
}

for (const sample of ["cat.png", "cat.jpg", "cat.pdf", "cat.mkv"]) {
  test(`hides and reveals text in ${sample}`, async ({ page }) => {
    await open(page, sample);
    await expect(page.locator("#status")).toContainText(`Lupin found no hidden data in ${sample}`);

    await page.fill("#text", "Meet me at the Louvre at eleven.");
    const out = await download(page, "#hide button");
    expect(out.name).toBe(sample.replace(".", "-lupin."));
    await expect(page.locator("#status")).toContainText(`Hid 32 bytes in ${sample}`);

    await page.setInputFiles("#file", { name: out.name, mimeType: "application/octet-stream", buffer: out.buffer });
    await expect(page.locator("#status")).toHaveText(`${out.name} hides 32 bytes:`);
    await expect(page.locator("#message")).toHaveText("Meet me at the Louvre at eleven.");
  });
}

test("reveals the message in out.png", async ({ page }) => {
  await open(page, "out.png");
  await expect(page.locator("#message")).toHaveText("meet me at the park at 5pm");
  const saved = await download(page, "#save");
  expect(saved.name).toBe("out-hidden.txt");
  expect(saved.buffer.toString()).toBe("meet me at the park at 5pm");
});

test("hides a binary file and returns it unchanged", async ({ page }) => {
  const secret = Buffer.from(Array.from({ length: 70_000 }, (_, i) => (i * 7) % 256)); // > one 64 KB JPEG segment
  await open(page, "cat.jpg");
  await page.getByLabel("A file").check();
  await page.setInputFiles("#payload", { name: "secret.bin", mimeType: "application/octet-stream", buffer: secret });
  const out = await download(page, "#hide button");

  await page.setInputFiles("#file", { name: out.name, mimeType: "image/jpeg", buffer: out.buffer });
  await expect(page.locator("#message")).toBeHidden();
  const saved = await download(page, "#save");
  expect(saved.name).toBe("cat-lupin-hidden.bin");
  expect(saved.buffer.equals(secret)).toBe(true);
});

test("asks for a payload instead of hiding nothing", async ({ page }) => {
  await open(page, "cat.png");
  await page.click("#hide button");
  await expect(page.locator("#status")).toHaveText("Type a message to hide first.");
});

test("rejects unsupported files", async ({ page }) => {
  await open(page, "cat.png");
  await expect(page.locator("#hide")).toBeVisible();
  await page.setInputFiles("#file", { name: "notes.txt", mimeType: "text/plain", buffer: Buffer.from("hello") });
  await expect(page.locator("#status")).toHaveText(
    "Lupin cannot use notes.txt (not a PDF, PNG, JPEG, MKV or WebM file). Choose a different file.",
  );
  await expect(page.locator("#hide")).toBeHidden();
});

test("accepts a file dropped anywhere on the page", async ({ page }) => {
  const bytes = [...readFileSync(new URL("../../examples/out.mkv", import.meta.url))];
  await page.evaluate((data) => {
    const dt = new DataTransfer();
    dt.items.add(new File([new Uint8Array(data)], "out.mkv"));
    document.querySelector("header")!.dispatchEvent(new DragEvent("drop", { dataTransfer: dt, bubbles: true, cancelable: true }));
  }, bytes);
  await expect(page.locator("#message")).toHaveText("meet me at the park at 5pm");
});

test("fits a phone screen without sideways scrolling", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 800 });
  await open(page, "out.png");
  await expect(page.locator("#reveal")).toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBe(390);
});

test("a hostile chunk length does not hang the page", async ({ page }) => {
  // A PNG chunk length of 0xFFFFFFF4 made 32-bit offset arithmetic wrap and loop forever
  const png = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0xff, 0xff, 0xff, 0xf4, ...Buffer.from("tEXt"), 0, 0, 0, 0];
  await page.setInputFiles("#file", { name: "evil.png", mimeType: "image/png", buffer: Buffer.from(png) });
  await expect(page.locator("#status")).toHaveText("Lupin cannot use evil.png (Invalid PNG: no IEND chunk found). Choose a different file.", {
    timeout: 3000,
  });
  await expect(page.locator("#hide")).toBeHidden();
});

test("choosing the same file twice loads it again", async ({ page }) => {
  await open(page, "out.png");
  await open(page, "cat.jpg");
  await expect(page.locator("#status")).toContainText("cat.jpg");
  await open(page, "out.png");
  await expect(page.locator("#status")).toContainText("out.png hides");
  // Browsers only fire change for the same file again if the input was cleared
  await expect(page.locator("#file")).toHaveValue("");
});

test("the most recent file wins when an earlier one loads slowly", async ({ page }) => {
  await openSlowly(page);
  await open(page, "cat.png");
  await page.waitForTimeout(1500);
  await expect(page.locator("#status")).toHaveText(/^Lupin found no hidden data in cat\.png/);
});

test("a hide cannot go to the previous file while a new one loads", async ({ page }) => {
  await open(page, "cat.png");
  await page.fill("#text", "x");
  await openSlowly(page);
  await expect(page.locator("#hide")).toBeHidden();
  await expect(page.locator("#status")).toHaveText(/^Lupin found no hidden data in slow\.pdf/);
});

test("resets the form and moves focus to the result after hiding", async ({ page }) => {
  await open(page, "cat.png");
  await page.getByLabel("A file").check();
  await open(page, "cat.jpg");
  await expect(page.getByLabel("A message")).toBeChecked();
  await expect(page.locator("#text")).toBeVisible();

  await page.fill("#text", "x");
  await download(page, "#hide button");
  expect(await page.evaluate(() => document.activeElement?.id)).toBe("status");
});

test("says so when WebAssembly cannot load", async ({ page }) => {
  await page.route("**/*.wasm", (route) => route.abort());
  await page.reload();
  const message = "This page could not load Lupin's WebAssembly module. Reload the page to try again.";
  await expect(page.locator("#status")).toHaveText(message);
  await open(page, "cat.png");
  await expect(page.locator("#status")).toHaveText(message);
  // The aborted request is logged as a failed resource load; nothing else may be
  const seen = errors.get(page)!;
  expect(seen.filter((e) => !e.startsWith("Failed to load resource"))).toEqual([]);
  seen.length = 0;
});

test("a chunk length ending at the 32-bit limit does not crash the engine", async ({ page }) => {
  // The next chunk offset lands on 0xFFFFFFFF, which used to overflow `pos + 8` on wasm32
  const png = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0xff, 0xff, 0xff, 0xeb, ...Buffer.from("IHDR"), ...Array(32).fill(0)];
  await page.setInputFiles("#file", { name: "edge.png", mimeType: "image/png", buffer: Buffer.from(png) });
  await expect(page.locator("#status")).toHaveText("Lupin cannot use edge.png (Invalid PNG: no IEND chunk found). Choose a different file.");
  await open(page, "out.png");
  await expect(page.locator("#message")).toHaveText("meet me at the park at 5pm");
});

test("reports damaged hidden data instead of offering to hide more", async ({ page }) => {
  const png = Buffer.from(readFileSync(new URL("../../examples/out.png", import.meta.url)));
  const chunk = png.indexOf("lpNg");
  png[chunk + 4] ^= 0xff; // breaks the chunk's CRC
  await open(page, "cat.png");
  await expect(page.locator("#hide")).toBeVisible();
  await page.setInputFiles("#file", { name: "damaged.png", mimeType: "image/png", buffer: png });
  await expect(page.locator("#status")).toHaveText("Lupin cannot use damaged.png (Corrupted hidden data in PNG). Choose a different file.");
  await expect(page.locator("#hide")).toBeHidden();
});

test("keeps the cover file when a file is dropped on the payload input", async ({ page }) => {
  await open(page, "cat.png");
  await page.getByLabel("A file").check();
  await page.evaluate(() => {
    const dt = new DataTransfer();
    dt.items.add(new File(["secret"], "secret.bin"));
    document.getElementById("payload")!.dispatchEvent(new DragEvent("drop", { dataTransfer: dt, bubbles: true, cancelable: true }));
  });
  await expect(page.locator("#status")).toContainText("Lupin found no hidden data in cat.png");
});

test("moves focus to the result after loading", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 800 });
  await open(page, "out.png");
  await expect(page.locator("#status")).toBeFocused();
  await expect(page.locator("#status")).toBeInViewport();
});

test("serves HTML with no-transform and assets as immutable", async ({ request }) => {
  // no-transform stops Cloudflare injecting scripts that the CSP would block
  for (const path of ["/", "/index.html"]) {
    expect((await request.get(path)).headers()["cache-control"]).toBe("public, max-age=0, must-revalidate, no-transform");
  }
  const html = await (await request.get("/")).text();
  const asset = html.match(/src="(\/assets\/[^"]+\.js)"/)![1];
  expect((await request.get(asset)).headers()["cache-control"]).toBe("public, max-age=31536000, immutable");
});
