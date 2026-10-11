/** Render a directed cut, frame by frame, around genuine SQLPage interactions. */
import { execFileSync } from "node:child_process";
import { copyFile, mkdir, readFile } from "node:fs/promises";
import path from "node:path";
import { chromium, expect } from "@playwright/test";

const root = process.cwd();
const brand = path.join(root, "examples/official-site/assets/brand");
const output = process.env.SQLPAGE_VIDEO_TEMP ?? "/tmp/sqlpage-brand-video";
const review = process.env.SQLPAGE_VIDEO_REVIEW === "1";
const baseURL = process.env.SQLPAGE_FIXTURE_URL ?? "http://127.0.0.1:8092";
await mkdir(output, { recursive: true });
const browser = await chromium.launch();
const context = await browser.newContext({
  viewport: { width: 1920, height: 1080 },
  deviceScaleFactor: 1,
  reducedMotion: "reduce",
});
const page = await context.newPage();
const assets = {
  "Outfit.woff2": "fonts/Outfit.woff2",
  "logo.svg": "logo-horizontal-light.svg",
  "mascot.png": "mascot-transparent.png",
};
await page.route("**/__brand-assets/*", async (route) => {
  const name = new URL(route.request().url()).pathname.split("/").pop();
  const file = assets[name];
  if (!file) return route.abort();
  await route.fulfill({
    body: await readFile(path.join(brand, file)),
    contentType: name.endsWith("woff2")
      ? "font/woff2"
      : name.endsWith("svg")
        ? "image/svg+xml"
        : "image/png",
  });
});
await page.route("**/__brand_video", (route) =>
  route.fulfill({
    path: path.join(root, "scripts/brand/video.html"),
    contentType: "text/html",
  }),
);
await page.route("**/__video-timeline.js", (route) =>
  route.fulfill({
    path: path.join(root, "scripts/brand/video-timeline.js"),
    contentType: "application/javascript",
  }),
);
page.on("pageerror", (error) => {
  throw error;
});
await page.goto(`${baseURL}/__brand_video`);
await page.evaluate(() => document.fonts.ready);
await page.waitForFunction(() => typeof window.renderVideo === "function");
const app = page.frameLocator("#app-frame");
await expect(app.getByRole("cell", { name: "Acme Corp" })).toBeVisible();
await app.locator("body").evaluate(() => document.fonts.ready);
await page.evaluate(() => window.renderVideo(0.8, { x: 1550, y: 520 }));
await page.screenshot({ path: path.join(output, "poster.png") });

