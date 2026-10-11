import { mkdir, readFile, writeFile } from "node:fs/promises";
import { chromium } from "@playwright/test";

const base = process.env.SQLPAGE_BRAND_URL || "http://127.0.0.1:8091";
const destination = process.argv[2] || "/tmp/sqlpage-brand-render";
await mkdir(destination, { recursive: true });
const browser = await chromium.launch({
  args: ["--use-angle=swiftshader", "--enable-unsafe-swiftshader"],
});
const page = await browser.newPage({
  viewport: { width: 1000, height: 1000 },
  deviceScaleFactor: 2,
});
// Omit the surrounding floor/orbits only in the isolated artwork capture.
// The official renderer, geometry, materials, camera, and lighting stay intact.
const rendererSource = await readFile(
  "examples/official-site/assets/landing/js/scene/database-scene.js",
  "utf8",
);
const decorationMount = "parallaxPivot.add(decoration.group);";
const decorationGlow = "glow.render(model, [decoration.group], screenBounds);";
if (
  !rendererSource.includes(decorationMount) ||
  !rendererSource.includes(decorationGlow)
)
  throw new Error(
    "Review the artwork-only decoration filter after renderer changes",
  );
await page.route("**/assets/landing/js/scene/database-scene.js", (route) =>
  route.fulfill({
    contentType: "application/javascript",
    body: rendererSource
      .replace(decorationMount, "")
      .replace(decorationGlow, "glow.render(model, [], screenBounds);"),
  }),
);
await page.route("**/__brand_export", (route) =>
  route.fulfill({
    contentType: "text/html",
    body: `<!doctype html><html><head><style>html,body{margin:0;background:transparent}#mount{position:absolute;inset:0}#hit{position:absolute}</style></head><body><div id="mount" data-brand-export></div><div id="hit"></div><script type="module">
import {createDatabaseScene} from '/assets/landing/js/scene/database-scene.js';
createDatabaseScene({mount:document.querySelector('#mount'),hitArea:document.querySelector('#hit'),getState:()=>({motion:false,opacity:1,turn:0,frame:{left:0,top:0,size:1000}}),onReady:()=>document.body.dataset.ready='true',onError:()=>document.body.dataset.ready='error'});
</script></body></html>`,
  }),
);
page.on("pageerror", (error) => process.stderr.write(`${error.message}\n`));
await page.goto(`${base}/__brand_export`);
await page.waitForFunction(() => document.body.dataset.ready, undefined, {
  timeout: 60000,
});
if ((await page.locator("body").getAttribute("data-ready")) !== "true")
  throw new Error("The reference sculpture could not render");
await page.screenshot({
  path: `${destination}/mascot-transparent.png`,
  omitBackground: true,
});
await writeFile(
  `${destination}/render.json`,
  JSON.stringify(
    {
      model: "sqlpage-database-bbca200908a1.glb",
      camera: "sculpture-layout.js",
      frame: 0,
      excluded: ["surrounding floor and orbit decoration"],
      viewport: [1000, 1000],
      scale: 2,
    },
    null,
    2,
  ),
);
await browser.close();
process.stdout.write(`Exported the actual sculpture to ${destination}\n`);
