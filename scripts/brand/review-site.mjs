import { execFileSync } from "node:child_process";
import { mkdir } from "node:fs/promises";
import { chromium, expect } from "@playwright/test";

const output = process.env.SQLPAGE_REVIEW_TEMP ?? "/tmp/sqlpage-brand-review";
const base = process.env.SQLPAGE_BRAND_URL ?? "http://127.0.0.1:8091";
await mkdir(output, { recursive: true });
const browser = await chromium.launch({
  args: ["--use-angle=swiftshader", "--enable-unsafe-swiftshader"],
});
const page = await browser.newPage({
  viewport: { width: 1440, height: 1000 },
  reducedMotion: "reduce",
});
for (const width of [390, 768, 1440]) {
  await page.setViewportSize({ width, height: 1000 });
  await page.goto(base);
  await page.evaluate(() => document.fonts.ready);
  await expect(page.locator(".sqlpage-world")).toHaveAttribute(
    "data-scene",
    "ready",
    { timeout: 30000 },
  );
  await page.screenshot({ path: `${output}/hero-${width}.png` });
  await page.locator("#components").scrollIntoViewIfNeeded();
  await page.screenshot({ path: `${output}/demo-${width}.png` });
  await page.goto(`${base}/documentation.sql`);
  await page.evaluate(() => document.fonts.ready);
  await page.screenshot({ path: `${output}/documentation-${width}.png` });
}
await page.goto(base);
for (const [name, selector] of [
  ["building", "#deployment-title"],
  ["compatibility", "#databases-title"],
  ["maintenance", "#commit-title"],
]) {
  await page.locator(selector).scrollIntoViewIfNeeded();
  await page.screenshot({ path: `${output}/${name}.png` });
}
await page.goto(`${base}/your-first-sql-website/`);
await page.screenshot({ path: `${output}/tutorial.png` });
await page.goto(`${base}/visual-identity.sql`);
await page.screenshot({ path: `${output}/identity-guide.png` });
await page.route("**/__brand-review", (route) =>
  route.fulfill({
    contentType: "text/html",
    body: `<!doctype html><html><head><link rel="stylesheet" href="/assets/brand/brand.css"><style>body{margin:0;padding:32px;background:#080e11;color:#f4f7f8;font:18px Outfit}section{display:flex;align-items:center;gap:24px;margin-bottom:24px}.surface{padding:24px;background:#f4f7f8;color:#080e11}.social{position:relative;width:900px}.social img{width:100%;display:block}.avatar{position:absolute;bottom:-16px;left:24px;width:100px;height:100px;border:4px solid #f4f7f8;border-radius:100%}.safe{position:absolute;left:19.8%;top:35.3%;width:60.4%;height:29.4%;border:1px solid #58cce0}h2{font-size:24px;font-weight:500}</style></head><body><h2>Navigation, small icons, and light/dark lockups</h2><section><img src="/assets/brand/logo-horizontal-light.svg" width="185"><img src="/assets/brand/favicon.svg" width="16" height="16"><img src="/assets/brand/favicon.svg" width="32" height="32"></section><section class="surface"><img src="/assets/brand/logo-horizontal-dark.svg" width="185"><img src="/assets/brand/symbol-dark.svg" height="64"></section><h2>Open Graph and GitHub</h2><section><img src="/assets/brand/social-og.png" width="600"><img src="/assets/brand/social-github.png" width="600"></section><h2>X avatar overlap</h2><div class="social"><img src="/assets/brand/social-x.png"><div class="avatar"></div></div><h2>YouTube central safe area</h2><div class="social"><img src="/assets/brand/social-youtube.png"><div class="safe"></div></div></body></html>`,
  }),
);
await page.goto(`${base}/__brand-review`);
await page.evaluate(() => document.fonts.ready);
await page.screenshot({ path: `${output}/brand-crops.png`, fullPage: true });
await browser.close();
for (const [index, time] of [
  0.1, 4.5, 8.5, 13.5, 16, 19.5, 22.5, 25.5,
].entries()) {
  execFileSync("ffmpeg", [
    "-hide_banner",
    "-loglevel",
    "error",
    "-y",
    "-ss",
    String(time),
    "-i",
    "docs/sqlpage.mp4",
    "-frames:v",
    "1",
    "-vf",
    "scale=640:-1",
    `${output}/video-${index}.png`,
  ]);
}
console.log(`Visual review captures: ${output}`);
