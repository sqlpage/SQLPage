import { execFileSync } from "node:child_process";
import {
  copyFileSync,
  cpSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  renameSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { parseArgs } from "node:util";
import { buildDocumentationSeed } from "./site-content.mjs";

export { buildDocumentationSeed } from "./site-content.mjs";

export const repository = resolve(import.meta.dirname, "..");
export const siteDirectory = join(repository, "target", "official-site");
const source = join(repository, "examples", "official-site");

function configurationSchema(file) {
  if (file) return JSON.parse(readFileSync(file, "utf8"));
  if (!existsSync(join(repository, "frontend", "dist", "tabler-sprite.svg"))) {
    execFileSync(process.execPath, ["scripts/build-frontend.mjs"], {
      cwd: repository,
      stdio: "inherit",
    });
  }
  return JSON.parse(
    execFileSync(
      "cargo",
      [
        "run",
        "--locked",
        "--quiet",
        "--features",
        "config-schema",
        "--example",
        "export_config_schema",
      ],
      {
        cwd: repository,
        encoding: "utf8",
        stdio: ["ignore", "pipe", "inherit"],
      },
    ),
  );
}

// Only this directory is shipped to the server. In particular, SQL files used to
// build the database must never become URLs beneath the public web root.
const excluded = new Set([
  "content",
  "README.md",
  "docker-compose.yml",
  "test.hurl",
  join("sqlpage", "migrations"),
  join("sqlpage", "sqlpage.db"),
  join("sqlpage", "https"),
]);

export function buildSite(schemaFile) {
  const seed = buildDocumentationSeed(configurationSchema(schemaFile));
  const parent = dirname(siteDirectory);
  mkdirSync(parent, { recursive: true });
  const temporary = mkdtempSync(join(parent, ".official-site-"));
  const next = join(temporary, "next");
  const previous = join(temporary, "previous");
  try {
    cpSync(source, next, {
      recursive: true,
      filter: (file) => !excluded.has(relative(source, file)),
    });
    copyFileSync(
      join(repository, "configuration.md"),
      join(next, "configuration.md"),
    );
    const migrations = join(next, "sqlpage", "migrations");
    mkdirSync(migrations, { recursive: true });
    writeFileSync(join(migrations, "001_official_site.sql"), seed);
    // Validate and stage everything before exchanging the previous preview.
    try {
      renameSync(siteDirectory, previous);
    } catch (error) {
      if (error.code !== "ENOENT") throw error;
    }
    try {
      renameSync(next, siteDirectory);
    } catch (error) {
      if (!existsSync(siteDirectory) && existsSync(previous)) {
        renameSync(previous, siteDirectory);
      }
      throw error;
    }
    console.log(`Prepared ${siteDirectory}`);
  } finally {
    rmSync(temporary, { force: true, recursive: true });
  }
}

if (import.meta.main) {
  const { values } = parseArgs({ options: { schema: { type: "string" } } });
  buildSite(values.schema);
}
