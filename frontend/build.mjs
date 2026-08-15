import { copyFile, mkdir, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import * as esbuild from "esbuild";

const require = createRequire(import.meta.url);
const frontend = dirname(fileURLToPath(import.meta.url));
const dist = join(frontend, "dist");

const BUNDLES = {
  "sqlpage.js": "src/sqlpage.ts",
  "apexcharts.js": "src/apexcharts.ts",
  "tomselect.js": "src/tomselect.ts",
  "sqlpage.css": "src/sqlpage.css",
};

const COPIES = {
  "favicon.svg": join(frontend, "src/favicon.svg"),
  "tabler-sprite.svg": require.resolve(
    "@tabler/icons-sprite/dist/tabler-sprite.svg",
  ),
};

function bundled_package(input_path) {
  const inside_node_modules = input_path.split("node_modules/");
  if (inside_node_modules.length < 2) return undefined;
  const [scope, name] = inside_node_modules.at(-1).split("/");
  return scope.startsWith("@") ? `${scope}/${name}` : scope;
}

function credits(metafile) {
  const bundled = Object.keys(metafile.inputs).map(bundled_package);
  const lines = [...new Set(bundled.filter(Boolean))].sort().map((name) => {
    const { version, license, homepage } = require(`${name}/package.json`);
    return ` * ${[`${name} ${version}`, license, homepage].filter(Boolean).join(" - ")}`;
  });
  if (lines.length === 0) return "";
  return `/*!\n * SQLPage bundles the following third-party code:\n${lines.join("\n")}\n */\n`;
}

async function build(name, entry) {
  const { outputFiles, metafile } = await esbuild.build({
    entryPoints: [join(frontend, entry)],
    outfile: join(dist, name),
    bundle: true,
    minify: true,
    charset: "utf8",
    legalComments: "inline",
    target: "es2022",
    metafile: true,
    write: false,
  });
  await writeFile(join(dist, name), credits(metafile) + outputFiles[0].text);
}

await mkdir(dist, { recursive: true });
await Promise.all([
  ...Object.entries(BUNDLES).map(([name, entry]) => build(name, entry)),
  ...Object.entries(COPIES).map(([name, from]) =>
    copyFile(from, join(dist, name)),
  ),
]);
