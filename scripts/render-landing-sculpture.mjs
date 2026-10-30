/** Regenerate both fallbacks from the production scene. Requires ImageMagick. */
import { execFile } from "node:child_process";
import { copyFile, mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";
import { chromium } from "@playwright/test";

const run = promisify(execFile);
const baseURL = process.argv[2];
if (!baseURL)
  throw new Error(
    "Usage: node scripts/render-landing-sculpture.mjs <local site URL>",
  );
const assets = fileURLToPath(
  new URL("../examples/official-site/assets/landing/assets/", import.meta.url),
);
const temporary = await mkdtemp(join(tmpdir(), "sqlpage-sculpture-"));
let browser;
try {
  browser = await chromium.launch({ args: ["--enable-unsafe-swiftshader"] });
  const page = await browser.newPage({
    viewport: { width: 1000, height: 1000 },
    deviceScaleFactor: 1,
  });
  const errors = [];
  page.on("pageerror", (error) => errors.push(error.message));
  // Offline rendering must never choose the low-power atlas it is replacing.
  // Keep this override identical to the desktop pose test in landing.spec.ts.
  await page.route(
    "**/assets/landing/js/scene/database-scene.js",
    async (route) => {
      const response = await route.fetch();
      const source = await response.text();
      const timing = "const elapsed = performance.now() - submittedAt;";
      if (!source.includes(timing))
        throw new Error("Update the offline GPU timing override");
      await route.fulfill({
        response,
        body: source.replace(timing, "const elapsed = 0;"),
      });
    },
  );
  await page.route("**/__sculpture-render", (route) =>
    route.fulfill({
      contentType: "text/html",
      body: `<!doctype html><html><body style="margin:0">
      <div class="sqlpage-world"><div id="mount"></div><div id="hit"></div></div>
      <script type="module">
        import { createDatabaseScene } from '/assets/landing/js/scene/database-scene.js';
        window.sculptureState = {motion:false, opacity:1, turn:0, frame:{left:0,top:0,size:1000}};
        createDatabaseScene({
          mount:document.querySelector('#mount'), hitArea:document.querySelector('#hit'),
          getState:()=>window.sculptureState,
          onReady:()=>{window.sceneReady=true},
          onError:()=>{throw new Error('Sculpture render failed')}
        });
      </script></body></html>`,
    }),
  );
  await page.goto(new URL("/__sculpture-render", baseURL).href);
  await page.waitForFunction(() => window.sceneReady, null, {
    timeout: 60_000,
  });
  const frames = [];
  for (let i = 0; i < 64; i++) {
    const turn = (i / 64) * Math.PI * 2;
    await page.evaluate((turn) => {
      window.sculptureState.turn = turn;
    }, turn);
    await page.waitForFunction(
      (turn) =>
        document.querySelector("canvas").dataset.scrollTurn === String(turn),
      turn,
    );
    // Wait for submission and paint before capturing the transparent square.
    await page.evaluate(
      () =>
        new Promise((resolve) =>
          requestAnimationFrame(() => requestAnimationFrame(resolve)),
        ),
    );
    const frame = join(temporary, `${String(i).padStart(2, "0")}.png`);
    await page
      .locator("canvas")
      .screenshot({ path: frame, omitBackground: true });
    frames.push(frame);
  }
  if (errors.length) throw new Error(errors.join("\n"));
  await run("magick", [
    frames[0],
    "-quality",
    "88",
    join(temporary, "scene-first-frame.webp"),
  ]);
  await run("magick", [
    "montage",
    ...frames,
    "-geometry",
    "400x400+0+0",
    "-tile",
    "8x8",
    "-background",
    "none",
    "-quality",
    "65",
    "-define",
    "heic:speed=4",
    join(temporary, "sculpture-turns.avif"),
  ]);
  // Leave the versioned pair intact if either encoder fails.
  for (const asset of ["scene-first-frame.webp", "sculpture-turns.avif"])
    await copyFile(join(temporary, asset), join(assets, asset));
  console.log(
    "Rendered the first frame and 64 low-power views from the production scene.",
  );
} finally {
  await browser?.close();
  await rm(temporary, { recursive: true, force: true });
}
