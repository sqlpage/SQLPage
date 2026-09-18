import { copyFile, mkdir, readFile, writeFile } from "node:fs/promises";
import { rolldown } from "rolldown";

const DIST = "frontend/dist";

const ENTRIES = ["sqlpage", "apexcharts", "tomselect"];

const STYLESHEET = [
  "node_modules/@tabler/core/dist/css/tabler.min.css",
  "node_modules/tom-select/dist/css/tom-select.bootstrap5.css",
  "node_modules/@tabler/core/dist/css/tabler-vendors.min.css",
];

const COPIED = {
  "favicon.svg": "sqlpage/favicon.svg",
  "tabler-sprite.svg":
    "node_modules/@tabler/icons-sprite/dist/tabler-sprite.svg",
};

// An unresolved import is a warning, and rolldown then leaves the dependency
// out of the bundle instead of failing, so no warning may be ignored here.
function refuse(warning) {
  throw new Error(`Unable to bundle the frontend: ${warning.message}`);
}

async function bundle(entry) {
  const build = await rolldown({
    input: { [entry]: `sqlpage/${entry}.js` },
    onwarn: refuse,
  });
  await build.write({ dir: DIST, format: "iife", minify: true });
  await build.close();
}

async function stylesheet() {
  const vendored = await Promise.all(STYLESHEET.map((s) => readFile(s)));
  const own = await readFile("sqlpage/sqlpage.css");
  const parts = [...vendored.map((v) => `${v}\n`), own];
  await writeFile(`${DIST}/sqlpage.css`, parts.join(""));
}

await mkdir(DIST, { recursive: true });
await Promise.all([
  ...ENTRIES.map(bundle),
  stylesheet(),
  ...Object.entries(COPIED).map(([name, from]) =>
    copyFile(from, `${DIST}/${name}`),
  ),
]);
