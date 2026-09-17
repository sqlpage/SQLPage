import { mkdir } from "node:fs/promises";
import { rolldown } from "rolldown";

const DIST = "frontend/dist";

const ENTRIES = ["sqlpage", "apexcharts", "tomselect"];

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

await mkdir(DIST, { recursive: true });
await Promise.all(ENTRIES.map(bundle));
