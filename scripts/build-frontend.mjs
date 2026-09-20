import { createHash } from "node:crypto";
import { copyFile, mkdir, readFile, writeFile } from "node:fs/promises";
import { gzipSync } from "node:zlib";
import { rolldown } from "rolldown";

const DIST = "frontend/dist";

const SCRIPTS = ["sqlpage", "apexcharts", "tomselect"];

const STYLESHEET = [
  "node_modules/@tabler/core/dist/css/tabler.min.css",
  "node_modules/tom-select/dist/css/tom-select.bootstrap5.css",
  "node_modules/@tabler/core/dist/css/tabler-vendors.min.css",
  "frontend/src/sqlpage.css",
];

const ICON_SPRITE = "node_modules/@tabler/icons-sprite/dist/tabler-sprite.svg";

// An unresolved import is a warning, and rolldown then leaves the dependency
// out of the bundle instead of failing, so no warning may be ignored here.
function refuse(warning) {
  throw new Error(`Unable to bundle the frontend: ${warning.message}`);
}

async function serve(name, content) {
  const hash = createHash("sha256").update(content).digest("hex").slice(0, 16);
  await writeFile(`${DIST}/${name}.gz`, gzipSync(content, { level: 9 }));
  await writeFile(
    `${DIST}/${name}.filename.txt`,
    name.replace(/\.(\w+)$/, `.${hash}.$1`),
  );
}

async function script(entry) {
  const build = await rolldown({
    input: { [entry]: `frontend/src/${entry}.js` },
    onwarn: refuse,
  });
  const { output } = await build.generate({ format: "iife", minify: true });
  await build.close();
  await serve(`${entry}.js`, output[0].code);
}

async function stylesheet() {
  const parts = await Promise.all(STYLESHEET.map((p) => readFile(p, "utf8")));
  await serve("sqlpage.css", parts.join("\n"));
}

await mkdir(DIST, { recursive: true });
await Promise.all([
  ...SCRIPTS.map(script),
  stylesheet(),
  readFile("frontend/src/favicon.svg").then((svg) => serve("favicon.svg", svg)),
  copyFile(ICON_SPRITE, `${DIST}/tabler-sprite.svg`),
]);