const stills = new Set([
  0, 135, 195, 270, 315, 395, 490, 580, 655, 705, 765, 850,
]);
let pointer = { x: 1540, y: 760 };
let journey = { start: pointer, target: pointer, frame: 0 };
let clicked = -100;
let rowOffsets = [];
async function pointTo(locator, frame) {
  const box = await locator.boundingBox();
  if (!box) throw new Error("A directed control is outside the real app");
  journey = {
    start: pointer,
    target: { x: box.x + box.width / 2, y: box.y + box.height / 2 },
    frame,
  };
}
function cursor(frame) {
  const u = Math.min(1, Math.max(0, (frame - journey.frame) / 15));
  const ease = u * u * (3 - 2 * u);
  pointer = {
    x: journey.start.x + (journey.target.x - journey.start.x) * ease,
    y: journey.start.y + (journey.target.y - journey.start.y) * ease,
  };
  return { ...pointer, click: Math.max(0, 1 - (frame - clicked) / 16) };
}
for (let frame = 0; frame < 900; frame++) {
  const t = frame / 30;
  await page.evaluate(({ t, pointer }) => window.renderVideo(t, pointer), {
    t,
    pointer: cursor(frame),
  });
  if (frame === 350) await pointTo(app.getByRole("searchbox"), frame);
  if (frame === 376) {
    await app.getByRole("searchbox").focus();
    clicked = frame;
  }
  if (frame >= 380 && frame <= 400 && (frame - 380) % 4 === 0) {
    await app
      .getByRole("searchbox")
      .pressSequentially("Globex"[(frame - 380) / 4]);
  }
  if (frame === 424) {
    await expect(
      app.getByRole("cell", { name: "Globex", exact: true }),
    ).toBeVisible();
    await expect(app.getByRole("cell", { name: "Acme Corp" })).toBeHidden();
  }
  if (frame === 445) {
    await app.getByRole("searchbox").fill("");
    await pointTo(
      app.getByRole("button", { name: "seats", exact: true }),
      frame,
    );
  }
  if (frame === 475) {
    const rows = app.locator("tbody tr");
    const before = await rows.evaluateAll((rows) =>
      rows.map((row) => ({
        name: row.textContent,
        y: row.getBoundingClientRect().top,
      })),
    );
    await app.getByRole("button", { name: "seats", exact: true }).click();
    clicked = frame;
    await expect(rows.first()).toContainText("Initech");
    rowOffsets = await rows.evaluateAll(
      (rows, before) =>
        rows.map((row) => {
          const old = before.find((item) => item.name === row.textContent);
          return (
            (old?.y ?? row.getBoundingClientRect().top) -
            row.getBoundingClientRect().top
          );
        }),
      before,
    );
  }
  if (frame === 510)
    await pointTo(app.getByRole("button", { name: "Add a customer" }), frame);
  if (frame === 533) {
    await app.getByRole("button", { name: "Add a customer" }).click();
    clicked = frame;
    await expect(app.getByLabel("Customer name")).toBeVisible();
  }
  if (frame === 546) await pointTo(app.getByLabel("Customer name"), frame);
  if (frame === 562) {
    await app.getByLabel("Customer name").focus();
    clicked = frame;
  }
  if (frame >= 564 && frame <= 609 && (frame - 564) % 3 === 0) {
    await app
      .getByLabel("Customer name")
      .pressSequentially("Northstar Studio"[(frame - 564) / 3]);
  }
  if (frame === 612)
    await pointTo(app.getByRole("button", { name: "Save customer" }), frame);
  if (frame === 633) {
    await app.getByRole("button", { name: "Save customer" }).click();
    clicked = frame;
    await expect(
      app.getByRole("cell", { name: "Northstar Studio" }),
    ).toBeInViewport();
  }
  if (frame === 650)
    await pointTo(app.getByRole("cell", { name: "Northstar Studio" }), frame);
  if (frame === 685) {
    const applicationFrame = page
      .frames()
      .find(
        (frame) => frame.parentFrame() && frame.url().includes("/brand-demo/"),
      );
    await Promise.all([
      applicationFrame.waitForNavigation({ waitUntil: "networkidle" }),
      applicationFrame.evaluate(() => location.reload()),
    ]);
    await expect(
      app.getByRole("cell", { name: "Northstar Studio" }),
    ).toBeVisible();
  }
  await app.locator("body").evaluate(
    (_body, { t, frame, rowOffsets }) => {
      const headers = [...document.querySelectorAll("thead th")];
      for (const [index, header] of headers.entries()) {
        const active =
          t >= 8.7 && t < 10.9 && index === (t < 9.4 ? 0 : t < 10.1 ? 1 : 3);
        header.style.background = active ? "#b5e9f1" : "";
        header.style.color = active ? "#080e11" : "";
      }
      const rows = [...document.querySelectorAll("tbody tr")];
      const progress = Math.max(0, Math.min(1, (frame - 475) / 18));
      const ease = 1 - (1 - progress) ** 3;
      for (const [index, row] of rows.entries()) {
        row.style.transform =
          frame >= 475 && frame < 493
            ? `translateY(${(rowOffsets[index] ?? 0) * (1 - ease)}px)`
            : "";
        const saved =
          t >= 21.3 && t < 24 && row.textContent.includes("Northstar Studio");
        row.style.background = saved ? "#b5e9f1" : "";
        for (const cell of row.children)
          cell.style.background = saved ? "#b5e9f1" : "";
        row.style.boxShadow = saved ? "inset 5px 0 #146575" : "";
      }
    },
    { t, frame, rowOffsets },
  );
  if (!review || stills.has(frame)) {
    await page.screenshot({
      path: path.join(output, `frame-${String(frame).padStart(4, "0")}.jpg`),
      type: "jpeg",
      quality: 94,
    });
  }
  if (frame % 90 === 0)
    console.log(`Rendered ${t}s / 30s${review ? " (review stills)" : ""}`);
}
await context.close();
await browser.close();
if (review) {
  console.log(`Review frames: ${output}`);
  process.exit(0);
}
function ffmpeg(args) {
  execFileSync(
    "ffmpeg",
    ["-hide_banner", "-loglevel", "error", "-y", ...args],
    { stdio: "inherit" },
  );
}
const mp4 = path.join(root, "docs/sqlpage.mp4");
const webm = path.join(root, "docs/sqlpage.webm");
ffmpeg([
  "-framerate",
  "30",
  "-i",
  path.join(output, "frame-%04d.jpg"),
  "-t",
  "30",
  "-an",
  "-r",
  "30",
  "-c:v",
  "libx264",
  "-crf",
  "23",
  "-preset",
  "slow",
  "-pix_fmt",
  "yuv420p",
  "-movflags",
  "+faststart",
  mp4,
]);
ffmpeg([
  "-i",
  mp4,
  "-an",
  "-c:v",
  "libvpx-vp9",
  "-crf",
  "35",
  "-b:v",
  "0",
  "-row-mt",
  "1",
  "-cpu-used",
  "4",
  webm,
]);
// Eight-second teaser: table, SQL, and the actual save, with readable pauses.
ffmpeg([
  "-i",
  mp4,
  "-filter_complex",
  "[0:v]split=3[a][b][c];[a]trim=start=0:end=2,setpts=PTS-STARTPTS[a1];[b]trim=start=5:end=8,setpts=PTS-STARTPTS[b1];[c]trim=start=21:end=24,setpts=PTS-STARTPTS[c1];[a1][b1][c1]concat=n=3:v=1:a=0,fps=8,scale=960:-1:flags=lanczos,split[x][y];[x]palettegen=max_colors=96[p];[y][p]paletteuse=dither=bayer",
  "-loop",
  "0",
  path.join(root, "docs/sqlpage.gif"),
]);
await copyFile(
  path.join(output, "poster.png"),
  path.join(root, "docs/sqlpage-poster.png"),
);
execFileSync("convert", [
  path.join(output, "poster.png"),
  "-quality",
  "92",
  path.join(root, "docs/sqlpage-poster.webp"),
]);
console.log(`Recorded and encoded the real customer app: ${mp4}`);
